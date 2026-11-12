use thiserror::Error;

/// Typed domain errors for the gitea-stopwatch crate.
///
/// Each variant carries the identifiers a reader needs to diagnose it (path,
/// operation) and deliberately carries no credential, raw pointer, or full file
/// body.
#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to read configuration at {path}: {source}")]
    ConfigRead {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to parse configuration at {path}: {message}")]
    ConfigParse { path: String, message: String },
    #[error("failed to resolve the user data directory")]
    UserDataDir,
    #[error("failed to resolve the cache directory")]
    CacheDir,
    #[error("failed to initialize logging at {path}: {source}")]
    LoggingInit {
        path: String,
        source: std::io::Error,
    },
    #[error("platform operation '{operation}' failed")]
    Platform { operation: String },
    #[error("I/O failure: {source}")]
    Io {
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
}
