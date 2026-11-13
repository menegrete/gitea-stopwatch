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
use iced::{Alignment, Element, Length, Subscription, Task, time, window};

use crate::core::{Geometry, clamp_to_monitor, load_geometry, save_geometry};
use crate::platform::focus::{self, FocusAction};
use crate::platform::logging::{self, Level};

/// Application title shown in the window decoration.
pub const APP_TITLE: &str = "gitea-stopwatch";

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
pub struct MainWindow {
    /// Current window geometry (size and position).
    geometry: Geometry,
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
            closing: false,
            user_data_dir,
            focus: FocusStatus::Idle,
        }
    }
}

/// Messages produced by the main window.
#[derive(Debug, Clone)]
pub enum Message {
    /// The user requested to close the window, carrying the window id.
    CloseRequested { id: window::Id },
    /// A placeholder navigation item was activated (no-op for now).
    PlaceholderNavActivated,
    /// The window was resized, carrying the new size.
    Resized { size: iced::Size },
    /// The window was moved, carrying the new position.
    Moved { position: iced::Point },
    /// Timer tick: look for a focus request left by a second launch.
    PollFocusRequest,
    /// The runtime resolved which window, if any, answers a focus request.
    FocusTargetResolved(Option<window::Id>),
}

/// Boot function: creates the initial state and no startup task.
///
/// The user data directory is injected by the library edge so the window and
/// the single-instance lock observe the same directory.
pub fn boot(user_data_dir: PathBuf) -> (MainWindow, Task<Message>) {
    let geometry = load_geometry().ok().flatten().unwrap_or_default();
    let geometry = clamp_to_monitor(geometry, None);

    (MainWindow::new(user_data_dir, geometry), Task::none())
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
            state.geometry.width = size.width as f64;
            state.geometry.height = size.height as f64;
            Task::none()
        }
        Message::Moved { position } => {
            state.geometry.x = position.x as f64;
            state.geometry.y = position.y as f64;
            Task::none()
        }
        Message::PlaceholderNavActivated => Task::none(),
        Message::PollFocusRequest => match focus::take_focus_request(&state.user_data_dir) {
            Ok(true) => {
                logging::log(
                    Level::Info,
                    module_path!(),
                    &[],
                    "focus request received from a second launch",
                );
                // Ask the runtime which window answers the request; the answer
                // arrives as `FocusTargetResolved` and decides raise vs fallback.
                window::latest().map(Message::FocusTargetResolved)
            }
            Ok(false) => Task::none(),
            Err(error) => {
                logging::log(
                    Level::Warn,
                    module_path!(),
                    &[("error", &error.to_string())],
                    "focus request could not be consumed",
                );
                Task::none()
            }
        },
        Message::FocusTargetResolved(target) => {
            match focus::decide_focus_action(target.is_some()) {
                FocusAction::Raise => {
                    state.focus = FocusStatus::Raised;
                    logging::log(
                        Level::Info,
                        module_path!(),
                        &[],
                        "raising the existing window for a second launch",
                    );
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
                    logging::log(
                        Level::Warn,
                        module_path!(),
                        &[],
                        "focus requested; raising the window is unsupported on this platform",
                    );
                    Task::none()
                }
            }
        }
    }
}

/// View function: renders the empty main window with placeholder navigation.
pub fn view(_state: &MainWindow) -> Element<'_, Message> {
    let nav = column![
        text("Placeholder navigation").size(18),
        text("Reserved for the future main window.").size(14),
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

    Subscription::batch([window_events, focus_requests])
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
}
