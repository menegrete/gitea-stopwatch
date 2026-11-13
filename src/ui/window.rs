//! Iced view layer: the empty main window shell.
//!
//! Provides an empty window with title, theme, and placeholder navigation
//! reserved for the future main window (spec: placeholder-navigation).
//! The window closes cleanly when the user closes it, leaving no residual
//! processes (spec: clean-shutdown). Window geometry is persisted and
//! restored (spec: window-geometry-persistence).
//!
//! The window also honors the focus half of the single-instance contract
//! (spec: single-instance-lock, scenario
//! `second-launch-focuses-existing-instance`): a second launch cannot open a
//! window, so it leaves a focus request that this instance polls for and
//! answers by raising its own window through the platform windowing system.

use std::path::PathBuf;
use std::time::Duration;

use iced::widget::{column, container, text};
use iced::{Alignment, Element, Length, Subscription, Task, Theme, time, window};

use crate::core::{Geometry, clamp_to_monitor, load_geometry, save_geometry};
use crate::platform::focus::{self, FocusAction};

/// Application title shown in the window decoration.
pub const APP_TITLE: &str = "gitea-stopwatch";

/// Placeholder navigation heading rendered in the empty main window.
///
/// The navigation area is reserved for the future main window and carries no
/// business functionality (spec: placeholder-navigation).
pub const PLACEHOLDER_NAV_TITLE: &str = "Placeholder navigation";

/// Supporting line rendered under the placeholder navigation heading.
pub const PLACEHOLDER_NAV_HINT: &str = "Reserved for the future main window.";

/// Theme configured for the main window (spec: empty-main-window).
pub const WINDOW_THEME: Theme = Theme::Light;

/// Test harness variable: delay in milliseconds after which the running
/// instance closes its own window.
///
/// A headless end-to-end run cannot press the window close button, so the
/// suite sets this variable to drive the *real* close path in the shipped
/// binary — `CloseRequested` → geometry persisted → window closed → process
/// exits `0` — instead of killing the process. When the variable is unset
/// (every normal run) the harness is inert.
pub const AUTO_CLOSE_ENV: &str = "GITEA_STOPWATCH_E2E_AUTO_CLOSE_MS";

/// Parse the auto-close harness delay.
///
/// A missing, non-numeric or zero value leaves the harness disarmed, so a
/// malformed value can never turn a normal run into a self-closing one.
fn parse_auto_close(raw: Option<&str>) -> Option<Duration> {
    let millis: u64 = raw?.trim().parse().ok()?;
    if millis == 0 {
        return None;
    }
    Some(Duration::from_millis(millis))
}

/// The configured auto-close delay, if the harness variable is set.
fn auto_close_delay() -> Option<Duration> {
    parse_auto_close(std::env::var(AUTO_CLOSE_ENV).ok().as_deref())
}

/// How often the running instance looks for a focus request from a second
/// launch.
///
/// The channel is a marker file in the user data directory, so polling is a
/// cheap filesystem check that keeps the second launch free of any IPC
/// handshake it would have to block on.
pub const FOCUS_POLL_INTERVAL: Duration = Duration::from_millis(400);

/// What the running instance did with the most recent focus request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusStatus {
    /// No focus request has been observed since the window opened.
    #[default]
    Idle,
    /// A raise was handed to the platform windowing system.
    Raised,
    /// A request arrived but there was no window to raise, so the documented
    /// fallback applied: the request was consumed and a warning was logged.
    Unsupported,
}

/// The empty main window application state.
#[derive(Debug)]
pub struct MainWindow {
    /// Current window geometry (size and position).
    geometry: Geometry,
    /// Geometry restored from the previous session, held until the runtime
    /// reports the monitor size so it is clamped before it reaches the window.
    ///
    /// `None` once the restore completed, when it could not be resolved, or
    /// when there was nothing to restore at all.
    pending_restore: Option<Geometry>,
    /// Whether the window close has been requested.
    closing: bool,
    /// User data directory holding the single-instance focus request channel.
    user_data_dir: PathBuf,
    /// Outcome of the most recent focus request, kept for observability.
    focus: FocusStatus,
}

