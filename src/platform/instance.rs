//! Launch arbitration for the single-instance contract.
//!
//! A process that starts either becomes the primary instance (it owns the
//! single-instance lock and opens the main window) or forwards a focus request
//! to the instance that already runs and exits without opening a window
//! (spec: single-instance-lock, scenario
//! `second-launch-focuses-existing-instance`).
//!
//! Arbitrating in one place keeps the launcher of `crate::run` and the
//! end-to-end tests driving the same sequence, and keeps the lock primitive
//! ([`crate::platform::lock`]) free of window concerns.

use std::path::Path;

use crate::core::Result;

use crate::platform::focus;
use crate::platform::lock::{self, LockOutcome};

/// The role a starting process takes in the single-instance contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchRole {
    /// This process owns the lock and opens the main window.
    Primary,
    /// Another instance already runs: it was asked to focus its window and this
    /// process exits without opening one.
    ForwardedToRunningInstance,
}

/// Claims the single-instance role for this process.
///
/// When the process becomes the primary instance any stale focus request left
/// by a previously crashed launch is discarded, so a fresh start never raises a
/// window in response to an obsolete request. When another instance owns the
/// lock this process writes a focus request for it and reports
/// [`LaunchRole::ForwardedToRunningInstance`]; failing to deliver that request
/// is logged but never prevents this process from exiting without a second
/// window.
pub fn claim(user_data_dir: &Path) -> Result<LaunchRole> {
    match lock::try_acquire(user_data_dir)? {
        LockOutcome::Acquired => {
            // Best effort: a leftover request must not raise a window on a
            // fresh start, and a failure to clear it cannot invalidate the
            // lock this process now owns.
            let _ = focus::clear_focus_request(user_data_dir);
            Ok(LaunchRole::Primary)
        }
        LockOutcome::AlreadyRunning { .. } => {
            if let Err(error) = focus::request_focus(user_data_dir) {
                tracing::warn!(
                    error = error.to_string(),
                    "focus request could not be delivered; the running instance was not raised"
                );
            }
            Ok(LaunchRole::ForwardedToRunningInstance)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn tmp_dir() -> PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "gitea-stopwatch-instance-test-{}-{}",
            std::process::id(),
            n
        ))
    }

    #[test]
    fn claim_returns_primary_when_the_lock_is_free() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);

        assert_eq!(claim(&dir).unwrap(), LaunchRole::Primary);
        assert!(dir.join("gitea-stopwatch.lock").exists());

        lock::release(&dir).unwrap();
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn claim_returns_forwarded_when_another_instance_holds_the_lock() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);
        // This process is the running instance: its own live pid keeps the lock.
        lock::acquire(&dir).unwrap();

        assert_eq!(claim(&dir).unwrap(), LaunchRole::ForwardedToRunningInstance);

        lock::release(&dir).unwrap();
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn claim_writes_a_focus_request_when_forwarding() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);
        lock::acquire(&dir).unwrap();

        claim(&dir).unwrap();

        assert!(
            focus::focus_request_path(&dir).exists(),
            "a forwarded launch must leave a focus request for the running instance"
        );
        assert_eq!(
            fs::read_to_string(focus::focus_request_path(&dir)).unwrap(),
            std::process::id().to_string(),
            "the request must record the forwarding process id"
        );

        lock::release(&dir).unwrap();
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn claim_discards_a_stale_focus_request_when_becoming_primary() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);
        // A request left behind by an earlier, crashed launch.
        focus::request_focus(&dir).unwrap();

        assert_eq!(claim(&dir).unwrap(), LaunchRole::Primary);
        assert!(
            !focus::focus_request_path(&dir).exists(),
            "a primary launch must not honor an obsolete focus request"
        );

        lock::release(&dir).unwrap();
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn claim_reports_primary_again_after_the_running_instance_exits() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);
        lock::acquire(&dir).unwrap();
        assert_eq!(claim(&dir).unwrap(), LaunchRole::ForwardedToRunningInstance);

        lock::release(&dir).unwrap();

        assert_eq!(claim(&dir).unwrap(), LaunchRole::Primary);

        lock::release(&dir).unwrap();
        let _ = fs::remove_dir_all(&dir);
    }
}
