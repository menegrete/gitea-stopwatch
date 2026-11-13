//! Binary entry point for the gitea-stopwatch desktop shell.
//!
//! `anyhow` is used only here, at the binary edge, to add context as errors
//! travel out to the process exit code. Logging (the `tracing` subscriber over
//! the rolling log file, and the panic hook) is initialized here before the
//! application runs.

#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(missing_debug_implementations)]
#![deny(rust_2018_idioms)]
#![deny(unused_must_use)]

use anyhow::Result;
use gitea_stopwatch::platform::{dirs, logging};
use gitea_stopwatch::run;

fn main() -> Result<()> {
    let log_dir = dirs::user_data_dir()?.join("logs");
    logging::init(&log_dir)?;

    tracing::info!("gitea-stopwatch starting");

    run().map_err(|err| anyhow::anyhow!("{err}"))
}