impl MainWindow {
    fn new(user_data_dir: PathBuf, geometry: Geometry) -> Self {
        Self {
            geometry,
            pending_restore: None,
            closing: false,
            user_data_dir,
            focus: FocusStatus::Idle,
        }
    }

    /// State carrying a restored geometry that still awaits the monitor report.
    fn restoring(user_data_dir: PathBuf, restored: Geometry) -> Self {
        Self {
            pending_restore: Some(restored),
            ..Self::new(user_data_dir, restored)
        }
    }
}

/// Messages produced by the main window.
#[derive(Debug, Clone)]
pub enum Message {
    /// The user requested to close the window, carrying the window id.
    CloseRequested {
        /// Identifier of the window the close request applies to.
        id: window::Id,
    },
    /// A placeholder navigation item was activated (no-op for now).
    PlaceholderNavActivated,
    /// The window was resized, carrying the new size.
    Resized {
        /// New logical size of the window.
        size: iced::Size,
    },
    /// The window was moved, carrying the new position.
    Moved {
        /// New logical position of the window.
        position: iced::Point,
    },
    /// The runtime resolved which window, if any, carries the restored
    /// geometry.
    WindowResolved(Option<window::Id>),
    /// The runtime reported the logical size of the monitor containing the
    /// window, if the platform can report one.
    MonitorResolved {
        /// Identifier of the window the monitor report applies to.
        id: window::Id,
        /// Logical size of the containing monitor, when the platform reports it.
        size: Option<iced::Size>,
    },
    /// Timer tick: look for a focus request left by a second launch.
    PollFocusRequest,
    /// The runtime resolved which window, if any, answers a focus request.
    FocusTargetResolved(Option<window::Id>),
    /// Harness tick: the configured auto-close delay elapsed.
    AutoCloseRequested,
    /// The runtime resolved which window the harness closes.
    AutoCloseTargetResolved(Option<window::Id>),
}

/// Bounds usable for clamping, from a runtime-reported monitor size.
///
/// Iced exposes the logical size of the monitor containing the window through
/// `iced::window::monitor_size(id)` — and only once the window exists. A
/// platform that cannot answer (headless session, or a Wayland session without
/// monitor geometry) reports `None`, and a zero-sized report carries no usable
/// information either; both degrade to "no clamping" rather than pinning the
/// window to a degenerate rectangle.
///
/// Iced 0.14 reports no monitor origin, so these bounds are measured from the
/// windowing system's origin rather than from the monitor's own top-left
/// corner. A geometry restored onto a secondary monitor is therefore clamped
/// against the monitor size but not against that monitor's offset; origin-aware
/// multi-monitor clamping is deferred until iced exposes monitor bounds.
fn monitor_bounds(monitor: Option<iced::Size>) -> Option<(f64, f64)> {
    let size = monitor?;
    let bounds = (f64::from(size.width), f64::from(size.height));

    if bounds.0 > 0.0 && bounds.1 > 0.0 {
        Some(bounds)
    } else {
        None
    }
}

/// The geometry to apply for a runtime-reported monitor.
///
/// Pure wiring decision: it turns the runtime's monitor report into the
/// `clamp_to_monitor` bounds, so the clamping itself stays a pure core
/// function and the wiring is unit-testable without a display server.
fn geometry_for_monitor(restored: Geometry, monitor: Option<iced::Size>) -> Geometry {
    clamp_to_monitor(restored, monitor_bounds(monitor))
}

/// Boot function: creates the initial state and the geometry-restore task.
///
/// The user data directory is injected by the library edge so the window and
/// the single-instance lock observe the same directory.
///
/// The monitor size cannot be observed before the window exists, so a restored
/// geometry is not clamped here: boot asks the runtime for the window id and
/// the clamp is applied as soon as the monitor is reported, then the clamped
/// geometry is pushed to the real window (spec:
/// window-geometry-persistence, scenario `geometry-restored-and-clamped`).
pub fn boot(user_data_dir: PathBuf) -> (MainWindow, Task<Message>) {
    match load_geometry().ok().flatten() {
        Some(geometry) => (
            MainWindow::restoring(user_data_dir, geometry),
            // Chained after the window opens, so the id resolves to the real
            // window; the runtime runs this task once the window exists.
            window::latest().map(Message::WindowResolved),
        ),
        // First session: nothing to restore, so nothing to clamp.
        None => (
            MainWindow::new(user_data_dir, Geometry::default()),
            Task::none(),
        ),
    }
}

