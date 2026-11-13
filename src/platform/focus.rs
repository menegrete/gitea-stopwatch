//! Focus-request channel between a second launch and the running instance.
//!
//! The single-instance lock ([`crate::platform::lock`]) keeps a second launch
//! from opening a second window, but the `single-instance-lock` requirement
//! also states that the already-running instance receives the focus
//! (scenario `second-launch-focuses-existing-instance`).
//!
//! A separate process cannot portably focus another process' window without
//! platform-specific FFI, and this crate forbids `unsafe` code, so the two
//! launches communicate through a marker file that lives next to the lock in
//! the user data directory:
//!
//! 1. The second launch writes the marker ([`request_focus`]) and exits
//!    without opening a window.
//! 2. The running instance polls the marker ([`take_focus_request`]), removes
//!    it, and asks the windowing system to raise and focus its own window
//!    through the Iced runtime (`iced::window::gain_focus`), which reaches
//!    X11, Win32 and Wayland without this crate writing any FFI.
//!
//! Where no window is available to raise — a headless run, or a request
//! observed before any window exists — [`decide_focus_action`] selects the
//! documented fallback: the request is consumed, a warning is logged, and the
//! instance keeps running with its window unchanged. The pure request channel
//! below needs no display server, so both the second-launch write and the
//! running-instance consume are exercised by unit and end-to-end tests.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Name of the marker file carrying a pending focus request.
pub const FOCUS_REQUEST_FILE_NAME: &str = "gitea-stopwatch.focus-request";

/// Path of the focus-request marker inside a user data directory.
pub fn focus_request_path(user_data_dir: &Path) -> PathBuf {
    user_data_dir.join(FOCUS_REQUEST_FILE_NAME)
}

/// What the running instance does with a pending focus request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusAction {
    /// A window exists: ask the windowing system to raise and focus it.
    Raise,
    /// No window exists to raise: log the documented fallback and keep running.
    LogFallback,
}

/// Decide how the running instance reacts to a focus request.
///
/// Pure: the caller resolves whether a window is available (`has_window`) and
/// this function maps that to the action, so the raise-versus-fallback branch
/// is testable without a display server.
pub fn decide_focus_action(has_window: bool) -> FocusAction {
    if has_window {
        FocusAction::Raise
    } else {
        FocusAction::LogFallback
    }
}

/// Writes a focus request for the running instance.
///
/// Called by a second launch after the single-instance lock refused it. The
/// marker records the requesting process id so a reader can attribute the
/// request. Writing is idempotent: repeated launches keep a single pending
/// request, which the running instance consumes once.
pub fn request_focus(user_data_dir: &Path) -> io::Result<()> {
    fs::create_dir_all(user_data_dir)?;
    fs::write(
        focus_request_path(user_data_dir),
        std::process::id().to_string(),
    )
}

/// Consumes a pending focus request, returning whether one was pending.
///
/// The request is removed as it is observed so a single second launch raises
/// the window once. A missing marker is not an error: `Ok(false)` means the
/// running instance simply found nothing to do.
pub fn take_focus_request(user_data_dir: &Path) -> io::Result<bool> {
    match fs::remove_file(focus_request_path(user_data_dir)) {
        Ok(()) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

/// Removes any pending focus request without reporting whether one existed.
///
/// Used when a process becomes the primary instance: a request left behind by
/// a previously crashed launch must not raise a window on a fresh start.
pub fn clear_focus_request(user_data_dir: &Path) -> io::Result<()> {
    match fs::remove_file(focus_request_path(user_data_dir)) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn tmp_dir() -> PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "gitea-stopwatch-focus-test-{}-{}",
            std::process::id(),
            n
        ))
    }

    #[test]
    fn focus_request_path_lives_in_the_user_data_directory() {
        let dir = Path::new("/data/gitea-stopwatch");
        assert_eq!(focus_request_path(dir), dir.join(FOCUS_REQUEST_FILE_NAME));
    }

    #[test]
    fn request_focus_creates_the_user_data_directory() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);

        request_focus(&dir).unwrap();

        assert!(dir.is_dir(), "requesting focus must create the directory");
        assert!(focus_request_path(&dir).exists());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn request_focus_records_the_requesting_process_id() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);

        request_focus(&dir).unwrap();

        assert_eq!(
            fs::read_to_string(focus_request_path(&dir)).unwrap(),
            std::process::id().to_string()
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn request_focus_is_idempotent() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);

        request_focus(&dir).unwrap();
        request_focus(&dir).unwrap();

        // A second request overwrites the marker instead of accumulating one
        // request per launch, so the running instance raises the window once.
        assert!(take_focus_request(&dir).unwrap());
        assert!(!take_focus_request(&dir).unwrap());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn take_focus_request_consumes_a_pending_request_once() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);
        request_focus(&dir).unwrap();

        assert!(take_focus_request(&dir).unwrap());
        assert!(
            !focus_request_path(&dir).exists(),
            "consuming the request must remove the marker"
        );
        assert!(
            !take_focus_request(&dir).unwrap(),
            "a consumed request must not be raised twice"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn take_focus_request_reports_no_request_when_the_marker_is_absent() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        assert!(!take_focus_request(&dir).unwrap());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn take_focus_request_reports_no_request_when_the_directory_is_absent() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);

        assert!(!take_focus_request(&dir).unwrap());
    }

    #[test]
    fn take_focus_request_surfaces_a_removal_failure() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);
        // A directory at the marker path cannot be removed as a file, which
        // must surface as an error instead of a silent `false`.
        fs::create_dir_all(focus_request_path(&dir)).unwrap();

        assert!(take_focus_request(&dir).is_err());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn clear_focus_request_removes_a_pending_request() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);
        request_focus(&dir).unwrap();

        clear_focus_request(&dir).unwrap();

        assert!(!focus_request_path(&dir).exists());
        assert!(!take_focus_request(&dir).unwrap());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn clear_focus_request_is_idempotent() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);

        clear_focus_request(&dir).unwrap();
        clear_focus_request(&dir).unwrap();
    }

    #[test]
    fn decide_focus_action_raises_when_a_window_exists() {
        assert_eq!(decide_focus_action(true), FocusAction::Raise);
    }

    #[test]
    fn decide_focus_action_falls_back_without_a_window() {
        assert_eq!(decide_focus_action(false), FocusAction::LogFallback);
    }
}
