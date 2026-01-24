//! zTerm - Common Utilities
//!
//! This crate provides shared utilities, configuration management,
//! and logging infrastructure for the zTerm application.

pub mod config;
pub mod error;
pub mod logging;

pub use config::Config;
pub use error::{Error, Result};
