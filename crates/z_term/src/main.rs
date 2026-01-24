//! zTerm - Main Application Entry
//!
//! A modern cross-platform terminal emulator built with Rust and GPUI.

use anyhow::Result;
use gpui::*;
use tracing::info;
use zterm_common::AppSettings;

mod app;
mod settings;
mod window;
mod workspace;

use app::ZTermApp;

fn main() -> Result<()> {
    // Initialize logging
    zterm_common::logging::init()?;

    info!("Starting zTerm");

    // Create and run GPUI application
    let app = Application::new();

    app.run(|cx| {
        // Initialize settings with hot-reload support
        // This replaces the old Config::init() call
        AppSettings::init(cx);

        // Set up application
        ZTermApp::init(cx);

        // Open main window
        ZTermApp::open_main_window(cx);
    });

    Ok(())
}
