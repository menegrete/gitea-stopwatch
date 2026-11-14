//! End-to-end tests for the gitea-stopwatch desktop shell.
//!
//! Every test works in its own isolated temp tree reached through explicit path
//! seams, never the developer's real application data, and removes that tree
//! before finishing. The test process environment is never mutated: a
//! process-wide mutation races with the threads the test harness runs in, so
//! isolation is carried by path arguments instead of environment overrides.
//!
//! The shell behaviour tests drive the crate's public surface with explicit
//! isolated paths (`dirs::user_data_dir_from`, `core::config_path_in`,
//! `core::load_geometry_at`, `core::save_geometry_at`, `platform::lock`) so
//! they run headless in CI without a display server:
//!
//! - `single-instance-lock` → `second-launch-focuses-existing-instance`
//! - `window-geometry-persistence` → `geometry-restored-and-clamped`
//! - `logging-and-panic-hook` → `logging-and-panic-hook-wired`
//!
//! The last test executes the *shipped binary* instead of the crate surface,
//! which is what proves the shell scenarios by execution:
//!
//! - `empty-main-window` → `window-opens-with-title-and-theme`,
//!   `clean-shutdown`
//! - `placeholder-navigation` → `placeholder-navigation-renders`
//!
//! The Iced shell needs an X11 display, so that test resolves a display
//! harness first: `DISPLAY` when it is set and reachable, otherwise
//! `xvfb-run -a` when that tool exists. When neither is available the test
//! prints a skip reason and passes, and the limitation is recorded in
//! `docs/standards-gaps.md`.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use gitea_stopwatch::core::{
    Error, Geometry, clamp_to_monitor, config_path_in, load_geometry_at, save_geometry_at,
};
use gitea_stopwatch::platform::instance::LaunchRole;
use gitea_stopwatch::platform::{dirs, focus, instance, lock, logging};
use gitea_stopwatch::ui::window::{
    APP_TITLE, AUTO_CLOSE_ENV, PLACEHOLDER_NAV_HINT, PLACEHOLDER_NAV_TITLE, WINDOW_THEME,
};

/// Name of the single-instance lock file (mirrors `platform::lock`).
const LOCK_FILE_NAME: &str = "gitea-stopwatch.lock";

/// Field names that would indicate a credential leaked into a log line.
const FORBIDDEN_LOG_FIELDS: [&str; 4] = ["token", "password", "secret", "api_key"];

/// Literal markers that would indicate an authentication secret in a log line.
const FORBIDDEN_LOG_MARKERS: [&str; 6] = [
    "authorization:",
    "bearer ",
    "ghp_",
    "github_pat_",
    "client_secret",
    "-----begin",
];

/// Monitor width the geometry is clamped against (narrower than the window).
const MONITOR_WIDTH: f64 = 1024.0;
/// Monitor height the geometry is clamped against (shorter than the window).
const MONITOR_HEIGHT: f64 = 768.0;

/// Build a unique isolated temp directory path for this test run.
fn isolated_dir(suffix: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    env::temp_dir().join(format!(
        "gitea-stopwatch-e2e-{}-{}-{}",
        std::process::id(),
        nanos,
        suffix
    ))
}

