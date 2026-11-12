//! Application configuration persistence.
//!
//! Window geometry (size and position) is persisted to a JSON config file in
//! the platform user data directory. On startup the geometry is restored and
//! clamped to the connected monitors; on exit it is saved again
//! (spec: window-geometry-persistence).

use serde::{Deserialize, Serialize};

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

/// Resolve the path to the configuration file.
fn config_path() -> Result<std::path::PathBuf> {
    Ok(dirs::user_data_dir()?.join(CONFIG_FILE_NAME))
}

/// Load persisted geometry from the configuration file.
///
/// Returns `Ok(None)` if no configuration file exists yet.
pub fn load_geometry() -> Result<Option<Geometry>> {
    let path = config_path()?;
    match std::fs::read_to_string(&path) {
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

/// Save geometry to the configuration file.
pub fn save_geometry(geometry: Geometry) -> Result<()> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let config = Config { geometry };
    let contents = serde_json::to_string_pretty(&config).map_err(|e| Error::ConfigParse {
        path: path.to_string_lossy().to_string(),
        message: e.to_string(),
    })?;
    std::fs::write(&path, contents).map_err(|e| Error::ConfigRead {
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
}
