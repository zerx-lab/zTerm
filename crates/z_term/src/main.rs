//! zTerm - Main Application Entry
//!
//! A modern cross-platform terminal emulator built with Rust and GPUI.

// Hide console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![recursion_limit = "256"]

use anyhow::Result;
use gpui::*;
use tracing::info;
use zterm_common::{AppSettings, LogGuard, log_startup_phases, mark_phase, start_timer};

mod app;
mod settings;
mod window;
mod workspace;

use app::ZTermApp;

fn main() -> Result<()> {
    // Start timing immediately
    start_timer();

    // Initialize logging (file + console)
    let _log_guard: LogGuard = zterm_common::logging::init()?;
    mark_phase("logging_init");

    info!("Starting zTerm");

    // Log the log file location
    if let Some(log_path) = zterm_common::log_file() {
        info!("Logging to: {:?}", log_path);
    }

    // Create GPUI application
    let app = Application::new();
    mark_phase("gpui_app_created");

    app.run(|cx| {
        mark_phase("gpui_run_start");

        // Initialize settings with hot-reload support
        AppSettings::init(cx);
        mark_phase("settings_init");

        // Set up application
        ZTermApp::init(cx);
        mark_phase("app_init");

        // Open main window
        ZTermApp::open_main_window(cx);
        mark_phase("window_opened");

        // Log startup performance
        log_startup_phases();

        info!("zTerm is ready");
    });

    Ok(())
}
