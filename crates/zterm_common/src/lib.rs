//! zTerm - Common Utilities
//!
//! This crate provides shared utilities, configuration management,
//! and logging infrastructure for the zTerm application.

pub mod config;
pub mod error;
pub mod logging;
pub mod settings;

pub use config::{Config, ConfigurableAction, KeybindingsConfig, MigrationResult, CONFIG_VERSION};
pub use error::{Error, Result};
pub use logging::{log_dir, log_file, log_startup_phases, mark_phase, start_timer, LogGuard};
pub use settings::AppSettings;
