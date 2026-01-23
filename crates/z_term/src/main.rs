//! zTerm - Main Application Entry
//!
//! A modern cross-platform terminal emulator built with Rust and GPUI.

use anyhow::Result;
use gpui::*;
use tracing::info;

mod app;
mod settings;
mod window;
mod workspace;

use app::ZTermApp;

fn main() -> Result<()> {
    // Initialize logging
    zterm_common::logging::init()?;

    info!("Starting zTerm");

    // Initialize configuration
    if let Err(e) = zterm_common::Config::init() {
        tracing::warn!("Failed to load config, using defaults: {}", e);
    }

    // Create and run GPUI application
    let app = Application::new();

    app.run(|cx| {
        // Set up application
        ZTermApp::init(cx);

        // Open main window
        ZTermApp::open_main_window(cx);
    });

    Ok(())
}