/// End-to-end check that the application's per-user state is isolated to an
/// explicit base directory and fully removed on cleanup.
///
/// The directories are resolved from explicit bases rather than from the
/// process-wide environment overrides, so nothing here mutates the environment
/// the other test threads run in.
#[test]
fn e2e_uses_isolated_user_data_and_cleans_up() {
    let data_base = isolated_dir("data");
    let cache_base = isolated_dir("cache");

    // Arrange: remember the environment overrides as they are, so the test can
    // prove afterwards that it never changed them (reading is safe).
    let env_data_before = env::var_os("GITEA_STOPWATCH_DATA_DIR");
    let env_cache_before = env::var_os("GITEA_STOPWATCH_CACHE_DIR");

    // Act: resolve user data and cache directories from explicit bases through
    // the app's platform abstraction.
    let resolved_data = dirs::user_data_dir_from(&data_base);
    let resolved_cache = dirs::cache_dir_from(&cache_base);

    // Assert: directories live under the isolated temp locations, never the
    // real application data directory.
    assert!(
        resolved_data.starts_with(&data_base),
        "user data dir should be under isolated data dir, got {:?}",
        resolved_data
    );
    assert!(
        resolved_data.ends_with("gitea-stopwatch"),
        "user data dir should be namespaced by app name"
    );
    assert!(
        resolved_cache.starts_with(&cache_base),
        "cache dir should be under isolated cache dir, got {:?}",
        resolved_cache
    );
    assert!(
        resolved_cache.ends_with("gitea-stopwatch"),
        "cache dir should be namespaced by app name"
    );

    // Act: the application materializes its state directories on first launch.
    fs::create_dir_all(&resolved_data).expect("the isolated user data dir must be created");
    fs::create_dir_all(&resolved_cache).expect("the isolated cache dir must be created");

    // Assert: the state now lives entirely inside the isolated tree.
    assert!(
        resolved_data.is_dir(),
        "the isolated user data dir must exist at {:?}",
        resolved_data
    );
    assert!(
        resolved_cache.is_dir(),
        "the isolated cache dir must exist at {:?}",
        resolved_cache
    );

    // Assert: window title is present (title presence check).
    assert!(
        !APP_TITLE.is_empty(),
        "window title (APP_TITLE) must be present"
    );
    assert_eq!(
        APP_TITLE, "gitea-stopwatch",
        "window title should match the app's advertised title"
    );

    // Cleanup: remove the isolated directory trees.
    let _ = fs::remove_dir_all(&data_base);
    let _ = fs::remove_dir_all(&cache_base);

    // Assert: cleanup removed everything the test created.
    assert!(
        !resolved_data.exists() && !data_base.exists(),
        "the isolated user data dir must be removed after the test"
    );
    assert!(
        !resolved_cache.exists() && !cache_base.exists(),
        "the isolated cache dir must be removed after the test"
    );

    // Assert: the process environment was never touched, so parallel tests
    // cannot race on it.
    assert_eq!(
        env::var_os("GITEA_STOPWATCH_DATA_DIR"),
        env_data_before,
        "the test must not mutate GITEA_STOPWATCH_DATA_DIR"
    );
    assert_eq!(
        env::var_os("GITEA_STOPWATCH_CACHE_DIR"),
        env_cache_before,
        "the test must not mutate GITEA_STOPWATCH_CACHE_DIR"
    );
}

#[test]
fn e2e_window_title_is_present() {
    assert!(!APP_TITLE.is_empty(), "window title must not be empty");
    assert_eq!(APP_TITLE, "gitea-stopwatch");
}

/// End-to-end check for the `single-instance-lock` requirement
/// (scenario `second-launch-focuses-existing-instance`).
///
/// The display server is not available in CI, so the test asserts the
/// observable contract the window behaviour rests on: while the first instance
/// holds the lock a second launch is rejected — therefore it exits without
/// opening a second window and the existing instance keeps ownership — and a
/// launch after the first instance exits succeeds.
#[test]
fn e2e_second_launch_is_rejected_while_first_instance_holds_the_lock() {
    let data_base = isolated_dir("lock-data");
    let user_data_dir = dirs::user_data_dir_from(&data_base);
    let lock_path = user_data_dir.join(LOCK_FILE_NAME);

    // Assert: the lock lives in the isolated tree, never the real app data.
    assert!(
        user_data_dir.starts_with(&data_base),
        "lock directory must live under the isolated base, got {:?}",
        user_data_dir
    );

    // Act: the first launch acquires the single-instance lock.
    lock::acquire(&user_data_dir).expect("the first launch must acquire the lock");

    // Assert: the lock file exists and names the owning process.
    assert!(
        lock_path.exists(),
        "the first launch must create the lock file at {:?}",
        lock_path
    );
    assert_eq!(
        fs::read_to_string(&lock_path)
            .expect("the lock file must be readable")
            .trim(),
        std::process::id().to_string(),
        "the lock file must record the owning process id"
    );

    // Act: a second launch tries to acquire the same lock.
    let second_launch = lock::acquire(&user_data_dir);

    // Assert: the second launch is rejected, so it exits instead of opening a
    // second window.
    match second_launch {
        Err(Error::Platform { operation }) => assert!(
            operation.contains("already held"),
            "the rejection must describe the held lock, got {:?}",
            operation
        ),
        other => panic!("a second launch must be rejected, got {:?}", other),
    }

    // Assert: the rejected launch did not take over the lock file, so the
    // existing instance stays the owner (and keeps the user's focus target).
    assert_eq!(
        fs::read_to_string(&lock_path)
            .expect("the lock file must survive a rejected launch")
            .trim(),
        std::process::id().to_string(),
        "a rejected second launch must not rewrite the lock ownership"
    );

    // Act: the first instance exits and releases the lock.
    lock::release(&user_data_dir).expect("releasing the lock must succeed");

    // Assert: the release removes the lock file and frees the next launch.
    assert!(!lock_path.exists(), "releasing must remove the lock file");
    lock::acquire(&user_data_dir).expect("a launch after the first exit must acquire the lock");
    lock::release(&user_data_dir).expect("releasing the lock must succeed");

    // Cleanup: remove the isolated directory tree.
    let _ = fs::remove_dir_all(&data_base);
    assert!(
        !data_base.exists(),
        "the isolated lock directory must be removed after the test"
    );
}

