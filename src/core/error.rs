//! Typed domain errors shared by the core, platform, and interface layers.

use thiserror::Error;

/// Typed domain errors for the gitea-stopwatch crate.
///
/// Each variant carries the identifiers a reader needs to diagnose it (path,
/// operation) and deliberately carries no credential, raw pointer, or full file
/// body.
#[derive(Error, Debug)]
pub enum Error {
    /// The configuration file exists but could not be read.
    #[error("failed to read configuration at {path}: {source}")]
    ConfigRead {
        /// Absolute path of the configuration file that failed to read.
        path: String,
        /// Underlying I/O failure that prevented the read.
        source: std::io::Error,
    },
    /// The configuration file was read but its contents could not be parsed.
    #[error("failed to parse configuration at {path}: {message}")]
    ConfigParse {
        /// Absolute path of the configuration file that failed to parse.
        path: String,
        /// Parser diagnostic describing the malformed content.
        message: String,
    },
    /// The platform user data directory could not be resolved.
    #[error("failed to resolve the user data directory")]
    UserDataDir,
    /// The platform cache directory could not be resolved.
    #[error("failed to resolve the cache directory")]
    CacheDir,
    /// Logging could not be initialized at the resolved path.
    #[error("failed to initialize logging at {path}: {source}")]
    LoggingInit {
        /// Absolute path of the log directory or file that failed.
        path: String,
        /// Underlying I/O failure that prevented initialization.
        source: std::io::Error,
    },
    /// A platform operation failed; the operation name identifies which one.
    #[error("platform operation '{operation}' failed")]
    Platform {
        /// Name of the platform operation that failed.
        operation: String,
    },
    /// An I/O operation failed outside a more specific variant.
    #[error("I/O failure: {source}")]
    Io {
        /// Underlying I/O failure.
        #[from]
        source: std::io::Error,
    },
}

/// Convenience result type used across the library.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_read_carries_path_and_source() {
        let source = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
        let err = Error::ConfigRead {
            path: "/config/app.toml".to_string(),
            source,
        };
        assert_eq!(
            err.to_string(),
            "failed to read configuration at /config/app.toml: missing"
        );
    }

    #[test]
    fn config_parse_carries_path_and_message() {
        let err = Error::ConfigParse {
            path: "/config/app.toml".to_string(),
            message: "expected a table".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "failed to parse configuration at /config/app.toml: expected a table"
        );
    }

    #[test]
    fn user_data_dir_and_cache_dir_have_stable_messages() {
        assert_eq!(
            Error::UserDataDir.to_string(),
            "failed to resolve the user data directory"
        );
        assert_eq!(
            Error::CacheDir.to_string(),
            "failed to resolve the cache directory"
        );
    }

    #[test]
    fn logging_init_carries_path_and_source() {
        let source = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let err = Error::LoggingInit {
            path: "/data/logs/app.log".to_string(),
            source,
        };
        assert_eq!(
            err.to_string(),
            "failed to initialize logging at /data/logs/app.log: denied"
        );
    }

    #[test]
    fn platform_carries_operation() {
        let err = Error::Platform {
            operation: "create_dir_all".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "platform operation 'create_dir_all' failed"
        );
    }

    #[test]
    fn io_error_converts_via_from_and_keeps_source_message() {
        let io = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "pipe closed");
        let err: Error = io.into();
        assert!(matches!(err, Error::Io { .. }));
        assert_eq!(err.to_string(), "I/O failure: pipe closed");
    }

    #[test]
    fn io_error_source_is_preserved_for_diagnosis() {
        let io = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let err = Error::Io { source: io };
        let source = std::error::Error::source(&err).expect("Io variant exposes its source");
        assert_eq!(source.to_string(), "denied");
    }
}
