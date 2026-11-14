//! Single-instance lock via a lock file in the platform user data directory.
//!
//! The lock file namespaces the application by process ID. On startup, the
//! application tries to create the lock file exclusively. If it already
//! exists and the owning process is still alive, the new instance does not own
//! the lock. If the owning process is dead, the stale lock is removed and the
//! new instance proceeds (spec: single-instance-lock).
//!
//! Liveness is resolved against the operating system process table through
//! [`sysinfo`], so orphan-lock recovery behaves the same on the platforms the
//! application ships to (Linux and Windows) and on macOS, and this crate keeps
//! its `#![forbid(unsafe_code)]` invariant.
//!
//! [`try_acquire`] reports the outcome so the launch path can tell a genuine
//! second launch from a fresh start and forward a focus request to the running
//! instance ([`crate::platform::instance`]); [`acquire`] keeps the simpler
//! "become the only instance or fail" contract.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

use crate::core::{Error, Result};

const LOCK_FILE_NAME: &str = "gitea-stopwatch.lock";

/// Acquisition attempts per call: the initial `create_new` plus one retry after
/// a stale lock is removed. The retry is capped so an undeletable stale lock
/// (read-only user-data directory, Windows sharing violation) reports a typed
/// error instead of recursing without bound.
const ACQUIRE_ATTEMPTS: u32 = 2;

/// Outcome of a single-instance acquisition attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockOutcome {
    /// This process created the lock and is the only running instance.
    Acquired,
    /// A live process already owns the lock. `owner_pid` is the PID recorded in
    /// the lock file, when it is parseable.
    AlreadyRunning {
        /// PID recorded in the lock file by the owning process, if parseable.
        owner_pid: Option<u32>,
    },
}

/// Tries to acquire the single-instance lock in the user data directory.
///
/// Returns [`LockOutcome::Acquired`] when this process created the lock, and
/// [`LockOutcome::AlreadyRunning`] when a live process already owns it. Stale
/// locks whose owner is no longer alive are removed and retried at most once;
/// when the stale lock cannot be replaced the call returns a typed [`Error`]
/// rather than retrying without bound.
pub fn try_acquire(user_data_dir: &Path) -> Result<LockOutcome> {
    let lock_path = user_data_dir.join(LOCK_FILE_NAME);

    fs::create_dir_all(user_data_dir)?;

    // A stale lock that cannot be removed leaves this populated so the
    // exhausted-attempts path can report the underlying I/O failure.
    let mut stale_lock_removal: Option<io::Error> = None;

    for _ in 0..ACQUIRE_ATTEMPTS {
        // Try to create the lock file exclusively. If it already exists,
        // another instance may be running.
        match OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&lock_path)
        {
            Ok(mut file) => {
                // We own the lock. Write our PID so a stale lock can be detected.
                let _ = writeln!(file, "{}", std::process::id());
                let _ = file.flush();
                return Ok(LockOutcome::Acquired);
            }
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                // Lock file exists: check whether the owner is still alive.
                let owner_pid = read_owner_pid(&lock_path);
                match owner_pid {
                    Some(pid) if is_process_alive(pid) => {
                        return Ok(LockOutcome::AlreadyRunning {
                            owner_pid: Some(pid),
                        });
                    }
                    _ => {
                        // Stale lock: remove it and let the bounded loop retry.
                        match fs::remove_file(&lock_path) {
                            Ok(()) => {}
                            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
                            Err(e) => stale_lock_removal = Some(e),
                        }
                    }
                }
            }
            Err(e) => return Err(Error::Io { source: e }),
        }
    }

    Err(match stale_lock_removal {
        Some(source) => Error::Io { source },
        None => Error::Platform {
            operation: "single-instance lock not acquired after removing a stale lock".to_string(),
        },
    })
}

