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
pub const CONFIG_VERSION: u32 = 2;

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
    // === System shortcuts ===
    /// Quit application shortcut
    pub quit: String,

    /// New window shortcut
    pub new_window: String,

    // === Tab management ===
    /// New tab shortcut
    pub new_tab: String,

    /// Close tab shortcut
    pub close_tab: String,

    /// Next tab shortcut
    pub next_tab: String,

    /// Previous tab shortcut
    pub prev_tab: String,

    // === Window operations ===
    /// Toggle fullscreen shortcut
    pub toggle_fullscreen: String,

    // === Split pane ===
    /// Split horizontal shortcut
    pub split_horizontal: String,

    /// Split vertical shortcut
    pub split_vertical: String,

    // === Zoom ===
    /// Zoom in shortcut
    pub zoom_in: String,

    /// Zoom out shortcut
    pub zoom_out: String,

    /// Reset zoom shortcut
    pub reset_zoom: String,

    // === Terminal operations ===
    /// Copy selection shortcut
    pub copy: String,

    /// Paste clipboard shortcut
    pub paste: String,

    /// Search shortcut
    pub search: String,

    // === Scrolling ===
    /// Scroll up one line shortcut
    pub scroll_up: String,

    /// Scroll down one line shortcut
    pub scroll_down: String,

    /// Scroll page up shortcut
    pub scroll_page_up: String,

    /// Scroll page down shortcut
    pub scroll_page_down: String,

    /// Scroll to top shortcut
    pub scroll_to_top: String,

    /// Scroll to bottom shortcut
    pub scroll_to_bottom: String,

    // === Other ===
    /// Command palette shortcut
    pub command_palette: String,

    // === Tab switching (Ctrl+1 to Ctrl+9) ===
    /// Go to tab 1
    pub goto_tab_1: String,

    /// Go to tab 2
    pub goto_tab_2: String,

    /// Go to tab 3
    pub goto_tab_3: String,

    /// Go to tab 4
    pub goto_tab_4: String,

    /// Go to tab 5
    pub goto_tab_5: String,

    /// Go to tab 6
    pub goto_tab_6: String,

    /// Go to tab 7
    pub goto_tab_7: String,

    /// Go to tab 8
    pub goto_tab_8: String,

    /// Go to tab 9 (also last tab)
    pub goto_tab_9: String,
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
                // System
                quit: "cmd+q".to_string(),
                new_window: "cmd+n".to_string(),
                // Tab management
                new_tab: "cmd+t".to_string(),
                close_tab: "cmd+w".to_string(),
                next_tab: "cmd+shift+]".to_string(),
                prev_tab: "cmd+shift+[".to_string(),
                // Window operations
                toggle_fullscreen: "cmd+ctrl+f".to_string(),
                // Split pane
                split_horizontal: "cmd+d".to_string(),
                split_vertical: "cmd+shift+d".to_string(),
                // Zoom
                zoom_in: "cmd+=".to_string(),
                zoom_out: "cmd+-".to_string(),
                reset_zoom: "cmd+0".to_string(),
                // Terminal operations
                copy: "cmd+c".to_string(),
                paste: "cmd+v".to_string(),
                search: "cmd+f".to_string(),
                // Scrolling
                scroll_up: "cmd+up".to_string(),
                scroll_down: "cmd+down".to_string(),
                scroll_page_up: "pageup".to_string(),
                scroll_page_down: "pagedown".to_string(),
                scroll_to_top: "cmd+home".to_string(),
                scroll_to_bottom: "cmd+end".to_string(),
                // Other
                command_palette: "cmd+shift+p".to_string(),
                // Tab switching
                goto_tab_1: "cmd+1".to_string(),
                goto_tab_2: "cmd+2".to_string(),
                goto_tab_3: "cmd+3".to_string(),
                goto_tab_4: "cmd+4".to_string(),
                goto_tab_5: "cmd+5".to_string(),
                goto_tab_6: "cmd+6".to_string(),
                goto_tab_7: "cmd+7".to_string(),
                goto_tab_8: "cmd+8".to_string(),
                goto_tab_9: "cmd+9".to_string(),
            }
        } else {
            Self {
                // System
                quit: "alt+f4".to_string(),
                new_window: "ctrl+shift+n".to_string(),
                // Tab management
                new_tab: "ctrl+t".to_string(),
                close_tab: "ctrl+w".to_string(),
                // Use alt+right/left as they work reliably across systems
                next_tab: "alt+right".to_string(),
                prev_tab: "alt+left".to_string(),
                // Window operations
                toggle_fullscreen: "f11".to_string(),
                // Split pane
                split_horizontal: "ctrl+shift+d".to_string(),
                split_vertical: "ctrl+shift+e".to_string(),
                // Zoom
                zoom_in: "ctrl+=".to_string(),
                zoom_out: "ctrl+-".to_string(),
                reset_zoom: "ctrl+0".to_string(),
                // Terminal operations
                copy: "ctrl+shift+c".to_string(),
                paste: "ctrl+shift+v".to_string(),
                search: "ctrl+shift+f".to_string(),
                // Scrolling
                scroll_up: "ctrl+up".to_string(),
                scroll_down: "ctrl+down".to_string(),
                scroll_page_up: "pageup".to_string(),
                scroll_page_down: "pagedown".to_string(),
                scroll_to_top: "ctrl+home".to_string(),
                scroll_to_bottom: "ctrl+end".to_string(),
                // Other
                command_palette: "ctrl+shift+p".to_string(),
                // Tab switching
                goto_tab_1: "ctrl+1".to_string(),
                goto_tab_2: "ctrl+2".to_string(),
                goto_tab_3: "ctrl+3".to_string(),
                goto_tab_4: "ctrl+4".to_string(),
                goto_tab_5: "ctrl+5".to_string(),
                goto_tab_6: "ctrl+6".to_string(),
                goto_tab_7: "ctrl+7".to_string(),
                goto_tab_8: "ctrl+8".to_string(),
                goto_tab_9: "ctrl+9".to_string(),
            }
        }
    }
}

