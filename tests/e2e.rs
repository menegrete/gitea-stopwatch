//! End-to-end tests for the gitea-stopwatch desktop shell.
//!
//! These tests run against an isolated user-data directory using the
//! `GITEA_STOPWATCH_DATA_DIR` and `GITEA_STOPWATCH_CACHE_DIR` environment
//! overrides (see `src/platform/dirs.rs`), so they never touch the
//! developer's real application data. Each test cleans up its isolated
//! directories and restores the environment after execution.
//!
//! The shell behaviour tests drive the crate's public surface with explicit
//! isolated paths (`dirs::user_data_dir_from`, `core::config_path_in`,
//! `core::load_geometry_at`, `core::save_geometry_at`, `platform::lock`) so
//! they run headless in CI without a display server:
//!
//! - `single-instance-lock` → `second-launch-focuses-existing-instance`
//! - `window-geometry-persistence` → `geometry-restored-and-clamped`
//! - `logging-and-panic-hook` → `logging-and-panic-hook-wired`

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use gitea_stopwatch::core::{
    Error, Geometry, clamp_to_monitor, config_path_in, load_geometry_at, save_geometry_at,
};
use gitea_stopwatch::platform::instance::LaunchRole;
use gitea_stopwatch::platform::{dirs, focus, instance, lock, logging};
use gitea_stopwatch::ui::window::APP_TITLE;

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

/// Point the app's env overrides at isolated directories, returning the
/// previous values so the caller can restore them.
fn override_env(data_dir: &Path, cache_dir: &Path) -> (Option<String>, Option<String>) {
    let prev_data = env::var("GITEA_STOPWATCH_DATA_DIR").ok();
    let prev_cache = env::var("GITEA_STOPWATCH_CACHE_DIR").ok();
    unsafe {
        env::set_var("GITEA_STOPWATCH_DATA_DIR", data_dir);
        env::set_var("GITEA_STOPWATCH_CACHE_DIR", cache_dir);
    }
    (prev_data, prev_cache)
}

/// Restore previously saved env override values (or remove them).
fn restore_env(prev: (Option<String>, Option<String>)) {
    match prev.0 {
        Some(v) => unsafe {
            env::set_var("GITEA_STOPWATCH_DATA_DIR", v);
        },
        None => unsafe {
            env::remove_var("GITEA_STOPWATCH_DATA_DIR");
        },
    }
    match prev.1 {
        Some(v) => unsafe {
            env::set_var("GITEA_STOPWATCH_CACHE_DIR", v);
        },
        None => unsafe {
            env::remove_var("GITEA_STOPWATCH_CACHE_DIR");
        },
    }
}

#[test]
fn e2e_uses_isolated_user_data_and_cleans_up() {
    let data_dir = isolated_dir("data");
    let cache_dir = isolated_dir("cache");

    // Arrange: isolated env overrides
    let prev = override_env(&data_dir, &cache_dir);

    // Act: resolve user data and cache directories through the app's platform
    // abstraction, which honours the env overrides.
    let resolved_data = dirs::user_data_dir().expect("user data dir should resolve");
    let resolved_cache = dirs::cache_dir().expect("cache dir should resolve");

    // Assert: directories live under the isolated temp locations, never the
    // real application data directory.
    assert!(
        resolved_data.starts_with(&data_dir),
        "user data dir should be under isolated data dir, got {:?}",
        resolved_data
    );
    assert!(
        resolved_data.ends_with("gitea-stopwatch"),
        "user data dir should be namespaced by app name"
    );
    assert!(
        resolved_cache.starts_with(&cache_dir),
        "cache dir should be under isolated cache dir, got {:?}",
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

    // Cleanup: remove isolated directories and restore the environment.
    restore_env(prev);
    let _ = fs::remove_dir_all(&data_dir);
    let _ = fs::remove_dir_all(&cache_dir);

    // Assert: env overrides are restored to their previous state.
    assert!(
        env::var("GITEA_STOPWATCH_DATA_DIR").is_err()
            || env::var("GITEA_STOPWATCH_DATA_DIR").unwrap() != data_dir.to_string_lossy(),
        "GITEA_STOPWATCH_DATA_DIR should not point to the isolated dir after cleanup"
    );
    assert!(
        env::var("GITEA_STOPWATCH_CACHE_DIR").is_err()
            || env::var("GITEA_STOPWATCH_CACHE_DIR").unwrap() != cache_dir.to_string_lossy(),
        "GITEA_STOPWATCH_CACHE_DIR should not point to the isolated dir after cleanup"
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

    // Act: the application records a structured startup event.
    logging::log(
        logging::Level::Info,
        "e2e",
        &[("phase", "startup")],
        "shell started",
    );

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