/// End-to-end check for the focus half of the `single-instance-lock`
/// requirement (scenario `second-launch-focuses-existing-instance`).
///
/// No display server is available in CI, so the test asserts the cross-process
/// contract the window behaviour rests on: the second launch is refused the
/// lock, leaves a focus request for the running instance, and therefore exits
/// without opening a second window; the running instance consumes that request
/// exactly once in order to raise its own window, and the documented fallback
/// covers the case where no window exists to raise.
#[test]
fn e2e_second_launch_requests_focus_and_exits_without_a_second_window() {
    let data_base = isolated_dir("focus-data");
    let user_data_dir = dirs::user_data_dir_from(&data_base);
    let lock_path = user_data_dir.join(LOCK_FILE_NAME);
    let focus_path = focus::focus_request_path(&user_data_dir);

    // Act/Assert: the first launch becomes the primary instance, owns the lock
    // and does not inherit a focus request.
    assert_eq!(
        instance::claim(&user_data_dir).expect("the first launch must claim the instance role"),
        LaunchRole::Primary
    );
    assert!(
        lock_path.exists(),
        "the primary instance must hold the lock"
    );
    assert!(
        !focus_path.exists(),
        "a fresh primary launch must not inherit a focus request at {:?}",
        focus_path
    );

    // Act: a second launch starts while the first instance is running.
    let second_launch =
        instance::claim(&user_data_dir).expect("a second launch must be arbitrated");

    // Assert: it is forwarded to the running instance instead of opening a
    // window of its own.
    assert_eq!(
        second_launch,
        LaunchRole::ForwardedToRunningInstance,
        "a second launch must be forwarded to the running instance"
    );
    assert!(
        focus_path.exists(),
        "the second launch must leave a focus request at {:?}",
        focus_path
    );
    assert_eq!(
        fs::read_to_string(&focus_path).expect("the focus request must be readable"),
        std::process::id().to_string(),
        "the focus request must record the forwarding process id"
    );

    // Assert: requesting focus does not disturb the lock ownership, so the
    // running instance stays the target that gets raised.
    assert_eq!(
        fs::read_to_string(&lock_path)
            .expect("the lock file must survive a forwarded launch")
            .trim(),
        std::process::id().to_string(),
        "a forwarded launch must not rewrite the lock ownership"
    );

    // Act: the running instance observes the request.
    let observed = focus::take_focus_request(&user_data_dir)
        .expect("the running instance must read the focus request");

    // Assert: the request is consumed exactly once, so the window is raised
    // once for one second launch.
    assert!(observed, "the running instance must observe the request");
    assert!(
        !focus_path.exists(),
        "consuming the request must remove it from {:?}",
        focus_path
    );
    assert!(
        !focus::take_focus_request(&user_data_dir).expect("a consumed request is not an error"),
        "a consumed focus request must not be raised twice"
    );

    // Assert: the raise-versus-fallback decision is explicit — a window is
    // raised when one exists, and the documented fallback applies when the run
    // is headless and there is no window to raise.
    assert_eq!(focus::decide_focus_action(true), focus::FocusAction::Raise);
    assert_eq!(
        focus::decide_focus_action(false),
        focus::FocusAction::LogFallback
    );

    // Act: the first instance exits, freeing the lock for the next launch.
    lock::release(&user_data_dir).expect("releasing the lock must succeed");
    assert_eq!(
        instance::claim(&user_data_dir).expect("a launch after the first exit must claim"),
        LaunchRole::Primary,
        "a launch after the first instance exits must become primary"
    );
    lock::release(&user_data_dir).expect("releasing the lock must succeed");

    // Cleanup: remove the isolated directory tree.
    let _ = fs::remove_dir_all(&data_base);
    assert!(
        !data_base.exists(),
        "the isolated focus directory must be removed after the test"
    );
}

