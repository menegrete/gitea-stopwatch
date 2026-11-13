//! Structured logging: a [`tracing`] subscriber over a rolling log file.
//!
//! [`init`] installs one process-wide [`tracing`] subscriber and the panic
//! hook. The subscriber renders every event as a single line
//! (`<epoch-seconds> <LEVEL> [<target>] <key=value>... <message>`) written
//! through a [`RollingFileWriter`], which rotates on size and on day
//! boundaries and retains a bounded number of archived files.
//!
//! Verbosity is read once from the [`LOG_LEVEL_ENV`] environment variable
//! (`off`, `error`, `warn`, `info`, `debug`, `trace`). The default is `info`,
//! so a release build never defaults to `trace` and a support run can raise
//! verbosity without a rebuild.
//!
//! A log event records the event and its identifiers only. No credential,
//! token, personal data, or full file/network body is written here, and the
//! log file lives under the namespaced user-data directory, where it is safe
//! to delete at any time.

use std::fmt::Write as FmtWrite;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use tracing::field::{Field, Visit};
use tracing::level_filters::LevelFilter;
use tracing::metadata::Metadata;
use tracing::span;
use tracing::{Event, Level, Subscriber};

use crate::core::{Error, Result};

const LOG_PREFIX: &str = "gitea-stopwatch";
const LOG_EXTENSION: &str = ".log";
const MESSAGE_FIELD: &str = "message";

/// Environment variable that selects the log verbosity (default `info`).
pub const LOG_LEVEL_ENV: &str = "GITEA_STOPWATCH_LOG";

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

/// Five-column label rendered for a log level.
fn level_label(level: &Level) -> &'static str {
    match *level {
        Level::ERROR => "ERROR",
        Level::WARN => "WARN ",
        Level::INFO => "INFO ",
        Level::DEBUG => "DEBUG",
        Level::TRACE => "TRACE",
    }
}

/// Parses the documented verbosity names, accepting any letter case and
/// surrounding whitespace. An unrecognized value falls back to the default
/// when read through [`level_from_env`].
fn parse_level(raw: &str) -> Option<LevelFilter> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "off" => Some(LevelFilter::OFF),
        "error" => Some(LevelFilter::ERROR),
        "warn" | "warning" => Some(LevelFilter::WARN),
        "info" => Some(LevelFilter::INFO),
        "debug" => Some(LevelFilter::DEBUG),
        "trace" => Some(LevelFilter::TRACE),
        _ => None,
    }
}

/// Resolves the verbosity from [`LOG_LEVEL_ENV`], defaulting to `info`.
///
/// The variable is read once so every event in a process is filtered by the
/// same value even if the environment changes later.
fn level_from_env() -> LevelFilter {
    static LEVEL: OnceLock<LevelFilter> = OnceLock::new();
    *LEVEL.get_or_init(|| {
        std::env::var(LOG_LEVEL_ENV)
            .ok()
            .and_then(|raw| parse_level(&raw))
            .unwrap_or(LevelFilter::INFO)
    })
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

/// Process-wide rolling log writer, installed once by [`init`]. Writes are
/// synchronized so the subscriber and the panic hook can log concurrently.
static LOG_WRITER: OnceLock<Mutex<Option<RollingFileWriter>>> = OnceLock::new();

/// Collects an event's fields into the ordered pairs the log line renders.
///
/// The `message` field is rendered after the other fields on the same line, so
/// it is captured separately. Every other value is rendered with `Debug`, which
/// quotes string values (`phase="startup"`) and prints numbers bare.
#[derive(Default)]
struct FieldVisitor {
    message: Option<String>,
    fields: Vec<(String, String)>,
}

impl FieldVisitor {
    fn push(&mut self, field: &Field, rendered: String) {
        if field.name() == MESSAGE_FIELD {
            self.message = Some(rendered);
        } else {
            self.fields.push((field.name().to_string(), rendered));
        }
    }
}

impl Visit for FieldVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        // `format_args!` (the shape a message literal takes) renders through
        // `Debug` as its formatted text, so the message stays unquoted.
        self.push(field, format!("{value:?}"));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == MESSAGE_FIELD {
            self.message = Some(value.to_string());
        } else {
            self.push(field, format!("{value:?}"));
        }
    }
}

/// Renders one log line into the process-wide rolling log file.
///
/// Writes are best effort: a log call never turns into an application error,
/// and an uninitialized writer makes the call a no-op.
fn write_event(level: &Level, target: &str, fields: &[(String, String)], message: &str) {
    let Some(log_writer) = LOG_WRITER.get() else {
        return;
    };
    let Ok(mut guard) = log_writer.lock() else {
        return;
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
        let _ = write!(fields_out, " {key}={value}");
    }
    let line = format!(
        "{secs} {level} [{target}]{fields_out} {message}\n",
        level = level_label(level)
    );
    let _ = writer.write_all(line.as_bytes());
    let _ = writer.flush();
}

