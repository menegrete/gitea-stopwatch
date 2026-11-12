//! Public API for the gitea-stopwatch desktop shell.

#![forbid(unsafe_code)]

pub mod core;
pub mod platform;
pub mod ui;

/// Run the gitea-stopwatch application.
///
/// Returns a typed [`core::Error`]; the binary edge converts it into an
/// `anyhow` error with added context and an exit code.
pub fn run() -> core::Result<()> {
    // Application runtime is wired here in later tasks.
    Ok(())
}
