//! Application configuration persistence.
//!
//! Window geometry (size and position) is persisted to a JSON config file in
//! the platform user data directory. On startup the geometry is restored and
//! clamped to the connected monitors; on exit it is saved again
//! (spec: window-geometry-persistence).

use serde::{Deserialize, Serialize};

use std::path::Path;

use crate::core::{Error, Result};
use crate::platform::dirs;

const CONFIG_FILE_NAME: &str = "gitea-stopwatch.json";

/// Default window size when no persisted geometry exists.
const DEFAULT_WIDTH: f64 = 800.0;
const DEFAULT_HEIGHT: f64 = 600.0;

/// Window geometry: size and position.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Geometry {
    /// Window width in logical pixels.
    pub width: f64,
    /// Window height in logical pixels.
    pub height: f64,
    /// Window X position in logical pixels.
    pub x: f64,
    /// Window Y position in logical pixels.
    pub y: f64,
}

impl Default for Geometry {
    fn default() -> Self {
        Self {
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            x: 100.0,
            y: 100.0,
        }
    }
}

/// The persisted configuration file contents.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Config {
    geometry: Geometry,
}

/// Resolve the configuration file path for an explicit user data base directory.
///
/// Composes the same platform namespacing and file name the application uses,
/// so a caller can address the real configuration location without mutating the
/// process environment. This is the single geometry path seam: tests pass an
/// isolated base directory instead of the developer's real application data.
pub fn config_path_in(base: &Path) -> std::path::PathBuf {
    dirs::user_data_dir_from(base).join(CONFIG_FILE_NAME)
}

/// Resolve the configuration file path inside an already-resolved user data
/// directory.
///
/// [`config_path_in`] namespaces a base directory first; this variant takes the
/// namespaced directory itself, which is what the application passes around for
/// its single-instance lock and focus channel. The UI edge builds the geometry
/// path from the injected directory with this function, so geometry is read and
/// written next to the lock and focus files rather than in a directory resolved
/// from the process environment.
pub fn config_path_in_user_data_dir(user_data_dir: &Path) -> std::path::PathBuf {
    user_data_dir.join(CONFIG_FILE_NAME)
}

/// Load persisted geometry from a specific configuration file.
///
/// The explicit path is the only way to reach geometry: there is deliberately
/// no environment-resolved variant, so no caller can accidentally read the
/// developer's real application data from a unit test.
pub fn load_geometry_at(path: &Path) -> Result<Option<Geometry>> {
    match std::fs::read_to_string(path) {
        Ok(contents) => {
            let config: Config =
                serde_json::from_str(&contents).map_err(|e| Error::ConfigParse {
                    path: path.to_string_lossy().to_string(),
                    message: e.to_string(),
                })?;
            Ok(Some(config.geometry))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(Error::ConfigRead {
            path: path.to_string_lossy().to_string(),
            source: e,
        }),
    }
}

/// Save geometry to a specific configuration file.
///
/// The explicit path is the only way geometry is written: there is
/// deliberately no environment-resolved variant, so no caller — in particular
/// no unit test — can accidentally write the developer's real application data.
pub fn save_geometry_at(path: &Path, geometry: Geometry) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let config = Config { geometry };
    let contents = serde_json::to_string_pretty(&config).map_err(|e| Error::ConfigParse {
        path: path.to_string_lossy().to_string(),
        message: e.to_string(),
    })?;
    std::fs::write(path, contents).map_err(|e| Error::ConfigRead {
        path: path.to_string_lossy().to_string(),
        source: e,
    })
}