/// Update function: processes messages and returns a task.
pub fn update(state: &mut MainWindow, message: Message) -> Task<Message> {
    match message {
        Message::CloseRequested { id } => {
            state.closing = true;
            // Persist geometry before closing.
            let _ = save_geometry(state.geometry);
            // Close the requested window; Iced exits when all windows are
            // closed, giving a clean shutdown with no residual processes.
            window::close(id)
        }
        Message::Resized { size } => {
            // While a restore is in flight the window still carries the size
            // the runtime opened it with, so the event must not clobber the
            // geometry being restored. Once the restore is applied, events are
            // the live geometry that the close path persists.
            if state.pending_restore.is_none() {
                state.geometry.width = f64::from(size.width);
                state.geometry.height = f64::from(size.height);
            }
            Task::none()
        }
        Message::Moved { position } => {
            if state.pending_restore.is_none() {
                state.geometry.x = f64::from(position.x);
                state.geometry.y = f64::from(position.y);
            }
            Task::none()
        }
        Message::WindowResolved(target) => match target {
            Some(id) => {
                // The monitor containing the window is observable only once the
                // window exists, which is why the clamp waits for this answer.
                window::monitor_size(id).map(move |size| Message::MonitorResolved { id, size })
            }
            None => {
                // Nothing to clamp against and nothing to resize: the restored
                // geometry is kept as loaded (documented no-monitor fallback).
                tracing::warn!(
                    "no window answered the geometry restore; restoring without a monitor clamp"
                );
                state.pending_restore = None;
                Task::none()
            }
        },
        Message::MonitorResolved { id, size } => {
            let Some(restored) = state.pending_restore.take() else {
                // No restore is in flight; a late report changes nothing.
                return Task::none();
            };

            let clamped = geometry_for_monitor(restored, size);
            state.geometry = clamped;

            if size.is_none() {
                tracing::warn!(
                    "the runtime reported no monitor; the restored geometry is not clamped"
                );
            }

            // Push the clamped geometry to the real window, so what the user
            // sees is exactly what the close path persists.
            window::resize(
                id,
                iced::Size::new(clamped.width as f32, clamped.height as f32),
            )
            .chain(window::move_to(
                id,
                iced::Point::new(clamped.x as f32, clamped.y as f32),
            ))
        }
        Message::PlaceholderNavActivated => Task::none(),
        Message::PollFocusRequest => match focus::take_focus_request(&state.user_data_dir) {
            Ok(true) => {
                tracing::info!("focus request received from a second launch");
                // Ask the runtime which window answers the request; the answer
                // arrives as `FocusTargetResolved` and decides raise vs fallback.
                window::latest().map(Message::FocusTargetResolved)
            }
            Ok(false) => Task::none(),
            Err(error) => {
                tracing::warn!(
                    error = error.to_string(),
                    "focus request could not be consumed"
                );
                Task::none()
            }
        },
        Message::FocusTargetResolved(target) => {
            match focus::decide_focus_action(target.is_some()) {
                FocusAction::Raise => {
                    state.focus = FocusStatus::Raised;
                    tracing::info!("raising the existing window for a second launch");
                    // A minimized window ignores a focus request, so restore it
                    // first and only then hand the raise to the platform.
                    target
                        .map(|id| {
                            window::set_mode(id, window::Mode::Windowed)
                                .chain(window::gain_focus(id))
                        })
                        .unwrap_or_else(Task::none)
                }
                FocusAction::LogFallback => {
                    state.focus = FocusStatus::Unsupported;
                    // Documented fallback: no window can be raised here (headless
                    // run or pre-window request), so the instance keeps running and
                    // records why the request could not be honored.
                    tracing::warn!(
                        "focus requested; raising the window is unsupported on this platform"
                    );
                    Task::none()
                }
            }
        }
        Message::AutoCloseRequested => window::latest().map(Message::AutoCloseTargetResolved),
        Message::AutoCloseTargetResolved(target) => match target {
            Some(id) => {
                tracing::info!("e2e harness: closing the window after the configured delay");
                // Reuse the exact path the window close button takes, so the
                // harness proves clean shutdown rather than a forced exit.
                update(state, Message::CloseRequested { id })
            }
            None => {
                tracing::warn!("e2e harness: auto-close requested but no window is open");
                Task::none()
            }
        },
    }
}

