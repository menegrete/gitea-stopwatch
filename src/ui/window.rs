//! Iced view layer: the empty main window shell.
//!
//! Provides an empty window with title, theme, and placeholder navigation
//! reserved for the future main window (spec: placeholder-navigation).
//! The window closes cleanly when the user closes it, leaving no residual
//! processes (spec: clean-shutdown). Window geometry is persisted and
//! restored (spec: window-geometry-persistence).

use iced::widget::{column, container, text};
use iced::{Alignment, Element, Length, Subscription, Task, window};

use crate::core::{Geometry, clamp_to_monitor, load_geometry, save_geometry};

/// Application title shown in the window decoration.
pub const APP_TITLE: &str = "gitea-stopwatch";

/// The empty main window application state.
pub struct MainWindow {
    /// Current window geometry (size and position).
    geometry: Geometry,
    /// Whether the window close has been requested.
    closing: bool,
}

impl MainWindow {
    fn new(geometry: Geometry) -> Self {
        Self {
            geometry,
            closing: false,
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
}

/// Boot function: creates the initial state and no startup task.
pub fn boot() -> (MainWindow, Task<Message>) {
    let geometry = load_geometry().ok().flatten().unwrap_or_default();
    let geometry = clamp_to_monitor(geometry, None);

    (MainWindow::new(geometry), Task::none())
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

/// Subscription function: listens for window close requests and geometry
/// changes.
pub fn subscription(_state: &MainWindow) -> Subscription<Message> {
    use iced::window::Event;

    window::events().filter_map(|(id, event)| match event {
        Event::CloseRequested => Some(Message::CloseRequested { id }),
        Event::Resized(size) => Some(Message::Resized { size }),
        Event::Moved(position) => Some(Message::Moved { position }),
        _ => None,
    })
}