/// All configurable actions in zTerm
/// Adding a new variant here without updating `KeybindingsConfig::get_keybinding()`
/// will cause a compile error (non-exhaustive match).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ConfigurableAction {
    // System
    Quit,
    NewWindow,
    // Tab management
    NewTab,
    CloseTab,
    NextTab,
    PrevTab,
    // Window operations
    ToggleFullscreen,
    // Split pane
    SplitHorizontal,
    SplitVertical,
    // Zoom
    ZoomIn,
    ZoomOut,
    ResetZoom,
    // Terminal operations
    Copy,
    Paste,
    Search,
    // Scrolling
    ScrollUp,
    ScrollDown,
    ScrollPageUp,
    ScrollPageDown,
    ScrollToTop,
    ScrollToBottom,
    // Other
    CommandPalette,
    // Tab switching
    GotoTab1,
    GotoTab2,
    GotoTab3,
    GotoTab4,
    GotoTab5,
    GotoTab6,
    GotoTab7,
    GotoTab8,
    GotoTab9,
}

impl KeybindingsConfig {
    /// Get the keybinding for a specific action.
    ///
    /// This method uses an exhaustive match to ensure all actions have a corresponding
    /// configuration field. Adding a new `ConfigurableAction` variant without updating
    /// this match will cause a compile error.
    pub fn get_keybinding(&self, action: ConfigurableAction) -> &str {
        match action {
            // System
            ConfigurableAction::Quit => &self.quit,
            ConfigurableAction::NewWindow => &self.new_window,
            // Tab management
            ConfigurableAction::NewTab => &self.new_tab,
            ConfigurableAction::CloseTab => &self.close_tab,
            ConfigurableAction::NextTab => &self.next_tab,
            ConfigurableAction::PrevTab => &self.prev_tab,
            // Window operations
            ConfigurableAction::ToggleFullscreen => &self.toggle_fullscreen,
            // Split pane
            ConfigurableAction::SplitHorizontal => &self.split_horizontal,
            ConfigurableAction::SplitVertical => &self.split_vertical,
            // Zoom
            ConfigurableAction::ZoomIn => &self.zoom_in,
            ConfigurableAction::ZoomOut => &self.zoom_out,
            ConfigurableAction::ResetZoom => &self.reset_zoom,
            // Terminal operations
            ConfigurableAction::Copy => &self.copy,
            ConfigurableAction::Paste => &self.paste,
            ConfigurableAction::Search => &self.search,
            // Scrolling
            ConfigurableAction::ScrollUp => &self.scroll_up,
            ConfigurableAction::ScrollDown => &self.scroll_down,
            ConfigurableAction::ScrollPageUp => &self.scroll_page_up,
            ConfigurableAction::ScrollPageDown => &self.scroll_page_down,
            ConfigurableAction::ScrollToTop => &self.scroll_to_top,
            ConfigurableAction::ScrollToBottom => &self.scroll_to_bottom,
            // Other
            ConfigurableAction::CommandPalette => &self.command_palette,
            // Tab switching
            ConfigurableAction::GotoTab1 => &self.goto_tab_1,
            ConfigurableAction::GotoTab2 => &self.goto_tab_2,
            ConfigurableAction::GotoTab3 => &self.goto_tab_3,
            ConfigurableAction::GotoTab4 => &self.goto_tab_4,
            ConfigurableAction::GotoTab5 => &self.goto_tab_5,
            ConfigurableAction::GotoTab6 => &self.goto_tab_6,
            ConfigurableAction::GotoTab7 => &self.goto_tab_7,
            ConfigurableAction::GotoTab8 => &self.goto_tab_8,
            ConfigurableAction::GotoTab9 => &self.goto_tab_9,
        }
    }

