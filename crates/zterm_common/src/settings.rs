//! Application settings with hot-reload support
//!
//! This module provides a GPUI Global for application settings that automatically
//! reloads when the configuration file changes.

use crate::config::Config;
use gpui::{App, Global};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use parking_lot::RwLock;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};
use tracing::{error, info, warn};

/// Message sent from file watcher to main thread
#[derive(Debug, Clone)]
pub enum SettingsMessage {
    /// Configuration file has changed
    ConfigChanged,
}

/// Application settings as a GPUI Global
///
/// This struct holds the current configuration and manages file watching
/// for hot-reload functionality.
pub struct AppSettings {
    /// Current configuration (shared with watcher thread)
    config: Arc<RwLock<Config>>,

    /// File watcher (kept alive to maintain watching)
    #[allow(dead_code)]
    watcher: Option<RecommendedWatcher>,

    /// Receiver for settings messages from watcher thread
    #[allow(dead_code)]
    message_rx: Option<Receiver<SettingsMessage>>,

    /// Config change counter - increments each time config is reloaded
    pub change_counter: u64,
}

impl Global for AppSettings {}

impl AppSettings {
    /// Create new AppSettings with default config
    fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(Config::default())),
            watcher: None,
            message_rx: None,
            change_counter: 0,
        }
    }

    /// Initialize the settings service
    ///
    /// This loads the configuration from disk and starts file watching.
    /// Call this in your `app.run()` callback.
    ///
    /// The initialization process:
    /// 1. Creates config file if it doesn't exist
    /// 2. Migrates outdated config (adds new fields, creates backup)
    /// 3. Starts file watcher for hot-reload
    pub fn init(cx: &mut App) {
        // Load and migrate configuration
        let (config, migration_result) = Config::load_and_migrate().unwrap_or_else(|e| {
            warn!("Failed to load/migrate config, using defaults: {}", e);
            (Config::default(), crate::config::MigrationResult::Created)
        });

        // Log migration result
        match &migration_result {
            crate::config::MigrationResult::UpToDate => {
                info!("Configuration is up to date");
            }
            crate::config::MigrationResult::Created => {
                info!("Created new configuration file");
            }
            crate::config::MigrationResult::Migrated { backup_path } => {
                info!("Configuration migrated, backup saved to {:?}", backup_path);
            }
        }

        // Update the legacy global config
        Config::set_global(config.clone());

        // Create settings instance
        let mut settings = AppSettings::new();
        *settings.config.write() = config;

        // Set up file watcher
        if let Some(config_file) = Config::config_file() {
            settings.setup_file_watcher(config_file, cx);
        }

        // Set as GPUI global
        cx.set_global(settings);

        info!("AppSettings initialized with hot-reload support");
    }

    /// Get a clone of the current configuration
    pub fn config(&self) -> Config {
        self.config.read().clone()
    }

    /// Get the global AppSettings instance
    pub fn get_global(cx: &App) -> &Self {
        cx.global::<Self>()
    }

    /// Get a clone of the current configuration from the global instance
    pub fn global_config(cx: &App) -> Config {
        Self::get_global(cx).config()
    }

    /// Set up file watcher for hot-reload
    fn setup_file_watcher(&mut self, config_file: PathBuf, cx: &mut App) {
        // Ensure config directory exists
        if let Some(parent) = config_file.parent() {
            if !parent.exists() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    warn!("Failed to create config directory: {}", e);
                    return;
                }
            }
        }

        // Create a sample config file if it doesn't exist
        if !config_file.exists() {
            let default_config = Config::default();
            if let Err(e) = default_config.save() {
                warn!("Failed to create default config file: {}", e);
            } else {
                info!("Created default config file at {:?}", config_file);
            }
        }

        // Create channel for communication between watcher thread and main thread
        let (tx, rx): (Sender<SettingsMessage>, Receiver<SettingsMessage>) = channel();
        let config_arc = Arc::clone(&self.config);

        // Create the watcher
        let watcher_result = notify::recommended_watcher(move |res: Result<Event, _>| {
            match res {
                Ok(event) => {
                    // Only react to modify events
                    if matches!(
                        event.kind,
                        notify::EventKind::Modify(_) | notify::EventKind::Create(_)
                    ) {
                        info!("Config file changed, reloading...");

                        // Reload configuration
                        match Config::load() {
                            Ok(new_config) => {
                                // Update the shared config
                                *config_arc.write() = new_config.clone();

                                // Update legacy global
                                Config::set_global(new_config);

                                // Notify main thread
                                if let Err(e) = tx.send(SettingsMessage::ConfigChanged) {
                                    error!("Failed to send config change notification: {}", e);
                                } else {
                                    info!("Config reloaded successfully");
                                }
                            }
                            Err(e) => {
                                error!("Failed to reload config: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("File watcher error: {}", e);
                }
            }
        });

        match watcher_result {
            Ok(mut watcher) => {
                // Watch the config directory (not just the file) for better cross-platform support
                if let Some(parent) = config_file.parent() {
                    if let Err(e) = watcher.watch(parent, RecursiveMode::NonRecursive) {
                        error!("Failed to watch config directory: {}", e);
                        return;
                    }
                    info!("Watching config directory: {:?}", parent);
                }

                self.watcher = Some(watcher);
                self.message_rx = Some(rx);

                // Set up periodic check for config changes on the main thread
                Self::setup_change_listener(cx);
            }
            Err(e) => {
                error!("Failed to create file watcher: {}", e);
            }
        }
    }

    /// Set up a periodic listener to check for configuration changes
    fn setup_change_listener(cx: &mut App) {
        // Spawn a background task that periodically checks for config changes
        // and triggers UI updates
        cx.spawn(async move |cx| {
            loop {
                // Sleep for a short duration
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(100))
                    .await;

                // Check if there are pending config changes and notify observers
                let should_notify = cx.update_global::<AppSettings, _>(|settings, _cx| {
                    // Try to receive a message without blocking
                    if let Some(rx) = &settings.message_rx {
                        match rx.try_recv() {
                            Ok(SettingsMessage::ConfigChanged) => true,
                            Err(std::sync::mpsc::TryRecvError::Empty) => false,
                            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                                warn!("Config watcher channel disconnected");
                                false
                            }
                        }
                    } else {
                        false
                    }
                });

                if should_notify {
                    info!("Config change processed on main thread");
                    // Increment the change counter to signal config update
                    cx.update_global::<AppSettings, _>(|settings, _cx| {
                        settings.change_counter += 1;
                        info!("Config change counter: {}", settings.change_counter);
                    });
                }
            }
        })
        .detach();
    }

    /// Create AppSettings with a specific config (for testing)
    #[cfg(test)]
    pub fn with_config(config: Config) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            watcher: None,
            message_rx: None,
            change_counter: 0,
        }
    }

    /// Update the configuration (for testing)
    #[cfg(test)]
    pub fn set_config(&self, config: Config) {
        *self.config.write() = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn test_app_settings_new() {
        let settings = AppSettings::new();
        let config = settings.config();

        // Should have default values
        assert_eq!(config.terminal.font_size, 14.0);
        assert_eq!(config.ui.theme, "dark");
        assert_eq!(config.terminal.scrollback_lines, 10000);
    }

    #[test]
    fn test_app_settings_with_config() {
        let mut custom_config = Config::default();
        custom_config.terminal.font_size = 18.0;
        custom_config.ui.theme = "dracula".to_string();

        let settings = AppSettings::with_config(custom_config);
        let config = settings.config();

        assert_eq!(config.terminal.font_size, 18.0);
        assert_eq!(config.ui.theme, "dracula");
    }

    #[test]
    fn test_app_settings_set_config() {
        let settings = AppSettings::new();

        // Verify initial state
        assert_eq!(settings.config().terminal.font_size, 14.0);

        // Update config
        let mut new_config = Config::default();
        new_config.terminal.font_size = 20.0;
        settings.set_config(new_config);

        // Verify update
        assert_eq!(settings.config().terminal.font_size, 20.0);
    }

    #[test]
    fn test_app_settings_config_clone() {
        let settings = AppSettings::new();
        let config1 = settings.config();
        let config2 = settings.config();

        // Both should be independent clones
        assert_eq!(config1.terminal.font_size, config2.terminal.font_size);
    }

    #[test]
    fn test_app_settings_thread_safety() {
        use std::thread;

        let settings = AppSettings::new();
        let config_arc = Arc::clone(&settings.config);

        // Read from another thread
        let handle = thread::spawn(move || {
            let config = config_arc.read();
            config.terminal.font_size
        });

        let font_size = handle.join().unwrap();
        assert_eq!(font_size, 14.0);
    }

    #[test]
    fn test_settings_message_enum() {
        // Verify SettingsMessage can be created and matched
        let msg = SettingsMessage::ConfigChanged;
        match msg {
            SettingsMessage::ConfigChanged => {
                // Expected
            }
        }
    }

    #[test]
    fn test_config_terminal_settings() {
        let config = Config::default();

        assert_eq!(config.terminal.font_family, "JetBrainsMono Nerd Font Mono");
        assert_eq!(config.terminal.font_size, 14.0);
        assert!(config.terminal.cursor_blink);
        assert_eq!(config.terminal.cursor_style, "block");
    }

    #[test]
    fn test_config_ui_settings() {
        let config = Config::default();

        assert_eq!(config.ui.theme, "dark");
        assert_eq!(config.ui.opacity, 1.0);
        assert!(config.ui.show_tab_bar);
        assert_eq!(config.ui.window_width, 1200);
        assert_eq!(config.ui.window_height, 800);
    }

    #[test]
    fn test_config_modification() {
        let mut config = Config::default();

        // Modify terminal settings
        config.terminal.font_size = 16.0;
        config.terminal.font_family = "Consolas".to_string();

        // Modify UI settings
        config.ui.theme = "nord".to_string();
        config.ui.opacity = 0.95;

        // Verify modifications
        assert_eq!(config.terminal.font_size, 16.0);
        assert_eq!(config.terminal.font_family, "Consolas");
        assert_eq!(config.ui.theme, "nord");
        assert_eq!(config.ui.opacity, 0.95);
    }
}
