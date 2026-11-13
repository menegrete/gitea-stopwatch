//! Public API for the gitea-stopwatch desktop shell.

#![forbid(unsafe_code)]

pub mod core;
pub mod platform;
pub mod ui;

use std::path::PathBuf;

use crate::platform::instance::{LaunchRole, claim};
use crate::platform::logging::{self, Level};
use crate::ui::window::MainWindow;
use iced::Theme;

/// Run the gitea-stopwatch application.
///
/// Claims the single-instance role first, then runs the Iced main window event
/// loop. A process that is not the primary instance asks the running instance
/// to focus its window and exits without opening a second one (spec:
/// single-instance-lock). Returns a typed [`core::Error`]; the binary edge
/// converts it into an `anyhow` error with added context and an exit code.
pub fn run() -> core::Result<()> {
    let user_data_dir = crate::platform::dirs::user_data_dir()?;

    match claim(&user_data_dir)? {
        LaunchRole::Primary => run_shell(user_data_dir),
        LaunchRole::ForwardedToRunningInstance => {
            logging::log(
                Level::Info,
                module_path!(),
                &[],
                "another instance is running; focus requested; exiting without opening a window",
            );
            Ok(())
        }
    }
}

/// Runs the shell as the primary instance, holding the lock for its lifetime.
fn run_shell(user_data_dir: PathBuf) -> core::Result<()> {
    // Ensure the lock is released when the application exits.
    let _lock_guard = LockGuard {
        user_data_dir: user_data_dir.clone(),
    };

    // The window polls the same user data directory for focus requests left by
    // a second launch, so the boot closure receives it explicitly.
    let boot_dir = user_data_dir;
    iced::application(
        move || crate::ui::window::boot(boot_dir.clone()),
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
        let _ = crate::platform::lock::release(&self.user_data_dir);
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
        crate::platform::lock::acquire(&dir).unwrap();
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
