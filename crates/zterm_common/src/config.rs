//! Configuration management for zTerm

use crate::{Error, Result};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::info;

/// Global configuration instance
static CONFIG: Lazy<RwLock<Config>> = Lazy::new(|| RwLock::new(Config::default()));

/// Current configuration schema version
/// Increment this when adding new fields to the configuration
pub const CONFIG_VERSION: u32 = 1;

/// Main configuration structure for zTerm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Configuration schema version (for migration purposes)
    #[serde(default = "default_version")]
    pub version: u32,

    /// Terminal settings
    pub terminal: TerminalConfig,

    /// UI settings
    pub ui: UiConfig,

    /// Keybinding settings
    pub keybindings: KeybindingsConfig,
}

fn default_version() -> u32 {
    0 // Old configs without version field
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
    /// New tab shortcut
    pub new_tab: String,

    /// Close tab shortcut
    pub close_tab: String,

    /// Next tab shortcut
    pub next_tab: String,

    /// Previous tab shortcut
    pub prev_tab: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: CONFIG_VERSION,
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
        if cfg!(target_os = "macos") {
            Self {
                new_tab: "cmd+t".to_string(),
                close_tab: "cmd+w".to_string(),
                next_tab: "cmd+shift+]".to_string(),
                prev_tab: "cmd+shift+[".to_string(),
            }
        } else {
            Self {
                new_tab: "ctrl+t".to_string(),
                close_tab: "ctrl+w".to_string(),
                // Use alt+right/left as they work reliably across systems
                next_tab: "alt+right".to_string(),
                prev_tab: "alt+left".to_string(),
            }
        }
    }
}

/// Result of configuration migration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationResult {
    /// No migration needed, config is up to date
    UpToDate,
    /// Config was created (first run)
    Created,
    /// Config was migrated and backup was created
    Migrated { backup_path: PathBuf },
}

