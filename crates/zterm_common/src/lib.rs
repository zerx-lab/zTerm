//! zTerm - Common Utilities
//!
//! This crate provides shared utilities, configuration management,
//! and logging infrastructure for the zTerm application.

pub mod config;
pub mod error;
pub mod logging;
pub mod settings;

pub use config::{Config, MigrationResult, CONFIG_VERSION};
pub use error::{Error, Result};
pub use settings::AppSettings;
