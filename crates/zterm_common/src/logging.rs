//! Logging infrastructure for zTerm
//!
//! This module provides logging functionality with both console and file output.
//! Logs are saved to the same directory as the configuration file.

use crate::Result;
use crate::config::Config;
use parking_lot::RwLock;
use std::path::PathBuf;
use std::time::Instant;
use tracing::info;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// Global startup timer for measuring application startup time
static STARTUP_TIMER: RwLock<Option<StartupTimer>> = RwLock::new(None);

/// Startup timer for measuring various initialization phases
#[derive(Debug, Clone)]
pub struct StartupTimer {
    /// Application start time
    start: Instant,
    /// Phase markers
    phases: Vec<(String, Instant)>,
}

impl StartupTimer {
    /// Create a new startup timer
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            phases: Vec::new(),
        }
    }

    /// Mark a phase completion
    pub fn mark(&mut self, phase: &str) {
        self.phases.push((phase.to_string(), Instant::now()));
    }

    /// Get elapsed time since start in milliseconds
    pub fn elapsed_ms(&self) -> u128 {
        self.start.elapsed().as_millis()
    }

    /// Get elapsed time for a specific phase
    pub fn phase_elapsed_ms(&self, phase: &str) -> Option<u128> {
        let mut prev_time = self.start;
        for (name, time) in &self.phases {
            if name == phase {
                return Some(time.duration_since(prev_time).as_millis());
            }
            prev_time = *time;
        }
        None
    }

    /// Log all phases
    pub fn log_phases(&self) {
        info!(
            "=== Startup Performance ===\n  Total time: {}ms",
            self.elapsed_ms()
        );
        let mut prev_time = self.start;
        for (name, time) in &self.phases {
            let phase_duration = time.duration_since(prev_time).as_millis();
            info!("  {} - {}ms", name, phase_duration);
            prev_time = *time;
        }
    }
}

impl Default for StartupTimer {
    fn default() -> Self {
        Self::new()
    }
}

/// Start the global startup timer
///
/// Call this at the very beginning of main() to measure total startup time.
pub fn start_timer() {
    *STARTUP_TIMER.write() = Some(StartupTimer::new());
}

/// Mark a phase completion in the global startup timer
pub fn mark_phase(phase: &str) {
    if let Some(timer) = STARTUP_TIMER.write().as_mut() {
        timer.mark(phase);
    }
}

/// Log all phases from the global startup timer
pub fn log_startup_phases() {
    if let Some(timer) = STARTUP_TIMER.read().as_ref() {
        timer.log_phases();
    }
}

/// Get the elapsed time since startup in milliseconds
pub fn startup_elapsed_ms() -> u128 {
    STARTUP_TIMER
        .read()
        .as_ref()
        .map(|t| t.elapsed_ms())
        .unwrap_or(0)
}

/// Get the log directory path (same as config directory)
pub fn log_dir() -> Option<PathBuf> {
    Config::config_dir().map(|p| p.join("logs"))
}

/// Get the current log file path
pub fn log_file() -> Option<PathBuf> {
    log_dir().map(|p| {
        let date = chrono::Local::now().format("%Y-%m-%d");
        p.join(format!("zterm.{}.log", date))
    })
}

/// Log file guard - keeps the non-blocking writer alive
///
/// This must be kept alive for the duration of the application to ensure
/// logs are properly flushed to the file.
pub struct LogGuard {
    _file_guard: Option<WorkerGuard>,
}

/// Initialize the logging system with both console and file output
///
/// Returns a LogGuard that must be kept alive for the duration of the application.
pub fn init() -> Result<LogGuard> {
    // Create log directory if it doesn't exist
    let file_guard = if let Some(log_path) = log_dir() {
        std::fs::create_dir_all(&log_path)?;

        // Create a file appender that rotates daily
        let file_appender = tracing_appender::rolling::daily(&log_path, "zterm.log");
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

        // Set up the subscriber with both console and file layers
        // Filter out GPUI window-related errors that occur during window close
        // These are harmless race conditions in GPUI's async window handling
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            EnvFilter::new("info,gpui=warn,gpui::window=off,gpui::platform::windows::window=off")
        });

        tracing_subscriber::registry()
            .with(filter)
            // Console layer with pretty formatting
            .with(
                fmt::layer()
                    .with_target(true)
                    .with_thread_ids(false)
                    .with_file(false)
                    .with_line_number(false),
            )
            // File layer with more detailed formatting
            .with(
                fmt::layer()
                    .with_writer(non_blocking)
                    .with_ansi(false)
                    .with_target(true)
                    .with_thread_ids(true)
                    .with_file(true)
                    .with_line_number(true),
            )
            .init();

        Some(guard)
    } else {
        // Fallback to console-only logging if we can't determine log directory
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            EnvFilter::new("info,gpui=warn,gpui::window=off,gpui::platform::windows::window=off")
        });

        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().with_target(true))
            .init();

        None
    };

    Ok(LogGuard {
        _file_guard: file_guard,
    })
}

/// Initialize the logging system with only console output (no file logging)
///
/// Useful for development or when file access is not needed.
pub fn init_console_only() -> Result<LogGuard> {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug,gpui=warn"));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(true))
        .init();

    Ok(LogGuard { _file_guard: None })
}

/// Initialize logging for tests
#[cfg(test)]
pub fn init_test() {
    let _ = tracing_subscriber::fmt()
        .with_test_writer()
        .with_env_filter("debug")
        .try_init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_startup_timer() {
        let mut timer = StartupTimer::new();
        std::thread::sleep(std::time::Duration::from_millis(10));
        timer.mark("phase1");
        std::thread::sleep(std::time::Duration::from_millis(10));
        timer.mark("phase2");

        assert!(timer.elapsed_ms() >= 20);
        assert!(timer.phase_elapsed_ms("phase1").is_some());
        assert!(timer.phase_elapsed_ms("phase2").is_some());
    }

    #[test]
    fn test_log_dir() {
        let dir = log_dir();
        // Should end with "zterm/logs"
        if let Some(d) = dir {
            assert!(d.ends_with("zterm/logs") || d.ends_with("zterm\\logs"));
        }
    }

    #[test]
    fn test_log_file() {
        let file = log_file();
        if let Some(f) = file {
            let filename = f.file_name().unwrap().to_string_lossy();
            assert!(filename.starts_with("zterm."));
            assert!(filename.ends_with(".log"));
        }
    }

    #[test]
    fn test_global_timer() {
        start_timer();
        std::thread::sleep(std::time::Duration::from_millis(5));
        mark_phase("test_phase");

        let elapsed = startup_elapsed_ms();
        assert!(elapsed >= 5);
    }
}