impl Config {
    /// Get the configuration directory path
    pub fn config_dir() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("zterm"))
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

    /// Load configuration with automatic migration
    ///
    /// This method:
    /// 1. Creates the config file if it doesn't exist
    /// 2. Merges missing fields from defaults if the config is outdated
    /// 3. Creates a backup before any modifications
    pub fn load_and_migrate() -> Result<(Self, MigrationResult)> {
        let config_file = Self::config_file()
            .ok_or_else(|| Error::config("Could not determine config directory"))?;

        // Ensure config directory exists
        if let Some(config_dir) = Self::config_dir() {
            std::fs::create_dir_all(&config_dir)?;
        }

        // If config doesn't exist, create it with defaults
        if !config_file.exists() {
            let config = Self::default();
            config.save()?;
            info!("Created default configuration file at {:?}", config_file);
            return Ok((config, MigrationResult::Created));
        }

        // Load existing config content
        let content = std::fs::read_to_string(&config_file)?;

        // Try to parse as TOML Value first to check structure
        let user_value: toml::Value = toml::from_str(&content)
            .map_err(|e| Error::config(format!("Failed to parse config: {}", e)))?;

        // Get the version from the user config
        let user_version = user_value
            .get("version")
            .and_then(|v| v.as_integer())
            .map(|v| v as u32)
            .unwrap_or(0);

        // If version matches, just load normally
        if user_version == CONFIG_VERSION {
            let config: Config = toml::from_str(&content)?;
            return Ok((config, MigrationResult::UpToDate));
        }

        // Version mismatch - need to migrate
        info!(
            "Config version {} is outdated (current: {}), migrating...",
            user_version, CONFIG_VERSION
        );

        // Create backup before modifying
        let backup_path = Self::create_backup(&config_file)?;
        info!("Created backup at {:?}", backup_path);

        // Merge with defaults
        let default_str = toml::to_string(&Self::default())
            .map_err(|e| Error::config(format!("Failed to serialize defaults: {}", e)))?;
        let default_value: toml::Value = toml::from_str(&default_str)
            .map_err(|e| Error::config(format!("Failed to parse defaults: {}", e)))?;

        let merged_value = Self::merge_toml_values(default_value, user_value);

        // Parse the merged config
        let merged_content = toml::to_string_pretty(&merged_value)
            .map_err(|e| Error::config(format!("Failed to serialize merged config: {}", e)))?;

        let mut config: Config = toml::from_str(&merged_content)?;

        // Update version to current
        config.version = CONFIG_VERSION;

        // Save the migrated config
        config.save()?;
        info!("Configuration migrated successfully");

        Ok((config, MigrationResult::Migrated { backup_path }))
    }

    /// Create a backup of the configuration file
    fn create_backup(config_file: &PathBuf) -> Result<PathBuf> {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!("config.{}.toml.bak", timestamp);
        let backup_path = config_file.parent().unwrap().join(backup_name);

        std::fs::copy(config_file, &backup_path)?;
        Ok(backup_path)
    }

    /// Recursively merge TOML values
    ///
    /// - For tables: merge recursively, user values take precedence
    /// - For other types: user value takes precedence if present
    /// - Missing keys in user config are filled from defaults
    fn merge_toml_values(default: toml::Value, user: toml::Value) -> toml::Value {
        match (default, user) {
            // Both are tables - merge recursively
            (toml::Value::Table(mut default_table), toml::Value::Table(user_table)) => {
                for (key, user_value) in user_table {
                    if let Some(default_value) = default_table.remove(&key) {
                        // Key exists in both - merge recursively
                        default_table.insert(key, Self::merge_toml_values(default_value, user_value));
                    } else {
                        // Key only in user config - keep it (might be custom user setting)
                        default_table.insert(key, user_value);
                    }
                }
                // Remaining keys from default are kept (new settings)
                toml::Value::Table(default_table)
            }
            // User value takes precedence for non-table types
            (_default, user) => user,
        }
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

    /// Initialize global configuration with migration support
    pub fn init_with_migration() -> Result<MigrationResult> {
        let (config, result) = Self::load_and_migrate()?;
        Self::set_global(config);
        Ok(result)
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

    #[test]
    fn test_default_config_has_version() {
        let config = Config::default();
        assert_eq!(config.version, CONFIG_VERSION);
    }

    #[test]
    fn test_merge_toml_values_preserves_user_values() {
        let default: toml::Value = toml::from_str(
            r#"
            [terminal]
            font_size = 14.0
            font_family = "Default Font"
            "#,
        )
        .unwrap();

        let user: toml::Value = toml::from_str(
            r#"
            [terminal]
            font_size = 18.0
            "#,
        )
        .unwrap();

        let merged = Config::merge_toml_values(default, user);

        // User value should be preserved
        assert_eq!(
            merged["terminal"]["font_size"].as_float().unwrap(),
            18.0
        );
        // Missing value should be filled from default
        assert_eq!(
            merged["terminal"]["font_family"].as_str().unwrap(),
            "Default Font"
        );
    }

    #[test]
    fn test_merge_toml_values_adds_missing_sections() {
        let default: toml::Value = toml::from_str(
            r#"
            [terminal]
            font_size = 14.0

            [ui]
            theme = "dark"
            "#,
        )
        .unwrap();

        let user: toml::Value = toml::from_str(
            r#"
            [terminal]
            font_size = 16.0
            "#,
        )
        .unwrap();

        let merged = Config::merge_toml_values(default, user);

        // User value preserved
        assert_eq!(
            merged["terminal"]["font_size"].as_float().unwrap(),
            16.0
        );
        // Missing section added from default
        assert_eq!(merged["ui"]["theme"].as_str().unwrap(), "dark");
    }

    #[test]
    fn test_merge_toml_values_preserves_user_custom_keys() {
        let default: toml::Value = toml::from_str(
            r#"
            [terminal]
            font_size = 14.0
            "#,
        )
        .unwrap();

        let user: toml::Value = toml::from_str(
            r#"
            [terminal]
            font_size = 16.0
            custom_setting = "user defined"
            "#,
        )
        .unwrap();

        let merged = Config::merge_toml_values(default, user);

        // User value preserved
        assert_eq!(
            merged["terminal"]["font_size"].as_float().unwrap(),
            16.0
        );
        // Custom user key preserved
        assert_eq!(
            merged["terminal"]["custom_setting"].as_str().unwrap(),
            "user defined"
        );
    }

    #[test]
    fn test_merge_toml_values_nested_tables() {
        let default: toml::Value = toml::from_str(
            r#"
            [outer]
            [outer.inner]
            key1 = "default1"
            key2 = "default2"
            "#,
        )
        .unwrap();

        let user: toml::Value = toml::from_str(
            r#"
            [outer]
            [outer.inner]
            key1 = "user1"
            "#,
        )
        .unwrap();

        let merged = Config::merge_toml_values(default, user);

        // User value preserved
        assert_eq!(
            merged["outer"]["inner"]["key1"].as_str().unwrap(),
            "user1"
        );
        // Default value filled
        assert_eq!(
            merged["outer"]["inner"]["key2"].as_str().unwrap(),
            "default2"
        );
    }

    #[test]
    fn test_merge_full_config() {
        // Simulate an old config missing new fields
        let old_config_str = r#"
            version = 0
            [terminal]
            font_size = 18.0
            font_family = "User Font"
            scrollback_lines = 5000
            bell_enabled = false
            cursor_style = "bar"
            cursor_blink = false
            [ui]
            theme = "dracula"
            opacity = 0.9
        "#;

        let user_value: toml::Value = toml::from_str(old_config_str).unwrap();

        let default_str = toml::to_string(&Config::default()).unwrap();
        let default_value: toml::Value = toml::from_str(&default_str).unwrap();

        let merged = Config::merge_toml_values(default_value, user_value);
        let merged_str = toml::to_string_pretty(&merged).unwrap();
        let config: Config = toml::from_str(&merged_str).unwrap();

        // User values preserved
        assert_eq!(config.terminal.font_size, 18.0);
        assert_eq!(config.terminal.font_family, "User Font");
        assert_eq!(config.terminal.scrollback_lines, 5000);
        assert!(!config.terminal.bell_enabled);
        assert_eq!(config.terminal.cursor_style, "bar");
        assert!(!config.terminal.cursor_blink);
        assert_eq!(config.ui.theme, "dracula");
        assert_eq!(config.ui.opacity, 0.9);

        // Default values filled for missing fields
        assert!(config.terminal.shell.is_none()); // Default
        assert!(config.terminal.shell_args.is_empty()); // Default
        assert!(config.ui.show_tab_bar); // Default
        assert_eq!(config.ui.tab_bar_position, "top"); // Default

        // Keybindings section should be filled with defaults
        assert!(!config.keybindings.new_tab.is_empty());
        assert!(!config.keybindings.close_tab.is_empty());
    }

    #[test]
    fn test_migration_result_enum() {
        // Test that all variants can be created
        let _up_to_date = MigrationResult::UpToDate;
        let _created = MigrationResult::Created;
        let _migrated = MigrationResult::Migrated {
            backup_path: PathBuf::from("/tmp/backup.toml"),
        };
    }

    #[test]
    fn test_config_version_constant() {
        assert!(CONFIG_VERSION >= 1);
    }
}
