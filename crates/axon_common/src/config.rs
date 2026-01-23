//! Configuration management for Axon Terminal

use crate::{Error, Result};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Global configuration instance
static CONFIG: Lazy<RwLock<Config>> = Lazy::new(|| RwLock::new(Config::default()));

/// Main configuration structure for Axon Terminal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Terminal settings
    pub terminal: TerminalConfig,

    /// UI settings
    pub ui: UiConfig,

    /// Keybinding settings
    pub keybindings: KeybindingsConfig,
}

/// Terminal-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConfig {
    /// Default shell to use
    pub shell: Option<String>,

    /// Shell arguments
    pub shell_args: Vec<String>,

    /// Working directory
    pub working_directory: Option<PathBuf>,

    /// Scrollback buffer size (number of lines)
    pub scrollback_lines: usize,

    /// Enable bell sound
    pub bell_enabled: bool,

    /// Cursor style: "block", "underline", "bar"
    pub cursor_style: String,

    /// Cursor blink enabled
    pub cursor_blink: bool,

    /// Font family
    pub font_family: String,

    /// Font size in points
    pub font_size: f32,

    /// Line height multiplier
    pub line_height: f32,
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Theme name
    pub theme: String,

    /// Window opacity (0.0 - 1.0)
    pub opacity: f32,

    /// Show tab bar
    pub show_tab_bar: bool,

    /// Tab bar position: "top", "bottom"
    pub tab_bar_position: String,

    /// Window decorations enabled
    pub decorations: bool,

    /// Initial window width
    pub window_width: u32,

    /// Initial window height
    pub window_height: u32,
}

/// Keybindings configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingsConfig {
    /// Copy shortcut
    pub copy: String,

    /// Paste shortcut
    pub paste: String,

    /// New tab shortcut
    pub new_tab: String,

    /// Close tab shortcut
    pub close_tab: String,

    /// Next tab shortcut
    pub next_tab: String,

    /// Previous tab shortcut
    pub prev_tab: String,

    /// Split horizontal shortcut
    pub split_horizontal: String,

    /// Split vertical shortcut
    pub split_vertical: String,

    /// Search shortcut
    pub search: String,

    /// Command palette shortcut
    pub command_palette: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            terminal: TerminalConfig::default(),
            ui: UiConfig::default(),
            keybindings: KeybindingsConfig::default(),
        }
    }
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            shell: None, // Auto-detect
            shell_args: vec![],
            working_directory: None,
            scrollback_lines: 10000,
            bell_enabled: true,
            cursor_style: "block".to_string(),
            cursor_blink: true,
            font_family: "JetBrainsMono Nerd Font Mono".to_string(),
            font_size: 14.0,
            line_height: 1.2,
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            opacity: 1.0,
            show_tab_bar: true,
            tab_bar_position: "top".to_string(),
            decorations: true,
            window_width: 1200,
            window_height: 800,
        }
    }
}

impl Default for KeybindingsConfig {
    fn default() -> Self {
        let modifier = if cfg!(target_os = "macos") {
            "cmd"
        } else {
            "ctrl"
        };

        Self {
            copy: format!("{modifier}+c"),
            paste: format!("{modifier}+v"),
            new_tab: format!("{modifier}+t"),
            close_tab: format!("{modifier}+w"),
            next_tab: format!("{modifier}+tab"),
            prev_tab: format!("{modifier}+shift+tab"),
            split_horizontal: format!("{modifier}+d"),
            split_vertical: format!("{modifier}+shift+d"),
            search: format!("{modifier}+f"),
            command_palette: format!("{modifier}+shift+p"),
        }
    }
}

impl Config {
    /// Get the configuration directory path
    pub fn config_dir() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("axon_term"))
    }

    /// Get the configuration file path
    pub fn config_file() -> Option<PathBuf> {
        Self::config_dir().map(|p| p.join("config.toml"))
    }

    /// Load configuration from file
    pub fn load() -> Result<Self> {
        let config_file = Self::config_file()
            .ok_or_else(|| Error::config("Could not determine config directory"))?;

        if !config_file.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&config_file)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_dir = Self::config_dir()
            .ok_or_else(|| Error::config("Could not determine config directory"))?;

        std::fs::create_dir_all(&config_dir)?;

        let config_file = config_dir.join("config.toml");
        let content = toml::to_string_pretty(self)
            .map_err(|e| Error::config(format!("Failed to serialize config: {}", e)))?;
        std::fs::write(config_file, content)?;

        Ok(())
    }

    /// Get the global configuration (read-only)
    pub fn global() -> parking_lot::RwLockReadGuard<'static, Config> {
        CONFIG.read()
    }

    /// Update the global configuration
    pub fn set_global(config: Config) {
        *CONFIG.write() = config;
    }

    /// Initialize global configuration from file
    pub fn init() -> Result<()> {
        let config = Self::load()?;
        Self::set_global(config);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.terminal.scrollback_lines, 10000);
        assert_eq!(config.ui.theme, "dark");
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.terminal.font_size, config.terminal.font_size);
    }
}