/// End-to-end check for the `window-geometry-persistence` requirement
/// (scenario `geometry-restored-and-clamped`).
///
/// The persisted geometry is written and restored through the same
/// configuration file location the application resolves, and the restored
/// geometry is clamped to the connected monitors.
#[test]
fn e2e_geometry_is_restored_and_clamped_to_monitors() {
    let data_base = isolated_dir("geometry-data");
    let config_path = config_path_in(&data_base);

    // Assert: the configuration file lives in the isolated tree.
    assert!(
        config_path.starts_with(&data_base),
        "configuration file must live under the isolated base, got {:?}",
        config_path
    );

    // Act/Assert: a first session with no persisted geometry has nothing to
    // restore.
    assert_eq!(
        load_geometry_at(&config_path).expect("a missing config file is not an error"),
        None,
        "a first session must restore no geometry"
    );

    // Act: the previous session persisted its geometry on exit.
    let persisted = Geometry {
        width: 1280.0,
        height: 800.0,
        x: 240.0,
        y: 120.0,
    };
    save_geometry_at(&config_path, persisted).expect("saving geometry must succeed");

    // Assert: the geometry round-trips through the configuration file.
    let restored = load_geometry_at(&config_path)
        .expect("loading geometry must succeed")
        .expect("a persisted geometry must be restored");
    assert_eq!(
        restored, persisted,
        "the restored geometry must match the persisted one"
    );

    // Act: startup clamps the restored geometry to the connected monitor,
    // which is narrower than the restored window.
    let monitor = Some((MONITOR_WIDTH, MONITOR_HEIGHT));
    let clamped = clamp_to_monitor(restored, monitor);

    // Assert: the window fits the monitor and stays on screen.
    assert_eq!(
        clamped.width, MONITOR_WIDTH,
        "width must be clamped to the monitor"
    );
    assert_eq!(
        clamped.height, MONITOR_HEIGHT,
        "height must be clamped to the monitor"
    );
    assert!(clamped.x >= 0.0, "x must stay on screen");
    assert!(clamped.y >= 0.0, "y must stay on screen");
    assert!(
        clamped.x + clamped.width <= MONITOR_WIDTH,
        "the clamped window must fit horizontally"
    );
    assert!(
        clamped.y + clamped.height <= MONITOR_HEIGHT,
        "the clamped window must fit vertically"
    );

    // Act: a window positioned outside the monitor is pulled back on screen.
    let offscreen = Geometry {
        x: 5000.0,
        y: 5000.0,
        ..persisted
    };
    let pulled_back = clamp_to_monitor(offscreen, monitor);

    // Assert: the off-screen position is bounded by the monitor edges.
    assert_eq!(
        pulled_back.x,
        MONITOR_WIDTH - pulled_back.width,
        "an off-screen x position must be pulled back inside the monitor"
    );
    assert_eq!(
        pulled_back.y,
        MONITOR_HEIGHT - pulled_back.height,
        "an off-screen y position must be pulled back inside the monitor"
    );
    assert!(
        pulled_back.x >= 0.0 && pulled_back.y >= 0.0,
        "the pulled-back window must be fully on screen"
    );

    // Assert: an unknown monitor size leaves the geometry untouched.
    assert_eq!(
        clamp_to_monitor(restored, None),
        restored,
        "geometry must be restored unchanged when the monitor size is unknown"
    );

    // Cleanup: remove the isolated directory tree and the persisted file.
    let _ = fs::remove_dir_all(&data_base);
    assert!(
        !config_path.exists(),
        "the isolated configuration file must be removed after the test"
    );
}

