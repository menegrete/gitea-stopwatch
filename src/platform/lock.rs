//! Single-instance lock via a lock file in the platform user data directory.
//!
//! The lock file namespaces the application by process ID. On startup, the
//! application tries to create the lock file exclusively. If it already
//! exists and the owning process is still alive, the new instance exits.
//! If the owning process is dead, the stale lock is removed and the new
//! instance proceeds (spec: single-instance-lock).

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use crate::core::{Error, Result};

const LOCK_FILE_NAME: &str = "gitea-stopwatch.lock";

/// Acquires the single-instance lock in the user data directory.
///
/// Returns `Ok(())` if the lock was acquired (this is the first instance).
/// Returns an [`Error`] if another instance is already running or if the
/// lock file cannot be created or read.
pub fn acquire(user_data_dir: &Path) -> Result<()> {
    let lock_path = user_data_dir.join(LOCK_FILE_NAME);

    fs::create_dir_all(user_data_dir)?;

    // Try to create the lock file exclusively. If it already exists, another
    // instance may be running.
    match OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&lock_path)
    {
        Ok(mut file) => {
            // We own the lock. Write our PID so a stale lock can be detected.
            let _ = writeln!(file, "{}", std::process::id());
            let _ = file.flush();
            Ok(())
        }
        Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => {
            // Lock file exists: check whether the owner is still alive.
            match read_owner_pid(&lock_path) {
                Some(pid) if is_process_alive(pid) => Err(Error::Platform {
                    operation: "single-instance lock already held".to_string(),
                }),
                _ => {
                    // Stale lock: remove it and try again.
                    let _ = fs::remove_file(&lock_path);
                    acquire(user_data_dir)
                }
            }
        }
        Err(e) => Err(Error::Io { source: e }),
    }
}

/// Releases the single-instance lock by removing the lock file.
pub fn release(user_data_dir: &Path) -> io::Result<()> {
    let lock_path = user_data_dir.join(LOCK_FILE_NAME);
    let _ = fs::remove_file(&lock_path);
    Ok(())
}

/// Reads the owner PID from the lock file, if present and parseable.
fn read_owner_pid(lock_path: &Path) -> Option<u32> {
    let contents = fs::read_to_string(lock_path).ok()?;
    contents.trim().parse::<u32>().ok()
}

/// Checks whether a process with the given PID is still alive.
///
/// On Linux, this checks `/proc/<pid>`. On other platforms, this is a
/// best-effort check that returns `true` (assume alive) to avoid
/// falsely removing a lock we cannot verify.
fn is_process_alive(pid: u32) -> bool {
    #[cfg(target_os = "linux")]
    {
        Path::new("/proc").join(pid.to_string()).exists()
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = pid;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::env;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn tmp_dir() -> PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        env::temp_dir().join(format!(
            "gitea-stopwatch-lock-test-{}-{}",
            std::process::id(),
            n
        ))
    }

    #[test]
    fn acquire_creates_lock_file() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        assert!(acquire(&dir).is_ok());
        assert!(dir.join(LOCK_FILE_NAME).exists());

        release(&dir).unwrap();
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn acquire_fails_when_lock_held_by_alive_process() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        // Pre-create the lock file with our PID.
        let lock_path = dir.join(LOCK_FILE_NAME);
        {
            let mut file = File::create(&lock_path).unwrap();
            writeln!(file, "{}", std::process::id()).unwrap();
        }

        // Acquisition should fail because our own process is alive.
        let result = acquire(&dir);
        assert!(result.is_err());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn release_removes_lock_file() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        acquire(&dir).unwrap();
        assert!(dir.join(LOCK_FILE_NAME).exists());

        release(&dir).unwrap();
        assert!(!dir.join(LOCK_FILE_NAME).exists());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn is_process_alive_returns_true_for_own_pid() {
        assert!(is_process_alive(std::process::id()));
    }

    #[test]
    fn is_process_alive_returns_false_for_nonexistent_pid() {
        assert!(!is_process_alive(99999999));
    }

    #[test]
    fn read_owner_pid_returns_none_for_unparseable_contents() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let lock_path = dir.join(LOCK_FILE_NAME);
        fs::write(&lock_path, "not-a-pid").unwrap();

        assert_eq!(read_owner_pid(&lock_path), None);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_owner_pid_returns_none_when_lock_file_is_missing() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);

        assert_eq!(read_owner_pid(&dir.join(LOCK_FILE_NAME)), None);
    }

    #[test]
    fn acquire_replaces_lock_file_with_unparseable_owner() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let lock_path = dir.join(LOCK_FILE_NAME);
        fs::write(&lock_path, "garbage").unwrap();

        // An unparseable owner cannot be alive, so the stale lock is replaced.
        assert!(acquire(&dir).is_ok());
        assert_eq!(
            fs::read_to_string(&lock_path).unwrap().trim(),
            std::process::id().to_string()
        );

        release(&dir).unwrap();
        let _ = fs::remove_dir_all(&dir);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn acquire_replaces_lock_file_owned_by_dead_process() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let lock_path = dir.join(LOCK_FILE_NAME);
        // A PID that is not present in /proc can never be alive.
        fs::write(&lock_path, "99999999").unwrap();

        assert!(acquire(&dir).is_ok());
        assert_eq!(
            fs::read_to_string(&lock_path).unwrap().trim(),
            std::process::id().to_string()
        );

        release(&dir).unwrap();
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn release_is_idempotent_when_lock_file_is_absent() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        assert!(release(&dir).is_ok());
        assert!(release(&dir).is_ok());

        let _ = fs::remove_dir_all(&dir);
    }
}