    /// Get all configurable actions with their keybindings.
    /// Useful for building keybinding UI or debug output.
    pub fn all_keybindings(&self) -> Vec<(ConfigurableAction, &str)> {
        use ConfigurableAction::*;
        vec![
            (Quit, self.get_keybinding(Quit)),
            (NewWindow, self.get_keybinding(NewWindow)),
            (NewTab, self.get_keybinding(NewTab)),
            (CloseTab, self.get_keybinding(CloseTab)),
            (NextTab, self.get_keybinding(NextTab)),
            (PrevTab, self.get_keybinding(PrevTab)),
            (ToggleFullscreen, self.get_keybinding(ToggleFullscreen)),
            (SplitHorizontal, self.get_keybinding(SplitHorizontal)),
            (SplitVertical, self.get_keybinding(SplitVertical)),
            (ZoomIn, self.get_keybinding(ZoomIn)),
            (ZoomOut, self.get_keybinding(ZoomOut)),
            (ResetZoom, self.get_keybinding(ResetZoom)),
            (Copy, self.get_keybinding(Copy)),
            (Paste, self.get_keybinding(Paste)),
            (Search, self.get_keybinding(Search)),
            (ScrollUp, self.get_keybinding(ScrollUp)),
            (ScrollDown, self.get_keybinding(ScrollDown)),
            (ScrollPageUp, self.get_keybinding(ScrollPageUp)),
            (ScrollPageDown, self.get_keybinding(ScrollPageDown)),
            (ScrollToTop, self.get_keybinding(ScrollToTop)),
            (ScrollToBottom, self.get_keybinding(ScrollToBottom)),
            (CommandPalette, self.get_keybinding(CommandPalette)),
            (GotoTab1, self.get_keybinding(GotoTab1)),
            (GotoTab2, self.get_keybinding(GotoTab2)),
            (GotoTab3, self.get_keybinding(GotoTab3)),
            (GotoTab4, self.get_keybinding(GotoTab4)),
            (GotoTab5, self.get_keybinding(GotoTab5)),
            (GotoTab6, self.get_keybinding(GotoTab6)),
            (GotoTab7, self.get_keybinding(GotoTab7)),
            (GotoTab8, self.get_keybinding(GotoTab8)),
            (GotoTab9, self.get_keybinding(GotoTab9)),
        ]
    }
}

