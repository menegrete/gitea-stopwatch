//! Platform edge: filesystem, credential store, network, child processes.

pub mod dirs;
pub mod logging;

pub use dirs::{cache_dir, user_data_dir};
pub use logging::init;
