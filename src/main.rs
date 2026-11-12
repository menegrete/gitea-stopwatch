//! Binary entry point for the gitea-stopwatch desktop shell.
//!
//! `anyhow` is used only here, at the binary edge, to add context as errors
//! travel out to the process exit code. Logging (rolling log file, panic
//! hook) is initialized here before the application runs.

use anyhow::Result;
use gitea_stopwatch::platform::logging::Level;
use gitea_stopwatch::platform::{dirs, logging};
use gitea_stopwatch::run;

fn main() -> Result<()> {
    let log_dir = dirs::user_data_dir()?.join("logs");
    logging::init(&log_dir)?;

    logging::log(Level::Info, module_path!(), &[], "gitea-stopwatch starting");

    run().map_err(|err| anyhow::anyhow!("{err}"))
}
