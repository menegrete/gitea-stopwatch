use std::path::{Path, PathBuf};

use crate::core::{Error, Result};

const APP_NAME: &str = "gitea-stopwatch";

/// Environment variable overriding the user data directory (for tests).
const DATA_DIR_ENV: &str = "GITEA_STOPWATCH_DATA_DIR";
/// Environment variable overriding the cache directory (for tests).
const CACHE_DIR_ENV: &str = "GITEA_STOPWATCH_CACHE_DIR";

/// Resolve the namespaced user data directory from an explicit base directory.
///
/// Pure and deterministic: no environment or file-system access, so it can be
/// tested without touching the developer's real application data.
pub fn user_data_dir_from(base: &Path) -> PathBuf {
    base.join(APP_NAME)
}

/// Resolve the namespaced cache directory from an explicit base directory.
pub fn cache_dir_from(base: &Path) -> PathBuf {
    base.join(APP_NAME)
}

/// Resolve the user data directory, preferring the `GITEA_STOPWATCH_DATA_DIR`
/// environment variable and falling back to the platform data directory
/// (`dirs::data_dir`), namespaced by the application name.
pub fn user_data_dir() -> Result<PathBuf> {
    let base = std::env::var(DATA_DIR_ENV)
        .ok()
        .map(PathBuf::from)
        .or_else(dirs::data_dir)
        .ok_or(Error::UserDataDir)?;
    Ok(user_data_dir_from(&base))
}

/// Resolve the cache directory, preferring the `GITEA_STOPWATCH_CACHE_DIR`
/// environment variable and falling back to the platform cache directory
/// (`dirs::cache_dir`), namespaced by the application name.
pub fn cache_dir() -> Result<PathBuf> {
    let base = std::env::var(CACHE_DIR_ENV)
        .ok()
        .map(PathBuf::from)
        .or_else(dirs::cache_dir)
        .ok_or(Error::CacheDir)?;
    Ok(cache_dir_from(&base))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_data_dir_from_namespaces_base() {
        assert_eq!(
            user_data_dir_from(Path::new("/var/lib")),
            PathBuf::from("/var/lib/gitea-stopwatch")
        );
    }

    #[test]
    fn cache_dir_from_namespaces_base() {
        assert_eq!(
            cache_dir_from(Path::new("/var/cache")),
            PathBuf::from("/var/cache/gitea-stopwatch")
        );
    }

    #[test]
    fn user_data_dir_returns_namespaced_path() {
        let dir = user_data_dir().unwrap();
        assert!(dir.ends_with("gitea-stopwatch"));
    }

    #[test]
    fn cache_dir_returns_namespaced_path() {
        let dir = cache_dir().unwrap();
        assert!(dir.ends_with("gitea-stopwatch"));
    }

    #[test]
    fn namespacing_helpers_are_independent_of_the_base_directory() {
        // Both helpers apply the same app namespace while preserving the base,
        // so an override base always maps to a distinct namespaced directory.
        let data_base = Path::new("/tmp/override-data");
        let cache_base = Path::new("/tmp/override-cache");
        assert_eq!(user_data_dir_from(data_base), data_base.join(APP_NAME));
        assert_eq!(cache_dir_from(cache_base), cache_base.join(APP_NAME));
        assert_ne!(user_data_dir_from(data_base), cache_dir_from(cache_base));
    }

    #[test]
    fn user_data_dir_from_accepts_relative_bases() {
        assert_eq!(
            user_data_dir_from(Path::new("relative")),
            PathBuf::from("relative").join(APP_NAME)
        );
    }
}
