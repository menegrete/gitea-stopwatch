//! Platform edge: filesystem, credential store, network, child processes.

pub mod dirs;
pub mod focus;
pub mod instance;
pub mod lock;
pub mod logging;

pub use dirs::{cache_dir, user_data_dir};
pub use focus::{FocusAction, decide_focus_action, request_focus, take_focus_request};
pub use instance::{LaunchRole, claim};
pub use lock::{LockOutcome, acquire, release, try_acquire};
pub use logging::init;
