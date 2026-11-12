//! Application logic: pure core with no I/O or framework types in signatures.

mod config;
mod error;

pub use config::{Geometry, clamp_to_monitor, load_geometry, save_geometry};
pub use error::{Error, Result};