/// Theme configured for the main window (spec: empty-main-window).
pub fn theme(_state: &MainWindow) -> Theme {
    WINDOW_THEME
}

/// View function: renders the empty main window with placeholder navigation.
pub fn view(_state: &MainWindow) -> Element<'_, Message> {
    let nav = column![
        text(PLACEHOLDER_NAV_TITLE).size(18),
        text(PLACEHOLDER_NAV_HINT).size(14),
    ]
    .spacing(10)
    .align_x(Alignment::Center);

    column![
        text(APP_TITLE).size(24),
        container(nav)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    ]
    .spacing(20)
    .align_x(Alignment::Center)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// Subscription function: listens for window close requests, geometry changes
/// and focus requests from a second launch.
pub fn subscription(state: &MainWindow) -> Subscription<Message> {
    use iced::window::Event;

    let window_events = window::events().filter_map(|(id, event)| match event {
        Event::CloseRequested => Some(Message::CloseRequested { id }),
        Event::Resized(size) => Some(Message::Resized { size }),
        Event::Moved(position) => Some(Message::Moved { position }),
        _ => None,
    });

    if state.closing {
        // No focus request needs answering once the window is closing.
        return window_events;
    }

    let focus_requests = time::every(FOCUS_POLL_INTERVAL).map(|_| Message::PollFocusRequest);

    match auto_close_delay() {
        // The harness is armed: close once after the configured delay.
        Some(delay) => Subscription::batch([
            window_events,
            focus_requests,
            time::every(delay).map(|_| Message::AutoCloseRequested),
        ]),
        None => Subscription::batch([window_events, focus_requests]),
    }
}

