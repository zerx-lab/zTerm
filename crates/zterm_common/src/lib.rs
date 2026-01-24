//! zTerm - Common Utilities
//!
//! This crate provides shared utilities, configuration management,
//! and logging infrastructure for the zTerm application.

pub mod config;
pub mod error;
pub mod logging;
pub mod settings;

pub use config::{CONFIG_VERSION, Config, ConfigurableAction, KeybindingsConfig, MigrationResult};
pub use error::{Error, Result};
pub use logging::{LogGuard, log_dir, log_file, log_startup_phases, mark_phase, start_timer};
pub use settings::AppSettings;