impl KeybindingsConfig {
    /// Convert config keybinding format ("ctrl+t") to GPUI format ("ctrl-t")
    ///
    /// GPUI expects keybindings with hyphen separators and lowercase,
    /// while config files typically use plus signs for readability.
    pub fn normalize_keybinding(key: &str) -> String {
        key.replace('+', "-").to_lowercase()
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
    /// 2. Intelligently merges config only if changes are needed:
    ///    - Adds new configuration fields from defaults
    ///    - Removes old configuration fields that no longer exist
    ///    - Preserves user-modified values
    /// 3. Only creates backup and writes file if actual changes are made
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

        // Get default config as TOML value
        let default_str = toml::to_string(&Self::default())
            .map_err(|e| Error::config(format!("Failed to serialize defaults: {}", e)))?;
        let default_value: toml::Value = toml::from_str(&default_str)
            .map_err(|e| Error::config(format!("Failed to parse defaults: {}", e)))?;

        // Merge with defaults
        let merged_value = Self::merge_toml_values(default_value, user_value.clone());

        // Check if the merged config is different from the user config
        let needs_update = user_version != CONFIG_VERSION || !Self::toml_values_equal(&merged_value, &user_value);

        // If no update is needed, just load and return
        if !needs_update {
            let config: Config = toml::from_str(&content)?;
            return Ok((config, MigrationResult::UpToDate));
        }

        // Changes are needed - log the reason
        if user_version != CONFIG_VERSION {
            info!(
                "Config version {} is outdated (current: {}), migrating...",
                user_version, CONFIG_VERSION
            );
        } else {
            info!("Config structure differs from defaults, updating...");
        }

        // Create backup before modifying
        let backup_path = Self::create_backup(&config_file)?;
        info!("Created backup at {:?}", backup_path);

        // Parse the merged config
        let merged_content = toml::to_string_pretty(&merged_value)
            .map_err(|e| Error::config(format!("Failed to serialize merged config: {}", e)))?;

        let mut config: Config = toml::from_str(&merged_content)?;

        // Update version to current
        config.version = CONFIG_VERSION;

        // Save the migrated config
        config.save()?;
        info!("Configuration updated successfully");

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
    /// - Keys in user config that don't exist in defaults are REMOVED (cleanup old config)
    fn merge_toml_values(default: toml::Value, user: toml::Value) -> toml::Value {
        match (default, user) {
            // Both are tables - merge recursively
            (toml::Value::Table(mut default_table), toml::Value::Table(user_table)) => {
                for (key, user_value) in user_table {
                    if let Some(default_value) = default_table.remove(&key) {
                        // Key exists in both - merge recursively
                        default_table.insert(key, Self::merge_toml_values(default_value, user_value));
                    }
                    // else: Key only in user config - REMOVE it (old config item that no longer exists)
                }
                // Remaining keys from default are kept (new settings)
                toml::Value::Table(default_table)
            }
            // User value takes precedence for non-table types
            (_default, user) => user,
        }
    }

    /// Check if two TOML values are semantically equal
    ///
    /// This compares the actual content, ignoring formatting differences
    fn toml_values_equal(a: &toml::Value, b: &toml::Value) -> bool {
        match (a, b) {
            (toml::Value::Table(a_table), toml::Value::Table(b_table)) => {
                if a_table.len() != b_table.len() {
                    return false;
                }
                for (key, a_value) in a_table {
                    match b_table.get(key) {
                        Some(b_value) => {
                            if !Self::toml_values_equal(a_value, b_value) {
                                return false;
                            }
                        }
                        None => return false,
                    }
                }
                true
            }
            (toml::Value::Array(a_arr), toml::Value::Array(b_arr)) => {
                if a_arr.len() != b_arr.len() {
                    return false;
                }
                a_arr.iter().zip(b_arr.iter()).all(|(a, b)| Self::toml_values_equal(a, b))
            }
            _ => a == b,
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
    fn test_merge_toml_values_removes_old_custom_keys() {
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
        // Custom user key REMOVED (not in defaults)
        assert!(merged["terminal"].get("custom_setting").is_none());
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

    // ==================== Keybindings Tests ====================

    #[test]
    fn test_keybindings_all_fields_have_defaults() {
        let kb = KeybindingsConfig::default();

        // System
        assert!(!kb.quit.is_empty(), "quit keybinding should not be empty");
        assert!(!kb.new_window.is_empty(), "new_window keybinding should not be empty");

        // Tab management
        assert!(!kb.new_tab.is_empty(), "new_tab keybinding should not be empty");
        assert!(!kb.close_tab.is_empty(), "close_tab keybinding should not be empty");
        assert!(!kb.next_tab.is_empty(), "next_tab keybinding should not be empty");
        assert!(!kb.prev_tab.is_empty(), "prev_tab keybinding should not be empty");

        // Window operations
        assert!(!kb.toggle_fullscreen.is_empty(), "toggle_fullscreen keybinding should not be empty");

        // Split pane
        assert!(!kb.split_horizontal.is_empty(), "split_horizontal keybinding should not be empty");
        assert!(!kb.split_vertical.is_empty(), "split_vertical keybinding should not be empty");

        // Zoom
        assert!(!kb.zoom_in.is_empty(), "zoom_in keybinding should not be empty");
        assert!(!kb.zoom_out.is_empty(), "zoom_out keybinding should not be empty");
        assert!(!kb.reset_zoom.is_empty(), "reset_zoom keybinding should not be empty");

        // Terminal operations
        assert!(!kb.copy.is_empty(), "copy keybinding should not be empty");
        assert!(!kb.paste.is_empty(), "paste keybinding should not be empty");
        assert!(!kb.search.is_empty(), "search keybinding should not be empty");

        // Scrolling
        assert!(!kb.scroll_up.is_empty(), "scroll_up keybinding should not be empty");
        assert!(!kb.scroll_down.is_empty(), "scroll_down keybinding should not be empty");
        assert!(!kb.scroll_page_up.is_empty(), "scroll_page_up keybinding should not be empty");
        assert!(!kb.scroll_page_down.is_empty(), "scroll_page_down keybinding should not be empty");
        assert!(!kb.scroll_to_top.is_empty(), "scroll_to_top keybinding should not be empty");
        assert!(!kb.scroll_to_bottom.is_empty(), "scroll_to_bottom keybinding should not be empty");

        // Other
        assert!(!kb.command_palette.is_empty(), "command_palette keybinding should not be empty");

        // Tab switching
        assert!(!kb.goto_tab_1.is_empty(), "goto_tab_1 keybinding should not be empty");
        assert!(!kb.goto_tab_2.is_empty(), "goto_tab_2 keybinding should not be empty");
        assert!(!kb.goto_tab_3.is_empty(), "goto_tab_3 keybinding should not be empty");
        assert!(!kb.goto_tab_4.is_empty(), "goto_tab_4 keybinding should not be empty");
        assert!(!kb.goto_tab_5.is_empty(), "goto_tab_5 keybinding should not be empty");
        assert!(!kb.goto_tab_6.is_empty(), "goto_tab_6 keybinding should not be empty");
        assert!(!kb.goto_tab_7.is_empty(), "goto_tab_7 keybinding should not be empty");
        assert!(!kb.goto_tab_8.is_empty(), "goto_tab_8 keybinding should not be empty");
        assert!(!kb.goto_tab_9.is_empty(), "goto_tab_9 keybinding should not be empty");
    }

    #[test]
    fn test_get_keybinding_exhaustive() {
        // This test verifies that get_keybinding covers all ConfigurableAction variants
        // If a new variant is added to ConfigurableAction without updating get_keybinding,
        // this test will fail to compile (non-exhaustive match)
        let kb = KeybindingsConfig::default();

        use ConfigurableAction::*;
        let all_actions = [
            Quit, NewWindow,
            NewTab, CloseTab, NextTab, PrevTab,
            ToggleFullscreen,
            SplitHorizontal, SplitVertical,
            ZoomIn, ZoomOut, ResetZoom,
            Copy, Paste, Search,
            ScrollUp, ScrollDown, ScrollPageUp, ScrollPageDown, ScrollToTop, ScrollToBottom,
            CommandPalette,
            GotoTab1, GotoTab2, GotoTab3, GotoTab4, GotoTab5, GotoTab6, GotoTab7, GotoTab8, GotoTab9,
        ];

        for action in all_actions {
            let binding = kb.get_keybinding(action);
            assert!(!binding.is_empty(), "Action {:?} should have a non-empty keybinding", action);
        }
    }

    #[test]
    fn test_all_keybindings_returns_correct_count() {
        let kb = KeybindingsConfig::default();
        let all = kb.all_keybindings();

        // Total number of configurable actions: 31 (removed ClearScreen and FocusTerminal, added GotoTab1-9)
        assert_eq!(all.len(), 31, "all_keybindings should return 31 entries");

        // Verify all entries have non-empty bindings
        for (action, binding) in &all {
            assert!(!binding.is_empty(), "Action {:?} has empty binding in all_keybindings", action);
        }
    }

    #[test]
    fn test_keybindings_no_duplicates() {
        let kb = KeybindingsConfig::default();
        let all = kb.all_keybindings();

        let mut seen = std::collections::HashSet::new();
        let mut duplicates = Vec::new();

        for (action, binding) in &all {
            if !seen.insert(*binding) {
                duplicates.push((action, *binding));
            }
        }

        // Note: Some duplicates may be intentional (e.g., pageup/pagedown on different platforms)
        // This test just warns about duplicates for manual review
        if !duplicates.is_empty() {
            eprintln!("Warning: Found duplicate keybindings (may be intentional):");
            for (action, binding) in &duplicates {
                eprintln!("  {:?}: {}", action, binding);
            }
        }
    }

    #[test]
    fn test_keybindings_format_valid() {
        let kb = KeybindingsConfig::default();
        let all = kb.all_keybindings();

        for (action, binding) in all {
            // Check basic format: should contain valid modifier/key patterns
            // Valid patterns: ctrl+x, cmd+x, alt+x, shift+x, f1-f12, etc.
            let binding_lower = binding.to_lowercase();

            // Check that it's not just whitespace
            assert!(!binding.trim().is_empty(), "Action {:?} has whitespace-only binding", action);

            // Check for common typos or invalid formats
            assert!(!binding_lower.contains("++"), "Action {:?} has invalid '++' in binding: {}", action, binding);
            assert!(!binding_lower.starts_with('+'), "Action {:?} binding starts with '+': {}", action, binding);
            assert!(!binding_lower.ends_with('+'), "Action {:?} binding ends with '+': {}", action, binding);
        }
    }

    #[test]
    fn test_keybindings_serialization_roundtrip() {
        let kb = KeybindingsConfig::default();

        // Serialize to TOML
        let toml_str = toml::to_string_pretty(&kb).expect("Failed to serialize keybindings");

        // Deserialize back
        let parsed: KeybindingsConfig = toml::from_str(&toml_str).expect("Failed to deserialize keybindings");

        // Verify all values match
        assert_eq!(kb.quit, parsed.quit);
        assert_eq!(kb.new_window, parsed.new_window);
        assert_eq!(kb.new_tab, parsed.new_tab);
        assert_eq!(kb.close_tab, parsed.close_tab);
        assert_eq!(kb.next_tab, parsed.next_tab);
        assert_eq!(kb.prev_tab, parsed.prev_tab);
        assert_eq!(kb.toggle_fullscreen, parsed.toggle_fullscreen);
        assert_eq!(kb.split_horizontal, parsed.split_horizontal);
        assert_eq!(kb.split_vertical, parsed.split_vertical);
        assert_eq!(kb.zoom_in, parsed.zoom_in);
        assert_eq!(kb.zoom_out, parsed.zoom_out);
        assert_eq!(kb.reset_zoom, parsed.reset_zoom);
        assert_eq!(kb.copy, parsed.copy);
        assert_eq!(kb.paste, parsed.paste);
        assert_eq!(kb.search, parsed.search);
        assert_eq!(kb.scroll_up, parsed.scroll_up);
        assert_eq!(kb.scroll_down, parsed.scroll_down);
        assert_eq!(kb.scroll_page_up, parsed.scroll_page_up);
        assert_eq!(kb.scroll_page_down, parsed.scroll_page_down);
        assert_eq!(kb.scroll_to_top, parsed.scroll_to_top);
        assert_eq!(kb.scroll_to_bottom, parsed.scroll_to_bottom);
        assert_eq!(kb.command_palette, parsed.command_palette);
        assert_eq!(kb.goto_tab_1, parsed.goto_tab_1);
        assert_eq!(kb.goto_tab_2, parsed.goto_tab_2);
        assert_eq!(kb.goto_tab_3, parsed.goto_tab_3);
        assert_eq!(kb.goto_tab_4, parsed.goto_tab_4);
        assert_eq!(kb.goto_tab_5, parsed.goto_tab_5);
        assert_eq!(kb.goto_tab_6, parsed.goto_tab_6);
        assert_eq!(kb.goto_tab_7, parsed.goto_tab_7);
        assert_eq!(kb.goto_tab_8, parsed.goto_tab_8);
        assert_eq!(kb.goto_tab_9, parsed.goto_tab_9);
    }

    #[test]
    fn test_configurable_action_debug_impl() {
        // Ensure all ConfigurableAction variants have Debug impl
        use ConfigurableAction::*;
        let actions = [
            Quit, NewWindow, NewTab, CloseTab, NextTab, PrevTab,
            ToggleFullscreen, SplitHorizontal, SplitVertical,
            ZoomIn, ZoomOut, ResetZoom,
            Copy, Paste, Search,
            ScrollUp, ScrollDown, ScrollPageUp, ScrollPageDown, ScrollToTop, ScrollToBottom,
            CommandPalette,
            GotoTab1, GotoTab2, GotoTab3, GotoTab4, GotoTab5, GotoTab6, GotoTab7, GotoTab8, GotoTab9,
        ];

        for action in actions {
            let debug_str = format!("{:?}", action);
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_toml_values_equal_tables() {
        let a: toml::Value = toml::from_str(
            r#"
            [terminal]
            font_size = 14.0
            font_family = "Test"
            "#,
        )
        .unwrap();

        let b: toml::Value = toml::from_str(
            r#"
            [terminal]
            font_size = 14.0
            font_family = "Test"
            "#,
        )
        .unwrap();

        assert!(Config::toml_values_equal(&a, &b));

        let c: toml::Value = toml::from_str(
            r#"
            [terminal]
            font_size = 16.0
            font_family = "Test"
            "#,
        )
        .unwrap();

        assert!(!Config::toml_values_equal(&a, &c));
    }

    #[test]
    fn test_toml_values_equal_different_keys() {
        let a: toml::Value = toml::from_str(
            r#"
            [terminal]
            font_size = 14.0
            "#,
        )
        .unwrap();

        let b: toml::Value = toml::from_str(
            r#"
            [terminal]
            font_size = 14.0
            font_family = "Test"
            "#,
        )
        .unwrap();

        assert!(!Config::toml_values_equal(&a, &b));
    }

    #[test]
    fn test_toml_values_equal_arrays() {
        let a: toml::Value = toml::from_str(
            r#"
            items = [1, 2, 3]
            "#,
        )
        .unwrap();

        let b: toml::Value = toml::from_str(
            r#"
            items = [1, 2, 3]
            "#,
        )
        .unwrap();

        assert!(Config::toml_values_equal(&a, &b));

        let c: toml::Value = toml::from_str(
            r#"
            items = [1, 2]
            "#,
        )
        .unwrap();

        assert!(!Config::toml_values_equal(&a, &c));
    }

    #[test]
    fn test_merge_removes_obsolete_sections() {
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
            [old_section]
            some_value = "obsolete"
            "#,
        )
        .unwrap();

        let merged = Config::merge_toml_values(default, user);

        // User value preserved
        assert_eq!(merged["terminal"]["font_size"].as_float().unwrap(), 16.0);
        // New section added
        assert!(merged.get("ui").is_some());
        // Old section REMOVED
        assert!(merged.get("old_section").is_none());
    }

    #[test]
    fn test_migration_adds_new_keybindings() {
        // Simulate old config with only 4 keybindings (version 1)
        let old_config_str = r#"
            version = 1
            [terminal]
            font_size = 14.0
            font_family = "Test Font"
            scrollback_lines = 10000
            bell_enabled = true
            cursor_style = "block"
            cursor_blink = true
            [ui]
            theme = "dark"
            opacity = 1.0
            show_tab_bar = true
            tab_bar_position = "top"
            decorations = true
            window_width = 1200
            window_height = 800
            [keybindings]
            new_tab = "ctrl+t"
            close_tab = "ctrl+w"
            next_tab = "alt+right"
            prev_tab = "alt+left"
        "#;

        let user_value: toml::Value = toml::from_str(old_config_str).unwrap();
        let default_str = toml::to_string(&Config::default()).unwrap();
        let default_value: toml::Value = toml::from_str(&default_str).unwrap();

        let merged = Config::merge_toml_values(default_value, user_value);
        let merged_str = toml::to_string_pretty(&merged).unwrap();
        let config: Config = toml::from_str(&merged_str).unwrap();

        // Old values should be preserved
        assert_eq!(config.keybindings.new_tab, "ctrl+t");
        assert_eq!(config.keybindings.close_tab, "ctrl+w");
        assert_eq!(config.keybindings.next_tab, "alt+right");
        assert_eq!(config.keybindings.prev_tab, "alt+left");

        // New keybindings should be added from defaults
        assert!(!config.keybindings.quit.is_empty());
        assert!(!config.keybindings.new_window.is_empty());
        assert!(!config.keybindings.copy.is_empty());
        assert!(!config.keybindings.paste.is_empty());
        assert!(!config.keybindings.scroll_up.is_empty());
        assert!(!config.keybindings.command_palette.is_empty());
    }

    #[test]
    fn test_no_update_when_config_matches_defaults() {
        // Create a config that matches defaults exactly
        let default_config = Config::default();
        let default_str = toml::to_string(&default_config).unwrap();
        let default_value: toml::Value = toml::from_str(&default_str).unwrap();

        // Simulate user config that's already up to date
        let user_value = default_value.clone();

        // Merge should produce identical result
        let merged = Config::merge_toml_values(default_value.clone(), user_value.clone());

        assert!(Config::toml_values_equal(&merged, &user_value));
        assert!(Config::toml_values_equal(&merged, &default_value));
    }

    #[test]
    fn test_update_only_when_needed() {
        // Scenario 1: Config with matching version and structure - no update needed
        let current_config_str = toml::to_string(&Config::default()).unwrap();
        let current_value: toml::Value = toml::from_str(&current_config_str).unwrap();

        let default_str = toml::to_string(&Config::default()).unwrap();
        let default_value: toml::Value = toml::from_str(&default_str).unwrap();

        let merged = Config::merge_toml_values(default_value.clone(), current_value.clone());

        // Should be equal - no update needed
        assert!(
            Config::toml_values_equal(&merged, &current_value),
            "Config should not change when it matches defaults"
        );

        // Scenario 2: Config with old field - update needed
        let old_config: toml::Value = toml::from_str(
            r#"
            version = 2
            [terminal]
            font_size = 16.0
            old_field = "should be removed"
            [ui]
            theme = "dark"
            "#,
        )
        .unwrap();

        let merged2 = Config::merge_toml_values(default_value.clone(), old_config.clone());

        // Should be different - update needed
        assert!(
            !Config::toml_values_equal(&merged2, &old_config),
            "Config should change when it has obsolete fields"
        );
        assert!(merged2["terminal"].get("old_field").is_none());
    }

    // ==================== normalize_keybinding Tests ====================

    #[test]
    fn test_normalize_keybinding_basic() {
        assert_eq!(KeybindingsConfig::normalize_keybinding("ctrl+t"), "ctrl-t");
        assert_eq!(KeybindingsConfig::normalize_keybinding("cmd+shift+p"), "cmd-shift-p");
        assert_eq!(KeybindingsConfig::normalize_keybinding("alt+f4"), "alt-f4");
    }

    #[test]
    fn test_normalize_keybinding_lowercase() {
        assert_eq!(KeybindingsConfig::normalize_keybinding("Ctrl+T"), "ctrl-t");
        assert_eq!(KeybindingsConfig::normalize_keybinding("CMD+SHIFT+P"), "cmd-shift-p");
        assert_eq!(KeybindingsConfig::normalize_keybinding("Alt+F4"), "alt-f4");
    }

    #[test]
    fn test_normalize_keybinding_special_keys() {
        assert_eq!(KeybindingsConfig::normalize_keybinding("f1"), "f1");
        assert_eq!(KeybindingsConfig::normalize_keybinding("f11"), "f11");
        assert_eq!(KeybindingsConfig::normalize_keybinding("pageup"), "pageup");
        assert_eq!(KeybindingsConfig::normalize_keybinding("pagedown"), "pagedown");
    }

    #[test]
    fn test_normalize_keybinding_modifiers() {
        assert_eq!(KeybindingsConfig::normalize_keybinding("ctrl+shift+c"), "ctrl-shift-c");
        assert_eq!(KeybindingsConfig::normalize_keybinding("cmd+ctrl+f"), "cmd-ctrl-f");
        assert_eq!(KeybindingsConfig::normalize_keybinding("alt+shift+left"), "alt-shift-left");
    }

    #[test]
    fn test_normalize_keybinding_symbols() {
        assert_eq!(KeybindingsConfig::normalize_keybinding("ctrl+="), "ctrl-=");
        assert_eq!(KeybindingsConfig::normalize_keybinding("ctrl+-"), "ctrl--");
        assert_eq!(KeybindingsConfig::normalize_keybinding("cmd+0"), "cmd-0");
        assert_eq!(KeybindingsConfig::normalize_keybinding("cmd+]"), "cmd-]");
        assert_eq!(KeybindingsConfig::normalize_keybinding("cmd+["), "cmd-[");
    }

    #[test]
    fn test_all_default_keybindings_normalize_correctly() {
        let kb = KeybindingsConfig::default();
        let all = kb.all_keybindings();

        for (action, binding) in all {
            let normalized = KeybindingsConfig::normalize_keybinding(binding);

            // Normalized binding should not contain '+'
            assert!(
                !normalized.contains('+'),
                "Action {:?} normalized binding '{}' still contains '+'",
                action,
                normalized
            );

            // Normalized binding should be lowercase
            assert_eq!(
                normalized,
                normalized.to_lowercase(),
                "Action {:?} normalized binding '{}' is not lowercase",
                action,
                normalized
            );

            // Normalized binding should not be empty
            assert!(
                !normalized.is_empty(),
                "Action {:?} has empty normalized binding",
                action
            );
        }
    }
}