/// Build a window state backed by an explicit user data directory.
///
/// Shared by the tests so the focus channel, geometry persistence and the
/// raise-versus-fallback decision are exercised without a display server.
#[cfg(test)]
fn test_window(user_data_dir: &std::path::Path) -> MainWindow {
    MainWindow::new(user_data_dir.to_path_buf(), Geometry::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn tmp_dir() -> PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "gitea-stopwatch-window-test-{}-{}",
            std::process::id(),
            n
        ))
    }

    #[test]
    fn boot_starts_idle_with_the_injected_data_directory() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);

        let (state, _task) = boot(dir.clone());

        assert_eq!(state.focus, FocusStatus::Idle);
        assert_eq!(state.user_data_dir, dir);
        assert!(!state.closing);
    }

    #[test]
    fn polling_without_a_request_keeps_the_window_idle() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let mut state = test_window(&dir);

        let _task = update(&mut state, Message::PollFocusRequest);

        assert_eq!(state.focus, FocusStatus::Idle);
        assert!(!focus::focus_request_path(&dir).exists());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn polling_consumes_a_second_launch_focus_request() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);
        focus::request_focus(&dir).unwrap();
        let mut state = test_window(&dir);

        let _task = update(&mut state, Message::PollFocusRequest);

        assert!(
            !focus::focus_request_path(&dir).exists(),
            "the request must be consumed as it is observed"
        );
        // The raise itself waits for the resolved window id, so the status is
        // still idle until `FocusTargetResolved` arrives.
        assert_eq!(state.focus, FocusStatus::Idle);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_resolved_window_raises_it() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);
        let mut state = test_window(&dir);

        let _task = update(
            &mut state,
            Message::FocusTargetResolved(Some(window::Id::unique())),
        );

        assert_eq!(state.focus, FocusStatus::Raised);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_missing_window_applies_the_documented_fallback() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);
        let mut state = test_window(&dir);

        let _task = update(&mut state, Message::FocusTargetResolved(None));

        assert_eq!(state.focus, FocusStatus::Unsupported);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn resize_and_move_update_the_persisted_geometry() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);
        let mut state = test_window(&dir);

        let _task = update(
            &mut state,
            Message::Resized {
                size: iced::Size::new(1024.0, 768.0),
            },
        );
        let _task = update(
            &mut state,
            Message::Moved {
                position: iced::Point::new(20.0, 30.0),
            },
        );

        assert_eq!(state.geometry.width, 1024.0);
        assert_eq!(state.geometry.height, 768.0);
        assert_eq!(state.geometry.x, 20.0);
        assert_eq!(state.geometry.y, 30.0);

        let _ = fs::remove_dir_all(&dir);
    }

    /// A restored geometry that neither fits a 1080p monitor nor sits on it.
    fn oversized_restore() -> Geometry {
        Geometry {
            width: 3200.0,
            height: 2000.0,
            x: 4000.0,
            y: 3000.0,
        }
    }

    #[test]
    fn a_smaller_reported_monitor_clamps_the_restored_geometry() {
        let clamped =
            geometry_for_monitor(oversized_restore(), Some(iced::Size::new(1920.0, 1080.0)));

        assert_eq!(
            clamped.width, 1920.0,
            "width must be clamped to the monitor"
        );
        assert_eq!(
            clamped.height, 1080.0,
            "height must be clamped to the monitor"
        );
        assert_eq!(clamped.x, 0.0, "an off-screen x must be pulled on screen");
        assert_eq!(clamped.y, 0.0, "an off-screen y must be pulled on screen");
    }

    #[test]
    fn a_larger_reported_monitor_keeps_the_restored_geometry() {
        let restored = Geometry {
            width: 1280.0,
            height: 800.0,
            x: 240.0,
            y: 120.0,
        };

        assert_eq!(
            geometry_for_monitor(restored, Some(iced::Size::new(2560.0, 1440.0))),
            restored,
            "a geometry that fits the monitor must be restored verbatim"
        );
    }

    #[test]
    fn an_unreported_monitor_keeps_the_restored_geometry() {
        let restored = oversized_restore();

        assert_eq!(
            geometry_for_monitor(restored, None),
            restored,
            "without a monitor report the geometry must be left untouched"
        );
        assert_eq!(monitor_bounds(None), None);
    }

    #[test]
    fn a_zero_sized_monitor_report_is_treated_as_unreported() {
        let restored = oversized_restore();

        for size in [
            iced::Size::new(0.0, 0.0),
            iced::Size::new(0.0, 1080.0),
            iced::Size::new(1920.0, 0.0),
        ] {
            assert_eq!(
                monitor_bounds(Some(size)),
                None,
                "a zero-sized report carries no usable bounds"
            );
            assert_eq!(
                geometry_for_monitor(restored, Some(size)),
                restored,
                "a degenerate report must not move or shrink the window"
            );
        }
    }

    #[test]
    fn a_reported_monitor_pulls_an_offscreen_restore_back_on_screen() {
        let restored = Geometry {
            width: 800.0,
            height: 600.0,
            x: 5000.0,
            y: 5000.0,
        };

        let clamped = geometry_for_monitor(restored, Some(iced::Size::new(1920.0, 1080.0)));

        assert_eq!(clamped.x, 1120.0);
        assert_eq!(clamped.y, 480.0);
    }

    #[test]
    fn the_monitor_report_clamps_and_applies_the_restored_geometry() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);
        let mut state = MainWindow::restoring(dir.clone(), oversized_restore());

        let _task = update(
            &mut state,
            Message::MonitorResolved {
                id: window::Id::unique(),
                size: Some(iced::Size::new(1280.0, 720.0)),
            },
        );

        assert_eq!(state.geometry.width, 1280.0);
        assert_eq!(state.geometry.height, 720.0);
        assert_eq!(state.geometry.x, 0.0);
        assert_eq!(state.geometry.y, 0.0);
        assert!(
            state.pending_restore.is_none(),
            "the restore must be applied exactly once"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_applied_restore_is_the_geometry_persisted_on_close() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);
        let mut state = MainWindow::restoring(dir.clone(), oversized_restore());

        let _task = update(
            &mut state,
            Message::MonitorResolved {
                id: window::Id::unique(),
                size: Some(iced::Size::new(1280.0, 720.0)),
            },
        );
        let _task = update(
            &mut state,
            Message::CloseRequested {
                id: window::Id::unique(),
            },
        );

        assert_eq!(
            state.geometry.width, 1280.0,
            "the close path must persist the clamped geometry"
        );
        assert_eq!(state.geometry.height, 720.0);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_restore_in_flight_ignores_the_default_window_events() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);
        let restored = oversized_restore();
        let mut state = MainWindow::restoring(dir.clone(), restored);

        // The runtime opens the window with its own default size before the
        // clamp is applied; those events must not overwrite the restore.
        let _task = update(
            &mut state,
            Message::Resized {
                size: iced::Size::new(1024.0, 768.0),
            },
        );
        let _task = update(
            &mut state,
            Message::Moved {
                position: iced::Point::new(0.0, 0.0),
            },
        );

        assert_eq!(state.geometry, restored);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_unresolved_window_keeps_the_restored_geometry() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);
        let restored = oversized_restore();
        let mut state = MainWindow::restoring(dir.clone(), restored);

        let _task = update(&mut state, Message::WindowResolved(None));

        assert_eq!(
            state.geometry, restored,
            "the documented fallback keeps the restored geometry as loaded"
        );
        assert!(state.pending_restore.is_none());
        assert!(!state.closing);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_monitor_report_without_a_pending_restore_is_ignored() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);
        let mut state = test_window(&dir);
        let before = state.geometry;

        let _task = update(
            &mut state,
            Message::MonitorResolved {
                id: window::Id::unique(),
                size: Some(iced::Size::new(640.0, 480.0)),
            },
        );

        assert_eq!(
            state.geometry, before,
            "a report arriving outside a restore must change nothing"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn close_marks_the_window_as_closing() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);
        let mut state = test_window(&dir);

        let _task = update(
            &mut state,
            Message::CloseRequested {
                id: window::Id::unique(),
            },
        );

        assert!(state.closing);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn auto_close_is_disarmed_by_missing_or_invalid_values() {
        // A malformed value must never turn a normal run into a self-closing
        // one, so every unusable input leaves the harness disarmed.
        assert_eq!(parse_auto_close(None), None);
        assert_eq!(parse_auto_close(Some("")), None);
        assert_eq!(parse_auto_close(Some("   ")), None);
        assert_eq!(parse_auto_close(Some("soon")), None);
        assert_eq!(parse_auto_close(Some("-1")), None);
        assert_eq!(parse_auto_close(Some("0")), None);
    }

    #[test]
    fn auto_close_reads_a_millisecond_delay() {
        assert_eq!(
            parse_auto_close(Some("2500")),
            Some(Duration::from_millis(2500))
        );
        assert_eq!(
            parse_auto_close(Some(" 150 ")),
            Some(Duration::from_millis(150))
        );
    }

    #[test]
    fn the_configured_theme_is_the_light_theme() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);
        let state = test_window(&dir);

        assert_eq!(theme(&state), WINDOW_THEME);
        assert_eq!(WINDOW_THEME, Theme::Light);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_harness_without_a_window_keeps_the_instance_running() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);
        let mut state = test_window(&dir);

        let _task = update(&mut state, Message::AutoCloseTargetResolved(None));

        assert!(
            !state.closing,
            "the harness must not mark the window as closing when there is no window"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_harness_tick_resolves_the_window_before_closing() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);
        let mut state = test_window(&dir);

        // The tick only resolves which window to close; the close itself is
        // the same path the close button takes and is proven end-to-end by the
        // suite that runs the shipped binary.
        let _task = update(&mut state, Message::AutoCloseRequested);

        assert!(
            !state.closing,
            "an unresolved tick must not mark the window as closing"
        );

        let _ = fs::remove_dir_all(&dir);
    }
}
