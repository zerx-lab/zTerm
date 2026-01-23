//! Axon Terminal - Main Application Entry
//!
//! A modern cross-platform terminal emulator built with Rust and GPUI.

use anyhow::Result;
use gpui::*;
use tracing::info;

mod app;
mod settings;
mod window;
mod workspace;

use app::AxonApp;

fn main() -> Result<()> {
    // Initialize logging
    axon_common::logging::init()?;

    info!("Starting Axon Terminal");

    // Initialize configuration
    if let Err(e) = axon_common::Config::init() {
        tracing::warn!("Failed to load config, using defaults: {}", e);
    }

    // Create and run GPUI application
    let app = Application::new();

    app.run(|cx| {
        // Set up application
        AxonApp::init(cx);

        // Open main window
        AxonApp::open_main_window(cx);
    });

    Ok(())
}
