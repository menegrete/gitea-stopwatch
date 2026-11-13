//! End-to-end tests for the gitea-stopwatch desktop shell.
//!
//! These tests run against an isolated user-data directory using the
//! `GITEA_STOPWATCH_DATA_DIR` and `GITEA_STOPWATCH_CACHE_DIR` environment
//! overrides (see `src/platform/dirs.rs`), so they never touch the
//! developer's real application data. Each test cleans up its isolated
//! directories and restores the environment after execution.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use gitea_stopwatch::platform::dirs;
use gitea_stopwatch::ui::window::APP_TITLE;

/// Build a unique isolated temp directory path for this test run.
fn isolated_dir(suffix: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    env::temp_dir().join(format!(
        "gitea-stopwatch-e2e-{}-{}-{}",
        std::process::id(),
        nanos,
        suffix
    ))
}

/// Point the app's env overrides at isolated directories, returning the
/// previous values so the caller can restore them.
fn override_env(data_dir: &Path, cache_dir: &Path) -> (Option<String>, Option<String>) {
    let prev_data = env::var("GITEA_STOPWATCH_DATA_DIR").ok();
    let prev_cache = env::var("GITEA_STOPWATCH_CACHE_DIR").ok();
    unsafe {
        env::set_var("GITEA_STOPWATCH_DATA_DIR", data_dir);
        env::set_var("GITEA_STOPWATCH_CACHE_DIR", cache_dir);
    }
    (prev_data, prev_cache)
}

/// Restore previously saved env override values (or remove them).
fn restore_env(prev: (Option<String>, Option<String>)) {
    match prev.0 {
        Some(v) => unsafe {
            env::set_var("GITEA_STOPWATCH_DATA_DIR", v);
        },
        None => unsafe {
            env::remove_var("GITEA_STOPWATCH_DATA_DIR");
        },
    }
    match prev.1 {
        Some(v) => unsafe {
            env::set_var("GITEA_STOPWATCH_CACHE_DIR", v);
        },
        None => unsafe {
            env::remove_var("GITEA_STOPWATCH_CACHE_DIR");
        },
    }
}

#[test]
fn e2e_uses_isolated_user_data_and_cleans_up() {
    let data_dir = isolated_dir("data");
    let cache_dir = isolated_dir("cache");

    // Arrange: isolated env overrides
    let prev = override_env(&data_dir, &cache_dir);

    // Act: resolve user data and cache directories through the app's platform
    // abstraction, which honours the env overrides.
    let resolved_data = dirs::user_data_dir().expect("user data dir should resolve");
    let resolved_cache = dirs::cache_dir().expect("cache dir should resolve");

    // Assert: directories live under the isolated temp locations, never the
    // real application data directory.
    assert!(
        resolved_data.starts_with(&data_dir),
        "user data dir should be under isolated data dir, got {:?}",
        resolved_data
    );
    assert!(
        resolved_data.ends_with("gitea-stopwatch"),
        "user data dir should be namespaced by app name"
    );
    assert!(
        resolved_cache.starts_with(&cache_dir),
        "cache dir should be under isolated cache dir, got {:?}",
        resolved_cache
    );

    // Assert: window title is present (title presence check).
    assert!(
        !APP_TITLE.is_empty(),
        "window title (APP_TITLE) must be present"
    );
    assert_eq!(
        APP_TITLE, "gitea-stopwatch",
        "window title should match the app's advertised title"
    );

    // Cleanup: remove isolated directories and restore the environment.
    restore_env(prev);
    let _ = fs::remove_dir_all(&data_dir);
    let _ = fs::remove_dir_all(&cache_dir);

    // Assert: env overrides are restored to their previous state.
    assert!(
        env::var("GITEA_STOPWATCH_DATA_DIR").is_err()
            || env::var("GITEA_STOPWATCH_DATA_DIR").unwrap() != data_dir.to_string_lossy(),
        "GITEA_STOPWATCH_DATA_DIR should not point to the isolated dir after cleanup"
    );
    assert!(
        env::var("GITEA_STOPWATCH_CACHE_DIR").is_err()
            || env::var("GITEA_STOPWATCH_CACHE_DIR").unwrap() != cache_dir.to_string_lossy(),
        "GITEA_STOPWATCH_CACHE_DIR should not point to the isolated dir after cleanup"
    );
}

#[test]
fn e2e_window_title_is_present() {
    assert!(!APP_TITLE.is_empty(), "window title must not be empty");
    assert_eq!(APP_TITLE, "gitea-stopwatch");
}