/// End-to-end check for the `logging-and-panic-hook` requirement
/// (scenario `logging-and-panic-hook-wired`).
///
/// Logging is initialized for an isolated user data directory, a structured
/// event and an unhandled panic are recorded in the rolling log file, and the
/// written content carries no credential or token.
#[test]
fn e2e_logging_and_panic_hook_write_to_the_rolling_log_without_secrets() {
    let data_base = isolated_dir("logging-data");
    let log_dir = dirs::user_data_dir_from(&data_base).join("logs");
    let log_file = log_dir.join("gitea-stopwatch.log");

    // Act: the process edge initializes logging against the isolated data dir.
    logging::init(&log_dir).expect("logging must initialize in an isolated data dir");

    // Act: the application records a structured startup event through the
    // `tracing` facade, which the installed subscriber renders to the file.
    tracing::info!(target: "e2e", phase = "startup", "shell started");

    // Act: an unhandled panic is raised; the installed hook must record it and
    // the unwind must be caught so the test process survives.
    let panicked = std::panic::catch_unwind(|| panic!("unhandled failure for e2e coverage"));

    // Assert: the panic reached the hook instead of aborting the process.
    assert!(panicked.is_err(), "the panicking closure must unwind");

    // Assert: the rolling log file is written under the isolated data dir.
    assert!(
        log_file.starts_with(&data_base),
        "the log file must live under the isolated base, got {:?}",
        log_file
    );
    assert!(
        log_file.exists(),
        "the rolling log file must exist at {:?}",
        log_file
    );
    let contents = fs::read_to_string(&log_file).expect("the rolling log file must be readable");

    // Assert: both the structured event and the panic record are present.
    assert!(
        contents.contains("shell started"),
        "the structured event must be logged, got:\n{}",
        contents
    );
    assert!(
        contents.contains("phase=\"startup\""),
        "structured fields must be logged as key=\"value\" pairs"
    );
    assert!(
        contents.contains("panic occurred at"),
        "the panic hook must record the panic, got:\n{}",
        contents
    );
    assert!(
        contents.contains("unhandled failure for e2e coverage"),
        "the panic hook must record the panic payload"
    );

    // Assert: no credential-shaped field name or secret literal was written.
    let lowered = contents.to_lowercase();
    for field in FORBIDDEN_LOG_FIELDS {
        assert!(
            !lowered.contains(&format!(" {}=\"", field)),
            "log content must not contain a {:?} field, got:\n{}",
            field,
            contents
        );
    }
    for marker in FORBIDDEN_LOG_MARKERS {
        assert!(
            !lowered.contains(marker),
            "log content must not contain the secret marker {:?}, got:\n{}",
            marker,
            contents
        );
    }

    // Cleanup: remove the isolated directory tree and the rolling log file.
    let _ = fs::remove_dir_all(&data_base);
    assert!(
        !log_file.exists(),
        "the isolated log file must be removed after the test"
    );
}

// ---------------------------------------------------------------------------
// Shipped-binary execution
// ---------------------------------------------------------------------------

/// Startup line the binary edge records before the Iced shell runs.
const STARTUP_LINE: &str = "gitea-stopwatch starting";

/// Delay, in milliseconds, after which the harness closes the window.
///
/// Long enough for the X server to map and expose the window on a slow CI
/// runner, short enough to keep the suite fast.
const AUTO_CLOSE_DELAY_MS: u64 = 6000;

/// How long the test waits for the binary to record its startup line.
const STARTUP_TIMEOUT: Duration = Duration::from_secs(15);

/// How long the test waits for the X server to expose the window.
const WINDOW_TIMEOUT: Duration = Duration::from_secs(9);

/// How long the test waits for the window close and the process exit.
const EXIT_TIMEOUT: Duration = Duration::from_secs(30);

/// Poll interval for the rolling log, the X server and the process status.
const POLL_INTERVAL: Duration = Duration::from_millis(100);

/// How the test reaches a display server to execute the shipped binary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DisplayHarness {
    /// `DISPLAY` is set and reachable: the binary runs directly on it and the
    /// X server can be introspected through `xwininfo`/`xprop`.
    Display,
    /// No `DISPLAY`, but `xvfb-run` exists: the binary runs under a private
    /// headless X server. The window cannot be introspected from here because
    /// `xvfb-run` allocates the display number of its own process.
    XvfbRun,
}

/// Resolve how (or whether) the Iced shell can be executed here.
///
/// Returns `None` when neither a reachable `DISPLAY` nor `xvfb-run` is
/// available, in which case the caller skips with a documented reason instead
/// of failing.
fn detect_display_harness() -> Option<DisplayHarness> {
    let display = env::var_os("DISPLAY").filter(|value| !value.is_empty());

    if display.is_some() {
        // A `DISPLAY` that is set but unreachable is an environment limitation
        // (stale session variable), not a product failure. Probe it when the
        // X11 client tool is available and skip when the probe fails.
        if command_on_path("xwininfo") && !x11_root_reachable() {
            return None;
        }
        return Some(DisplayHarness::Display);
    }

    if command_on_path("xvfb-run") {
        return Some(DisplayHarness::XvfbRun);
    }

    None
}

/// Whether an executable named `program` is reachable through `PATH`.
fn command_on_path(program: &str) -> bool {
    env::var_os("PATH")
        .map(|paths| env::split_paths(&paths).any(|dir| dir.join(program).is_file()))
        .unwrap_or(false)
}