/// Acquires the single-instance lock in the user data directory.
///
/// Returns `Ok(())` if the lock was acquired (this is the first instance).
/// Returns an [`Error`] if another instance is already running or if the
/// lock file cannot be created or read.
pub fn acquire(user_data_dir: &Path) -> Result<()> {
    match try_acquire(user_data_dir)? {
        LockOutcome::Acquired => Ok(()),
        LockOutcome::AlreadyRunning { .. } => Err(Error::Platform {
            operation: "single-instance lock already held".to_string(),
        }),
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

/// Checks whether a process with the given PID is alive on this platform.
///
/// The operating system process table is read through [`sysinfo`] and `pid` is
/// looked up there, which gives one code path with the same semantics on Linux,
/// Windows and macOS. A process table that cannot be read at all reports the
/// owner as alive: an unverifiable lock is kept instead of removed, so the worst
/// outcome is a relaunch that refuses to start rather than two live instances.
///
/// PID reuse is not detected: an owner that crashed while its PID was reassigned
/// keeps its lock until the reusing process exits, which the next launch repairs.
fn is_process_alive(pid: u32) -> bool {
    let pid = Pid::from_u32(pid);

    let Ok(mut system) = System::new() else {
        // Fail safe: without the process table the owner cannot be proven dead.
        return true;
    };

    // Only the requested PID is refreshed; the process kind carries no
    // per-process metrics because liveness needs the table entry alone.
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[pid]),
        false,
        ProcessRefreshKind::nothing(),
    );

    system.process(pid).is_some()
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
        // `u32::MAX` is outside the PID space of every supported platform
        // (Linux caps PIDs at `/proc/sys/kernel/pid_max`, Windows allocates
        // PIDs from a much smaller table), so no process can own it.
        assert!(!is_process_alive(u32::MAX));
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

    #[test]
    fn try_acquire_returns_acquired_for_a_free_lock() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);

        assert_eq!(try_acquire(&dir).unwrap(), LockOutcome::Acquired);
        assert!(dir.join(LOCK_FILE_NAME).exists());

        release(&dir).unwrap();
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn try_acquire_reports_the_live_owner_of_a_held_lock() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);
        // Acquiring with this process' own PID makes the owner provably alive.
        assert_eq!(try_acquire(&dir).unwrap(), LockOutcome::Acquired);

        assert_eq!(
            try_acquire(&dir).unwrap(),
            LockOutcome::AlreadyRunning {
                owner_pid: Some(std::process::id())
            }
        );

        release(&dir).unwrap();
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn try_acquire_keeps_the_owner_pid_written_to_the_lock_file() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let lock_path = dir.join(LOCK_FILE_NAME);
        {
            let mut file = File::create(&lock_path).unwrap();
            // The lock file is written as a foreign launch would leave it: this
            // process did not create it, so the reported owner PID has to be the
            // one read from disk rather than an assumed self PID.
            writeln!(file, "{}", std::process::id()).unwrap();
        }

        assert_eq!(
            try_acquire(&dir).unwrap(),
            LockOutcome::AlreadyRunning {
                owner_pid: Some(std::process::id())
            }
        );
        assert_eq!(
            fs::read_to_string(&lock_path).unwrap().trim(),
            std::process::id().to_string(),
            "a lock held by a live owner must be left untouched"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn acquire_replaces_lock_file_owned_by_dead_process() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let lock_path = dir.join(LOCK_FILE_NAME);
        // A PID outside the PID space of every supported platform is never alive,
        // so the orphaned lock must be recovered on Linux and Windows alike.
        fs::write(&lock_path, u32::MAX.to_string()).unwrap();

        assert!(acquire(&dir).is_ok());
        assert_eq!(
            fs::read_to_string(&lock_path).unwrap().trim(),
            std::process::id().to_string()
        );

        release(&dir).unwrap();
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn try_acquire_returns_error_when_a_stale_lock_cannot_be_removed() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);
        // A directory at the lock path is never removable with `remove_file`, so
        // stale-lock recovery cannot succeed: the bounded attempts must report a
        // typed error instead of recursing until the stack overflows.
        fs::create_dir_all(dir.join(LOCK_FILE_NAME)).unwrap();

        assert!(matches!(
            try_acquire(&dir),
            Err(Error::Io { .. } | Error::Platform { .. })
        ));

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
