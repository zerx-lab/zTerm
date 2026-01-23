//! Logging infrastructure for Axon Terminal

use crate::Result;
use tracing_subscriber::{
    fmt,
    prelude::*,
    EnvFilter,
};

/// Initialize the logging system
pub fn init() -> Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(true))
        .init();

    tracing::info!("Axon Terminal logging initialized");

    Ok(())
}

/// Initialize logging for tests
#[cfg(test)]
pub fn init_test() {
    let _ = tracing_subscriber::fmt()
        .with_test_writer()
        .with_env_filter("debug")
        .try_init();
}
