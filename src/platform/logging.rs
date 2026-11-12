use std::fmt::Write as FmtWrite;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::{Error, Result};

const LOG_PREFIX: &str = "gitea-stopwatch";
const LOG_EXTENSION: &str = ".log";

/// Default maximum size of a single log file before size-based rotation (5 MB).
const DEFAULT_MAX_SIZE: u64 = 5_000_000;
/// Default maximum number of retained log files (current plus archived).
const DEFAULT_MAX_FILES: usize = 5;

/// Days-since-epoch counter used to detect day boundaries for time rotation.
fn today() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() / 86_400)
        .unwrap_or(0)
}

/// Severity of a log event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Level::Error => write!(f, "ERROR"),
            Level::Warn => write!(f, "WARN "),
            Level::Info => write!(f, "INFO "),
            Level::Debug => write!(f, "DEBUG"),
            Level::Trace => write!(f, "TRACE"),
        }
    }
}

/// A rolling file writer that rotates on size and time and retains a bounded
/// number of archived files.
///
/// Rotation happens when the current day changes (time rotation) or when the
/// next write would exceed `max_size` (size rotation). Archived files are named
/// `<prefix>.log.<timestamp>` and only the most recent `max_files - 1` archives
/// are retained, keeping the total on-disk footprint bounded.
pub struct RollingFileWriter {
    dir: PathBuf,
    prefix: String,
    max_size: u64,
    max_files: usize,
    file: Option<File>,
    current_size: u64,
    current_date: u64,
    path: PathBuf,
}

impl RollingFileWriter {
    pub fn new(dir: &Path, prefix: &str, max_size: u64, max_files: usize) -> Result<Self> {
        fs::create_dir_all(dir)?;
        let path = dir.join(format!("{}{}", prefix, LOG_EXTENSION));
        let mut writer = Self {
            dir: dir.to_path_buf(),
            prefix: prefix.to_string(),
            max_size,
            max_files,
            file: None,
            current_size: 0,
            current_date: today(),
            path,
        };
        writer.open_current()?;
        Ok(writer)
    }

    fn open_current(&mut self) -> io::Result<()> {
        self.current_size = self.path.metadata().map_or(0, |m| m.len());
        self.current_date = today();
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        self.file = Some(file);
        Ok(())
    }

    fn rotate(&mut self) -> io::Result<()> {
        if let Some(file) = self.file.take() {
            drop(file);
        }
        if self.path.exists() {
            let archive = archive_path(&self.dir, &self.prefix);
            fs::rename(&self.path, &archive)?;
            self.cleanup()?;
        }
        self.open_current()
    }

    fn cleanup(&mut self) -> io::Result<()> {
        let pattern = format!("{}.log.", self.prefix);
        let mut entries: Vec<PathBuf> = fs::read_dir(&self.dir)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.file_name()
                    .and_then(|s| s.to_str())
                    .is_some_and(|s| s.starts_with(&pattern))
            })
            .collect();
        entries.sort();
        let keep = self.max_files.saturating_sub(1);
        while entries.len() > keep {
            if let Some(oldest) = entries.first() {
                let _ = fs::remove_file(oldest);
                entries.remove(0);
            }
        }
        Ok(())
    }
}

impl Write for RollingFileWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let now = today();
        let will_exceed = self
            .current_size
            .checked_add(buf.len() as u64)
            .is_none_or(|s| s > self.max_size);
        if now != self.current_date || will_exceed {
            self.rotate()?;
        }
        let file = self
            .file
            .as_mut()
            .ok_or_else(|| io::Error::other("log file is not open"))?;
        let n = file.write(buf)?;
        self.current_size += n as u64;
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.as_mut().map_or(Ok(()), |f| f.flush())
    }
}

fn archive_path(dir: &Path, prefix: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| format!("{:x}", d.as_secs()))
        .unwrap_or_else(|_| "unknown".to_string());
    dir.join(format!("{}.log.{}", prefix, stamp))
}