/// [`tracing`] subscriber that renders every event into the rolling log file.
///
/// Only events are rendered: spans are accepted and ignored, because the line
/// carries the event's own target and fields. Level filtering is fixed when the
/// subscriber is installed, from [`level_from_env`].
struct RollingFileSubscriber {
    level: LevelFilter,
}

impl RollingFileSubscriber {
    fn new(level: LevelFilter) -> Self {
        Self { level }
    }

    /// Whether an event at `level` is rendered. `LevelFilter` orders from most
    /// to least verbose, so a level at or below the filter is recorded.
    fn allows(&self, level: &Level) -> bool {
        *level <= self.level
    }
}

impl Subscriber for RollingFileSubscriber {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        self.allows(metadata.level())
    }

    fn max_level_hint(&self) -> Option<LevelFilter> {
        Some(self.level)
    }

    fn new_span(&self, _span: &span::Attributes<'_>) -> span::Id {
        // Events are the only records this subscriber renders; a fixed span
        // identifier satisfies the trace machinery without tracking parents.
        span::Id::from_u64(1)
    }

    fn record(&self, _span: &span::Id, _values: &span::Record<'_>) {}

    fn record_follows_from(&self, _span: &span::Id, _follows: &span::Id) {}

    fn enter(&self, _span: &span::Id) {}

    fn exit(&self, _span: &span::Id) {}

    fn event(&self, event: &Event<'_>) {
        let metadata = event.metadata();
        if !self.enabled(metadata) {
            return;
        }
        let mut visitor = FieldVisitor::default();
        event.record(&mut visitor);
        write_event(
            metadata.level(),
            metadata.target(),
            &visitor.fields,
            visitor.message.as_deref().unwrap_or_default(),
        );
    }
}

/// Installs the panic hook, which records the panic location and payload to the
/// rolling log before the process unwinds.
///
/// The hook writes to the writer directly instead of emitting a trace event, so
/// a panic raised while the subscriber is mid-write still leaves a record.
fn install_panic_hook() {
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
        write_event(
            &Level::ERROR,
            "panic",
            &[],
            &format!("panic occurred at {location}: {payload}"),
        );
    }));
}