/// Clamp geometry so the window fits within the monitor bounds.
///
/// If the monitor size is unknown, the geometry is returned unchanged.
pub fn clamp_to_monitor(geometry: Geometry, monitor_size: Option<(f64, f64)>) -> Geometry {
    let Some((monitor_width, monitor_height)) = monitor_size else {
        return geometry;
    };

    let mut clamped = geometry;

    // Clamp width and height to not exceed the monitor.
    clamped.width = clamped.width.min(monitor_width).max(100.0);
    clamped.height = clamped.height.min(monitor_height).max(100.0);

    // Clamp position so the window stays on screen.
    clamped.x = clamped
        .x
        .clamp(0.0, (monitor_width - clamped.width).max(0.0));
    clamped.y = clamped
        .y
        .clamp(0.0, (monitor_height - clamped.height).max(0.0));

    clamped
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    /// Monotonic counter for unique per-test temp directories (tests run in parallel).
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn tmp_dir() -> PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "gitea-stopwatch-config-test-{}-{}",
            std::process::id(),
            n
        ))
    }

    fn tmp_config_path() -> PathBuf {
        tmp_dir().join(CONFIG_FILE_NAME)
    }

    #[test]
    fn default_geometry_has_sensible_values() {
        let g = Geometry::default();
        assert!(g.width > 0.0);
        assert!(g.height > 0.0);
    }

    #[test]
    fn clamp_to_monitor_keeps_window_on_screen() {
        let geometry = Geometry {
            width: 2000.0,
            height: 2000.0,
            x: -100.0,
            y: -100.0,
        };
        let clamped = clamp_to_monitor(geometry, Some((1920.0, 1080.0)));
        assert!(clamped.x >= 0.0);
        assert!(clamped.y >= 0.0);
        assert!(clamped.x + clamped.width <= 1920.0);
        assert!(clamped.y + clamped.height <= 1080.0);
    }

    #[test]
    fn clamp_to_monitor_unknown_monitor_returns_unchanged() {
        let geometry = Geometry::default();
        assert_eq!(clamp_to_monitor(geometry, None), geometry);
    }

    #[test]
    fn clamp_to_monitor_small_window_stays_put() {
        let geometry = Geometry {
            width: 100.0,
            height: 100.0,
            x: 500.0,
            y: 500.0,
        };
        let clamped = clamp_to_monitor(geometry, Some((1920.0, 1080.0)));
        assert_eq!(clamped, geometry);
    }

    #[test]
    fn clamp_to_monitor_enforces_minimum_size() {
        let geometry = Geometry {
            width: 10.0,
            height: 20.0,
            x: 0.0,
            y: 0.0,
        };
        let clamped = clamp_to_monitor(geometry, Some((1920.0, 1080.0)));
        assert_eq!(clamped.width, 100.0);
        assert_eq!(clamped.height, 100.0);
    }

    #[test]
    fn clamp_to_monitor_tiny_monitor_keeps_minimum_size_visible() {
        // A monitor smaller than the minimum window size cannot satisfy both
        // constraints; the minimum size wins over the monitor bound.
        let geometry = Geometry {
            width: 50.0,
            height: 50.0,
            x: 0.0,
            y: 0.0,
        };
        let clamped = clamp_to_monitor(geometry, Some((50.0, 50.0)));
        assert_eq!(clamped.width, 100.0);
        assert_eq!(clamped.height, 100.0);
        assert_eq!(clamped.x, 0.0);
        assert_eq!(clamped.y, 0.0);
    }

    #[test]
    fn clamp_to_monitor_pulls_offscreen_position_back() {
        let geometry = Geometry {
            width: 800.0,
            height: 600.0,
            x: 5000.0,
            y: 5000.0,
        };
        let clamped = clamp_to_monitor(geometry, Some((1920.0, 1080.0)));
        assert_eq!(clamped.x, 1920.0 - 800.0);
        assert_eq!(clamped.y, 1080.0 - 600.0);
    }

    #[test]
    fn load_geometry_at_returns_none_when_file_is_missing() {
        let path = tmp_config_path();
        let _ = fs::remove_dir_all(path.parent().unwrap());
        assert_eq!(load_geometry_at(&path).unwrap(), None);
    }

    #[test]
    fn save_then_load_geometry_round_trips() {
        let path = tmp_config_path();
        let _ = fs::remove_dir_all(path.parent().unwrap());
        let geometry = Geometry {
            width: 1024.0,
            height: 768.0,
            x: 32.0,
            y: 64.0,
        };

        save_geometry_at(&path, geometry).unwrap();

        assert!(path.exists());
        assert_eq!(load_geometry_at(&path).unwrap(), Some(geometry));

        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn save_geometry_at_overwrites_existing_file() {
        let path = tmp_config_path();
        let _ = fs::remove_dir_all(path.parent().unwrap());
        let first = Geometry::default();
        let second = Geometry {
            width: 640.0,
            height: 480.0,
            x: 10.0,
            y: 20.0,
        };

        save_geometry_at(&path, first).unwrap();
        save_geometry_at(&path, second).unwrap();

        assert_eq!(load_geometry_at(&path).unwrap(), Some(second));

        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn load_geometry_at_reports_parse_error() {
        let path = tmp_config_path();
        let _ = fs::remove_dir_all(path.parent().unwrap());
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "this is not json").unwrap();

        let err = load_geometry_at(&path).unwrap_err();

        assert!(matches!(err, Error::ConfigParse { .. }));
        assert!(
            err.to_string()
                .contains(&path.to_string_lossy().to_string())
        );

        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn load_geometry_at_reports_read_error_for_non_notfound_failure() {
        let path = tmp_config_path();
        let _ = fs::remove_dir_all(path.parent().unwrap());
        // A directory where a file is expected fails with an error other than
        // NotFound, which must surface as ConfigRead rather than Ok(None).
        fs::create_dir_all(&path).unwrap();

        let err = load_geometry_at(&path).unwrap_err();

        assert!(matches!(err, Error::ConfigRead { .. }));

        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn save_geometry_at_reports_error_when_parent_is_a_file() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let blocking_file = dir.join("blocking");
        fs::write(&blocking_file, "not a directory").unwrap();
        let path = blocking_file.join(CONFIG_FILE_NAME);

        let err = save_geometry_at(&path, Geometry::default()).unwrap_err();

        assert!(matches!(err, Error::Io { .. }));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn save_geometry_at_reports_write_error_when_target_is_a_directory() {
        let dir = tmp_dir();
        let _ = fs::remove_dir_all(&dir);
        // The parent exists, so directory creation succeeds and the write
        // itself fails because the target path is a directory.
        let path = dir.join(CONFIG_FILE_NAME);
        fs::create_dir_all(&path).unwrap();

        let err = save_geometry_at(&path, Geometry::default()).unwrap_err();

        assert!(matches!(err, Error::ConfigRead { .. }));

        let _ = fs::remove_dir_all(&dir);
    }
}