/// Whether the X server named by `DISPLAY` accepts connections.
fn x11_root_reachable() -> bool {
    Command::new("xwininfo")
        .arg("-root")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

/// An isolated directory tree removed when the test finishes, including on
/// panic, so no scratch data survives a failed run.
struct Scratch {
    root: PathBuf,
}

impl Scratch {
    fn new(suffix: &str) -> Self {
        let root = isolated_dir(suffix);
        fs::create_dir_all(&root).expect("the scratch directory must be created");
        Self { root }
    }

    fn root(&self) -> &Path {
        &self.root
    }

    fn data_dir(&self) -> PathBuf {
        self.root.join("data")
    }

    fn cache_dir(&self) -> PathBuf {
        self.root.join("cache")
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// Owns the spawned shipped binary, killing it if the test panics before the
/// process exits, so a failing assertion never leaves a process behind.
struct ChildGuard {
    child: Option<Child>,
}

impl ChildGuard {
    fn new(child: Child) -> Self {
        Self { child: Some(child) }
    }

    fn id(&self) -> u32 {
        self.child
            .as_ref()
            .expect("the binary process is owned by the guard")
            .id()
    }

    fn child(&mut self) -> &mut Child {
        self.child
            .as_mut()
            .expect("the binary process is owned by the guard")
    }

    fn has_exited(&mut self) -> bool {
        self.child()
            .try_wait()
            .expect("the process status must be readable")
            .is_some()
    }

    /// Wait for the process exit, returning `None` when the deadline passes.
    fn wait_for_exit(&mut self, timeout: Duration) -> Option<ExitStatus> {
        let deadline = Instant::now() + timeout;
        loop {
            if let Some(status) = self
                .child()
                .try_wait()
                .expect("the process status must be readable")
            {
                return Some(status);
            }
            if Instant::now() >= deadline {
                return None;
            }
            thread::sleep(POLL_INTERVAL);
        }
    }

    /// Wait until the rolling log carries `needle`, returning what was read.
    fn wait_for_log_line(&mut self, log_file: &Path, needle: &str, timeout: Duration) -> String {
        let deadline = Instant::now() + timeout;
        loop {
            let contents = fs::read_to_string(log_file).unwrap_or_default();
            if contents.contains(needle) || self.has_exited() || Instant::now() >= deadline {
                return contents;
            }
            thread::sleep(POLL_INTERVAL);
        }
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// Launch the shipped binary against an isolated user-data directory.
///
/// `CARGO_BIN_EXE_gitea-stopwatch` is the executable Cargo built for this
/// package, so the test runs the same artifact the application ships rather
/// than an in-process approximation of it.
fn spawn_shipped_binary(
    harness: DisplayHarness,
    scratch: &Scratch,
    data_dir: &Path,
    cache_dir: &Path,
) -> Child {
    let binary = env!("CARGO_BIN_EXE_gitea-stopwatch");

    let mut command = match harness {
        DisplayHarness::Display => Command::new(binary),
        DisplayHarness::XvfbRun => {
            let mut command = Command::new("xvfb-run");
            // A 24-bit screen, because the renderer cannot create a surface on
            // the 8-bit screen some `xvfb-run` builds default to.
            command
                .arg("-a")
                .arg("--server-args=-screen 0 1280x1024x24")
                .arg(binary);
            command
        }
    };

    let stderr = fs::File::create(scratch.root().join("stderr.log"))
        .expect("the stderr capture file must be created");

    command
        .env("GITEA_STOPWATCH_DATA_DIR", data_dir)
        .env("GITEA_STOPWATCH_CACHE_DIR", cache_dir)
        // Harness: close the window through the real close path after the
        // delay, so clean shutdown is exercised instead of a forced kill.
        .env(AUTO_CLOSE_ENV, AUTO_CLOSE_DELAY_MS.to_string())
        // Debug verbosity makes the rolling log record the shell boot path
        // (window creation) in addition to the startup line.
        .env("GITEA_STOPWATCH_LOG", "debug")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::from(stderr))
        .spawn()
        .expect("the shipped binary must be launched")
}

/// Find an X11 window carrying `title` and owned by `pid`.
///
/// The pid check matters: the developer may already have the application
/// running, and the title alone would then let the test observe a window this
/// launch never opened.
fn find_window(pid: u32, title: &str) -> Option<String> {
    let output = Command::new("xwininfo")
        .args(["-root", "-tree"])
        .output()
        .ok()?;
    let tree = String::from_utf8_lossy(&output.stdout);
    let wanted = format!("\"{title}\"");

    for line in tree.lines() {
        if !line.contains(&wanted) {
            continue;
        }
        let Some(id) = line.split_whitespace().next() else {
            continue;
        };
        if !id.starts_with("0x") {
            continue;
        }
        if let Some(owner) = window_owner(id)
            && owner != pid
        {
            continue;
        }
        return Some(id.to_string());
    }

    None
}

/// Read the `_NET_WM_PID` of an X11 window, when the server exposes it.
fn window_owner(window: &str) -> Option<u32> {
    let output = Command::new("xprop")
        .args(["-id", window, "_NET_WM_PID"])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    text.split_once('=')?.1.trim().parse().ok()
}

/// Read the `xwininfo -id` report of an X11 window.
fn window_stats(window: &str) -> Option<String> {
    let output = Command::new("xwininfo")
        .args(["-id", window])
        .output()
        .ok()?;
    Some(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Extract a `field: value` integer from a `xwininfo` report.
fn window_dimension(stats: &str, field: &str) -> Option<u32> {
    stats.lines().find_map(|line| {
        let value = line.trim().strip_prefix(field)?.trim();
        value.parse().ok()
    })
}

/// Wait until the X server exposes the mapped window this launch opened.
///
/// A freshly created window is unmapped until the shell presents its first
/// frame, so the wait continues until the window is both present and viewable.
/// The last observation is returned even when the deadline passes, so the
/// failure message shows what the server actually reported.
fn wait_for_window(
    child: &mut ChildGuard,
    pid: u32,
    title: &str,
    timeout: Duration,
) -> Option<(String, String)> {
    let deadline = Instant::now() + timeout;
    let mut last: Option<(String, String)> = None;
    loop {
        if let Some(window) = find_window(pid, title)
            && let Some(stats) = window_stats(&window)
        {
            if stats.contains("Map State: IsViewable") {
                return Some((window, stats));
            }
            last = Some((window, stats));
        }
        if child.has_exited() || Instant::now() >= deadline {
            return last;
        }
        thread::sleep(POLL_INTERVAL);
    }
}

/// The window title, theme and placeholder navigation the shell is configured
/// with, observed through the crate's public surface.
///
/// The rendered pixels cannot be read back from a headless run, so the content
/// assertions that no display server can expose live here; the display test
/// below proves the window itself opens, is titled and closes.
#[test]
fn e2e_window_title_theme_and_placeholder_navigation_are_configured() {
    // Title: the exact string the application sets on the window, which the
    // display test below then observes on the X server.
    assert_eq!(APP_TITLE, "gitea-stopwatch", "the window title must be set");

    // Theme: the application is wired to the light theme (`src/lib.rs` passes
    // `ui::window::theme` to the application builder, and `theme` returns this
    // constant — asserted by the in-crate unit tests).
    assert_eq!(
        WINDOW_THEME,
        iced::Theme::Light,
        "the window theme must be configured"
    );

    // Placeholder navigation: both rendered lines carry content, and the
    // navigation area is reserved for the future main window.
    assert!(
        !PLACEHOLDER_NAV_TITLE.is_empty(),
        "the placeholder navigation heading must be rendered"
    );
    assert!(
        !PLACEHOLDER_NAV_HINT.is_empty(),
        "the placeholder navigation hint must be rendered"
    );
    assert!(
        PLACEHOLDER_NAV_HINT.to_lowercase().contains("reserved"),
        "the placeholder navigation must state that it is reserved, got {PLACEHOLDER_NAV_HINT:?}"
    );
}

/// End-to-end execution of the shipped binary.
///
/// This is the test that proves the shell scenarios by running the application
/// rather than by driving the crate's public surface:
///
/// - `empty-main-window` / `window-opens-with-title-and-theme`: the binary
///   boots, the X server exposes a mapped window carrying [`APP_TITLE`], and
///   the pid that owns it is this launch.
/// - `clean-shutdown`: the window is closed through the same close path the
///   window button takes, and the process then exits `0`, releases the
///   single-instance lock and persists the window geometry.
/// - `placeholder-navigation` / `placeholder-navigation-renders`: the mapped
///   window is the render of the shell view (whose content is asserted by
///   [`e2e_window_title_theme_and_placeholder_navigation_are_configured`],
///   because rendered pixels cannot be read back headlessly).
///
/// The test skips with a printed reason when no display server is available,
/// so a headless CI runner without Xvfb stays green; see
/// `docs/standards-gaps.md` for the documented limitation.
#[test]
fn e2e_shipped_binary_opens_a_window_and_shuts_down_cleanly() {
    let Some(harness) = detect_display_harness() else {
        println!(
            "SKIP e2e_shipped_binary_opens_a_window_and_shuts_down_cleanly: no reachable DISPLAY \
             and no xvfb-run on this runner, so the Iced shell cannot be executed headlessly. \
             This is a documented limitation (docs/standards-gaps.md), not a product failure."
        );
        return;
    };

    let scratch = Scratch::new("binary");
    let data_dir = scratch.data_dir();
    let cache_dir = scratch.cache_dir();
    let user_data_dir = dirs::user_data_dir_from(&data_dir);
    let log_file = user_data_dir.join("logs").join("gitea-stopwatch.log");
    let config_file = config_path_in(&data_dir);
    let lock_file = user_data_dir.join(LOCK_FILE_NAME);
    let stderr_file = scratch.root().join("stderr.log");

    // Assert: the run is isolated from the developer's real application data.
    assert!(
        log_file.starts_with(&data_dir) && config_file.starts_with(&data_dir),
        "the shipped binary must be launched against the isolated data dir"
    );

    let mut child = ChildGuard::new(spawn_shipped_binary(
        harness, &scratch, &data_dir, &cache_dir,
    ));
    let pid = child.id();

    // Act/Assert: the shipped binary executes its startup path and records it
    // in the isolated rolling log, proving the process edge ran.
    let contents = child.wait_for_log_line(&log_file, STARTUP_LINE, STARTUP_TIMEOUT);
    assert!(
        contents.contains(STARTUP_LINE),
        "the shipped binary must record its startup line {STARTUP_LINE:?}, got:\n{contents}\n{}",
        read_stderr(&stderr_file)
    );

    // Act/Assert: the X server exposes the mapped window of this launch.
    let introspect = harness == DisplayHarness::Display && command_on_path("xwininfo");
    if introspect {
        let (window, stats) = wait_for_window(&mut child, pid, APP_TITLE, WINDOW_TIMEOUT)
            .unwrap_or_else(|| {
                panic!(
                    "no X11 window titled {APP_TITLE:?} owned by pid {pid} appeared within \
                     {WINDOW_TIMEOUT:?}; log:\n{}\n{}",
                    fs::read_to_string(&log_file).unwrap_or_default(),
                    read_stderr(&stderr_file)
                )
            });

        assert!(
            stats.contains("Map State: IsViewable"),
            "the window {window} must be mapped and visible, got:\n{stats}"
        );
        assert!(
            window_dimension(&stats, "Width:").is_some_and(|width| width > 0),
            "the window {window} must have a width, got:\n{stats}"
        );
        assert!(
            window_dimension(&stats, "Height:").is_some_and(|height| height > 0),
            "the window {window} must have a height, got:\n{stats}"
        );
    } else {
        println!(
            "NOTE e2e_shipped_binary_opens_a_window_and_shuts_down_cleanly: window introspection \
             unavailable ({harness:?}, xwininfo present: {}); the window is proven by the \
             successful clean exit that follows, not by inspecting the X server",
            command_on_path("xwininfo")
        );
    }

    // Act: the harness closes the window through the application's own close
    // path, and the process exits on its own.
    let status = child.wait_for_exit(EXIT_TIMEOUT).unwrap_or_else(|| {
        panic!(
            "the shipped binary did not exit within {EXIT_TIMEOUT:?} after the auto-close \
             deadline; log:\n{}\n{}",
            fs::read_to_string(&log_file).unwrap_or_default(),
            read_stderr(&stderr_file)
        )
    });

    // Assert: the close path ended the process successfully, with no forced
    // kill and no error exit.
    assert!(
        status.success(),
        "closing the window must end the process successfully, got {status:?}; log:\n{}\n{}",
        fs::read_to_string(&log_file).unwrap_or_default(),
        read_stderr(&stderr_file)
    );

    // Assert: clean shutdown released the single-instance lock, so the next
    // launch starts as the primary instance.
    assert!(
        !lock_file.exists(),
        "the lock file {lock_file:?} must be released on shutdown"
    );

    // Assert: the close path persisted the window geometry, which only happens
    // when the application handles the close request itself.
    let geometry = load_geometry_at(&config_file)
        .expect("the persisted geometry must be readable")
        .expect("the close path must persist the window geometry");
    assert!(
        geometry.width > 0.0 && geometry.height > 0.0,
        "the persisted geometry must describe a real window, got {geometry:?}"
    );

    // Assert: the run recorded no panic, so shutdown was clean rather than a
    // crash that happened to exit successfully.
    let contents = fs::read_to_string(&log_file).expect("the rolling log must be readable");
    assert!(
        !contents.contains("panic occurred at"),
        "the run must not panic, got:\n{contents}"
    );
}

/// Read the captured stderr of a failed run for the panic message.
fn read_stderr(path: &Path) -> String {
    match fs::read_to_string(path) {
        Ok(text) if !text.trim().is_empty() => format!("stderr:\n{text}"),
        _ => "stderr: <empty>".to_string(),
    }
}