/// Initialize logging: install the rolling log writer, the [`tracing`]
/// subscriber that renders events into it, and the panic hook.
///
/// Returns a typed error when the rolling log file cannot be created or when
/// logging is already installed for this process. A subscriber installed
/// earlier by an embedding application is not an error: the writer stays
/// available to the panic hook and the conflict is recorded.
pub fn init(log_dir: &Path) -> Result<()> {
    let writer = RollingFileWriter::new(log_dir, LOG_PREFIX, DEFAULT_MAX_SIZE, DEFAULT_MAX_FILES)?;
    LOG_WRITER
        .set(Mutex::new(Some(writer)))
        .map_err(|_| Error::Platform {
            operation: "install logging".to_string(),
        })?;

    install_panic_hook();

    let subscriber = RollingFileSubscriber::new(level_from_env());
    if let Err(error) = tracing::subscriber::set_global_default(subscriber) {
        write_event(
            &Level::WARN,
            "logging",
            &[],
            &format!("a tracing subscriber was already installed: {error}"),
        );
    }

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

        let _ = fs::remove_dir_all(&dir);
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

        let _ = fs::remove_dir_all(&dir);
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

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn today_is_monotonic() {
        let first = today();
        let second = today();
        assert!(second >= first);
    }

    #[test]
    fn level_label_is_five_columns_wide() {
        let rendered: Vec<&str> = [
            Level::ERROR,
            Level::WARN,
            Level::INFO,
            Level::DEBUG,
            Level::TRACE,
        ]
        .iter()
        .map(level_label)
        .collect();
        assert_eq!(rendered, vec!["ERROR", "WARN ", "INFO ", "DEBUG", "TRACE"]);
        assert!(
            rendered.iter().all(|label| label.len() == 5),
            "every log level occupies the same five columns"
        );
    }

    #[test]
    fn parse_level_accepts_the_documented_names() {
        assert_eq!(parse_level("off"), Some(LevelFilter::OFF));
        assert_eq!(parse_level("ERROR"), Some(LevelFilter::ERROR));
        assert_eq!(parse_level(" warn "), Some(LevelFilter::WARN));
        assert_eq!(parse_level("warning"), Some(LevelFilter::WARN));
        assert_eq!(parse_level("Info"), Some(LevelFilter::INFO));
        assert_eq!(parse_level("debug"), Some(LevelFilter::DEBUG));
        assert_eq!(parse_level("trace"), Some(LevelFilter::TRACE));
    }

    #[test]
    fn parse_level_rejects_unknown_names() {
        assert_eq!(parse_level("verbose"), None);
        assert_eq!(parse_level(""), None);
    }

    #[test]
    fn level_filter_defaults_to_info_and_keeps_error_enabled() {
        // The documented default is `info`: raising or lowering it is the
        // environment variable's job, and a release build never starts at
        // `trace`.
        assert_eq!(level_from_env(), LevelFilter::INFO);

        let subscriber = RollingFileSubscriber::new(LevelFilter::INFO);
        assert!(subscriber.allows(&Level::ERROR));
        assert!(subscriber.allows(&Level::WARN));
        assert!(subscriber.allows(&Level::INFO));
        assert!(!subscriber.allows(&Level::DEBUG));
        assert!(!subscriber.allows(&Level::TRACE));
    }

    #[test]
    fn rotate_without_existing_current_file_reopens_it() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);
        let mut writer = RollingFileWriter::new(&dir, "t", 10, 3).unwrap();
        // Remove the current file so rotation finds nothing to archive.
        fs::remove_file(dir.join("t.log")).unwrap();

        writer.write_all(b"0123456789ABCDEF").unwrap();
        writer.flush().unwrap();
        drop(writer);

        assert!(dir.join("t.log").exists());
        let archives: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|s| s.starts_with("t.log."))
            .collect();
        assert!(
            archives.is_empty(),
            "nothing to archive when the current file was removed"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn rotation_prunes_archives_beyond_the_retention_bound() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        // Pre-seed sortable archives older than the retention bound.
        for name in ["t.log.aaa", "t.log.bbb", "t.log.ccc", "t.log.ddd"] {
            fs::write(dir.join(name), "old").unwrap();
        }
        let mut writer = RollingFileWriter::new(&dir, "t", 5, 3).unwrap();

        writer.write_all(b"0123456789").unwrap();
        writer.flush().unwrap();
        drop(writer);

        let archives: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|s| s.starts_with("t.log."))
            .collect();
        assert_eq!(
            archives.len(),
            2,
            "retention keeps max_files - 1 archives, got {:?}",
            archives
        );

        let _ = fs::remove_dir_all(&dir);
    }

    /// Installs the global subscriber and the panic hook for the test process.
    ///
    /// This is the only test that calls [`init`] so the "already installed"
    /// branch is exercised deterministically by the second call, and so the
    /// `tracing` macros below have a subscriber to write to.
    #[test]
    fn init_installs_logger_once_and_panic_hook_writes_to_log() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);

        init(&dir).expect("first init installs the global logger");
        let rejected_dir = tmp_dir();
        assert!(
            init(&rejected_dir).is_err(),
            "a second init must report the already-installed logger"
        );
        let _ = fs::remove_dir_all(&rejected_dir);

        // A `warn!` event carries structured fields; an `info!` event proves
        // the default filter admits the level the crate logs at.
        tracing::warn!(target: "logging-test", attempt = "1", "structured line");
        tracing::info!(target: "logging-test", phase = "startup", "facade event");
        // A `debug!` event is below the default `info` filter and is dropped by
        // the subscriber's level check.
        tracing::debug!(target: "logging-test", "filtered event");
        // Spans are accepted by the subscriber even though only events render:
        // creating, following, recording and entering one must stay a no-op
        // rather than fail.
        let operation = tracing::info_span!("test-operation");
        let follow_up = tracing::info_span!("test-follow-up");
        follow_up.follows_from(operation.id());
        follow_up.record("answered", 42);
        {
            let _entered = operation.enter();
        }

        let borrowed = std::panic::catch_unwind(|| panic!("boom"));
        assert!(borrowed.is_err(), "the panicking closure must unwind");

        let owned = std::panic::catch_unwind(|| panic!("owned {}", 1));
        assert!(owned.is_err(), "the formatting closure must unwind");

        let non_string = std::panic::catch_unwind(|| std::panic::panic_any(42_u8));
        assert!(non_string.is_err(), "panic_any must unwind");

        let contents = fs::read_to_string(dir.join(format!("{}{}", LOG_PREFIX, LOG_EXTENSION)))
            .expect("rolling log file should exist");
        assert!(contents.contains("structured line"));
        assert!(contents.contains("attempt=\"1\""));
        assert!(contents.contains("facade event"));
        assert!(contents.contains("phase=\"startup\""));
        assert!(contents.contains("[logging-test]"));
        assert!(contents.contains("INFO "));
        let facade_line = contents
            .lines()
            .find(|line| line.contains("facade event"))
            .expect("the info event must be recorded");
        assert!(
            facade_line.contains(" INFO  [logging-test] phase=\"startup\" facade event"),
            "the event must render level, target, fields and message, got: {facade_line}"
        );
        assert!(
            !contents.contains("filtered event"),
            "the default level filter must drop debug events, got:\n{contents}"
        );
        assert!(contents.contains("panic occurred at"));
        assert!(contents.contains("boom"));
        assert!(contents.contains("owned 1"));
        assert!(contents.contains("<non-string panic payload>"));

        let _ = fs::remove_dir_all(&dir);
    }
}
