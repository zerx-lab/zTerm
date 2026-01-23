//! Axon Terminal - Common Utilities
//!
//! This crate provides shared utilities, configuration management,
//! and logging infrastructure for the Axon Terminal application.

pub mod config;
pub mod error;
pub mod logging;

pub use config::Config;
pub use error::{Error, Result};