/// Global logger: initialized once by [`init`]. Writes are synchronized so the
/// panic hook and application threads can log concurrently.
static LOGGER: OnceLock<Mutex<Option<RollingFileWriter>>> = OnceLock::new();

/// Write a structured log event to the rolling log file.
///
/// `fields` are emitted as `key="value"` pairs. No credential, token, personal
/// data, or full file/network body is ever written here.
pub fn log(level: Level, target: &str, fields: &[(&str, &str)], message: &str) {
    let Some(logger) = LOGGER.get() else {
        return;
    };
    let mut guard = match logger.lock() {
        Ok(g) => g,
        Err(_) => return,
    };
    let Some(writer) = guard.as_mut() else {
        return;
    };
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let mut fields_out = String::new();
    for (key, value) in fields {
        let _ = write!(fields_out, " {key}=\"{value}\"");
    }
    let line = format!(
        "{ts} {level:5} [{target}]{fields} {message}\n",
        ts = secs,
        level = level,
        target = target,
        fields = fields_out,
        message = message
    );
    let _ = writer.write_all(line.as_bytes());
    let _ = writer.flush();
}

/// Initialize logging: install the rolling log writer in `log_dir` and the
/// panic hook, which records the panic location and payload to the log before
/// the process aborts.
pub fn init(log_dir: &Path) -> Result<()> {
    let writer = RollingFileWriter::new(log_dir, LOG_PREFIX, DEFAULT_MAX_SIZE, DEFAULT_MAX_FILES)?;
    LOGGER
        .set(Mutex::new(Some(writer)))
        .map_err(|_| Error::Platform {
            operation: "install logging".to_string(),
        })?;

    std::panic::set_hook(Box::new(|info| {
        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown".to_string());
        let payload = if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "<non-string panic payload>".to_string()
        };
        log(
            Level::Error,
            "panic",
            &[],
            &format!("panic occurred at {location}: {payload}"),
        );
    }));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    /// Monotonic counter for unique per-test temp directories (tests run in parallel).
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn tmp_dir() -> PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("gitea-stopwatch-test-{}-{}", std::process::id(), n))
    }

    #[test]
    fn rolling_writer_creates_current_file() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);
        let mut writer =
            RollingFileWriter::new(&dir, LOG_PREFIX, DEFAULT_MAX_SIZE, DEFAULT_MAX_FILES).unwrap();
        writer.write_all(b"hello").unwrap();
        writer.flush().unwrap();
        drop(writer);
        assert!(
            dir.join(format!("{}{}", LOG_PREFIX, LOG_EXTENSION))
                .exists()
        );
    }

    #[test]
    fn rolling_writer_rotates_on_size() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);
        let mut writer = RollingFileWriter::new(&dir, "t", 20, 5).unwrap();
        let data = "x".repeat(100);
        writer.write_all(data.as_bytes()).unwrap();
        writer.flush().unwrap();
        drop(writer);

        let current = dir.join("t.log");
        let archives: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|s| s.starts_with("t.log."))
            .collect();
        assert!(
            !archives.is_empty(),
            "expected at least one archived log file"
        );
        assert!(current.exists());
    }

    #[test]
    fn rolling_writer_bounds_retained_files() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);
        let max_files = 3; // current + 2 archives
        let mut writer = RollingFileWriter::new(&dir, "t", 1, max_files).unwrap();
        for _ in 0..10 {
            let data = "x".repeat(10);
            writer.write_all(data.as_bytes()).unwrap();
            writer.flush().unwrap();
        }
        drop(writer);

        let archives: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|s| s.starts_with("t.log."))
            .collect();
        assert!(
            archives.len() < max_files,
            "archived files ({}) exceed bound ({})",
            archives.len(),
            max_files - 1
        );
    }

    #[test]
    fn today_is_monotonic() {
        let first = today();
        let second = today();
        assert!(second >= first);
    }

    #[test]
    fn level_display_matches_expected() {
        assert_eq!(Level::Error.to_string(), "ERROR");
        assert_eq!(Level::Info.to_string(), "INFO ");
    }
}
