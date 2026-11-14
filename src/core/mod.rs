//! Application logic: pure core with no I/O or framework types in signatures.

mod config;
mod error;

pub use config::{
    Geometry, clamp_to_monitor, config_path_in, config_path_in_user_data_dir, load_geometry_at,
    save_geometry_at,
};
pub use error::{Error, Result};
