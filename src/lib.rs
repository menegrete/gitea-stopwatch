//! Public API for the gitea-stopwatch desktop shell.

#![forbid(unsafe_code)]

pub mod core;
pub mod platform;
pub mod ui;

use crate::platform::lock;
use crate::ui::window::MainWindow;
use iced::Theme;

/// Run the gitea-stopwatch application.
///
/// Ensures a single instance via a lock file, then runs the Iced main
/// window event loop. Returns a typed [`core::Error`]; the binary edge
/// converts it into an `anyhow` error with added context and an exit code.
pub fn run() -> core::Result<()> {
    let user_data_dir = crate::platform::dirs::user_data_dir()?;

    // Single-instance lock: exit if another instance is already running.
    lock::acquire(&user_data_dir).map_err(|err| core::Error::Platform {
        operation: format!("single-instance lock: {err}"),
    })?;

    // Ensure the lock is released when the application exits.
    let _lock_guard = LockGuard { user_data_dir };

    iced::application(
        crate::ui::window::boot,
        crate::ui::window::update,
        crate::ui::window::view,
    )
    .theme(|_: &MainWindow| Theme::Light)
    .subscription(|state: &MainWindow| crate::ui::window::subscription(state))
    .run()
    .map_err(|err| core::Error::Platform {
        operation: format!("iced shell: {err}"),
    })
}

/// Guards the single-instance lock, releasing it on drop.
struct LockGuard {
    user_data_dir: std::path::PathBuf,
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        let _ = lock::release(&self.user_data_dir);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lock_guard_releases_the_lock_file_on_drop() {
        let dir = std::env::temp_dir().join(format!(
            "gitea-stopwatch-lock-guard-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        lock::acquire(&dir).unwrap();
        let lock_path = dir.join("gitea-stopwatch.lock");
        assert!(lock_path.exists());

        {
            let _guard = LockGuard {
                user_data_dir: dir.clone(),
            };
        }

        assert!(
            !lock_path.exists(),
            "dropping the guard must release the lock"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }
}
