//! UI component tests for zterm_ui
//!
//! These tests verify the behavior of UI components without requiring
//! a full GPUI window context.

use zterm_common::Config;
use zterm_ui::{TabInfo, TerminalTheme};

mod tab_info_tests {
    use super::*;

    #[test]
    fn test_tab_info_creation() {
        let tab = TabInfo {
            id: 1,
            title: "Terminal 1".to_string(),
            active: true,
            shell_name: "pwsh".to_string(),
            working_directory: "C:\\Users\\test".to_string(),
        };
        assert_eq!(tab.id, 1);
        assert_eq!(tab.title, "Terminal 1");
        assert!(tab.active);
        assert_eq!(tab.shell_name, "pwsh");
    }

    #[test]
    fn test_tab_info_clone() {
        let tab = TabInfo {
            id: 2,
            title: "Terminal 2".to_string(),
            active: false,
            shell_name: "bash".to_string(),
            working_directory: "/home/user".to_string(),
        };
        let cloned = tab.clone();
        assert_eq!(tab.id, cloned.id);
        assert_eq!(tab.title, cloned.title);
        assert_eq!(tab.active, cloned.active);
        assert_eq!(tab.working_directory, cloned.working_directory);
    }

    #[test]
    fn test_tab_info_inactive() {
        let tab = TabInfo {
            id: 0,
            title: "Inactive Tab".to_string(),
            active: false,
            shell_name: "zsh".to_string(),
            working_directory: "~".to_string(),
        };
        assert!(!tab.active);
    }

    #[test]
    fn test_tab_info_empty_title() {
        let tab = TabInfo {
            id: 0,
            title: String::new(),
            active: true,
            shell_name: "cmd".to_string(),
            working_directory: "C:\\".to_string(),
        };
        assert!(tab.title.is_empty());
    }

    #[test]
    fn test_display_directory_home() {
        let tab = TabInfo {
            id: 0,
            title: "Test".to_string(),
            active: true,
            shell_name: "bash".to_string(),
            working_directory: dirs::home_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "~".to_string()),
        };
        assert_eq!(tab.display_directory(), "~");
    }

    #[test]
    fn test_display_directory_subdir() {
        let tab = TabInfo {
            id: 0,
            title: "Test".to_string(),
            active: true,
            shell_name: "bash".to_string(),
            working_directory: "/some/path/to/project".to_string(),
        };
        // Should show the last component
        assert_eq!(tab.display_directory(), "project");
    }
}

mod terminal_theme_tests {
    use super::*;

    #[test]
    fn test_default_theme_is_dark() {
        let theme = TerminalTheme::default();
        let dark = TerminalTheme::dark();
        assert_eq!(theme.font_family, dark.font_family);
        assert_eq!(theme.font_size, dark.font_size);
        assert_eq!(theme.line_height, dark.line_height);
    }

    #[test]
    fn test_dark_theme_properties() {
        let theme = TerminalTheme::dark();
        assert_eq!(theme.font_family.as_ref(), "JetBrainsMono Nerd Font Mono");
        assert_eq!(theme.font_size, 14.0);
        assert_eq!(theme.line_height, 1.4);
        assert_eq!(theme.ansi_colors.len(), 16);
    }

    #[test]
    fn test_light_theme_properties() {
        let theme = TerminalTheme::light();
        assert_eq!(theme.font_family.as_ref(), "JetBrainsMono Nerd Font Mono");
        assert_eq!(theme.font_size, 14.0);
        assert_eq!(theme.ansi_colors.len(), 16);
    }

    #[test]
    fn test_dracula_theme_properties() {
        let theme = TerminalTheme::dracula();
        assert_eq!(theme.font_family.as_ref(), "JetBrainsMono Nerd Font Mono");
        assert_eq!(theme.ansi_colors.len(), 16);
    }

    #[test]
    fn test_one_dark_theme_properties() {
        let theme = TerminalTheme::one_dark();
        assert_eq!(theme.font_family.as_ref(), "JetBrainsMono Nerd Font Mono");
        assert_eq!(theme.ansi_colors.len(), 16);
    }

    #[test]
    fn test_nord_theme_properties() {
        let theme = TerminalTheme::nord();
        assert_eq!(theme.font_family.as_ref(), "JetBrainsMono Nerd Font Mono");
        assert_eq!(theme.ansi_colors.len(), 16);
    }

    #[test]
    fn test_theme_clone() {
        let theme = TerminalTheme::dark();
        let cloned = theme.clone();
        assert_eq!(theme.font_family, cloned.font_family);
        assert_eq!(theme.font_size, cloned.font_size);
    }

    #[test]
    fn test_all_themes_have_16_ansi_colors() {
        assert_eq!(TerminalTheme::dark().ansi_colors.len(), 16);
        assert_eq!(TerminalTheme::light().ansi_colors.len(), 16);
        assert_eq!(TerminalTheme::dracula().ansi_colors.len(), 16);
        assert_eq!(TerminalTheme::one_dark().ansi_colors.len(), 16);
        assert_eq!(TerminalTheme::nord().ansi_colors.len(), 16);
    }

    #[test]
    fn test_all_themes_have_same_font_settings() {
        let themes = [
            TerminalTheme::dark(),
            TerminalTheme::light(),
            TerminalTheme::dracula(),
            TerminalTheme::one_dark(),
            TerminalTheme::nord(),
        ];

        for theme in &themes {
            assert_eq!(theme.font_family.as_ref(), "JetBrainsMono Nerd Font Mono");
            assert_eq!(theme.font_size, 14.0);
            assert_eq!(theme.line_height, 1.4);
        }
    }
}

mod terminal_theme_config_tests {
    use super::*;

    fn create_custom_config(theme: &str, font_size: f32, font_family: &str) -> Config {
        let mut config = Config::default();
        config.ui.theme = theme.to_string();
        config.terminal.font_size = font_size;
        config.terminal.font_family = font_family.to_string();
        config
    }

    #[test]
    fn test_from_config_dark() {
        let config = create_custom_config("dark", 16.0, "Consolas");
        let theme = TerminalTheme::from_config(&config);

        let dark = TerminalTheme::dark();
        assert_eq!(theme.background.r, dark.background.r);
        assert_eq!(theme.font_size, 16.0);
        assert_eq!(theme.font_family.as_ref(), "Consolas");
    }

    #[test]
    fn test_from_config_light() {
        let config = create_custom_config("light", 14.0, "Menlo");
        let theme = TerminalTheme::from_config(&config);

        let light = TerminalTheme::light();
        assert_eq!(theme.background.r, light.background.r);
        assert_eq!(theme.font_family.as_ref(), "Menlo");
    }

    #[test]
    fn test_from_config_dracula() {
        let config = create_custom_config("dracula", 18.0, "Fira Code");
        let theme = TerminalTheme::from_config(&config);

        let dracula = TerminalTheme::dracula();
        assert_eq!(theme.background.r, dracula.background.r);
        assert_eq!(theme.font_size, 18.0);
    }

    #[test]
    fn test_from_config_one_dark() {
        let config = create_custom_config("one_dark", 15.0, "Monaco");
        let theme = TerminalTheme::from_config(&config);

        let one_dark = TerminalTheme::one_dark();
        assert_eq!(theme.background.r, one_dark.background.r);
    }

    #[test]
    fn test_from_config_nord() {
        let config = create_custom_config("nord", 13.0, "Hack");
        let theme = TerminalTheme::from_config(&config);

        let nord = TerminalTheme::nord();
        assert_eq!(theme.background.r, nord.background.r);
    }

    #[test]
    fn test_from_config_unknown_defaults_to_dark() {
        let config = create_custom_config("unknown_theme", 14.0, "Arial");
        let theme = TerminalTheme::from_config(&config);

        let dark = TerminalTheme::dark();
        assert_eq!(theme.background.r, dark.background.r);
    }

    #[test]
    fn test_from_config_applies_font_settings() {
        let mut config = Config::default();
        config.terminal.font_family = "Custom Font".to_string();
        config.terminal.font_size = 20.0;

        let theme = TerminalTheme::from_config(&config);

        assert_eq!(theme.font_family.as_ref(), "Custom Font");
        assert_eq!(theme.font_size, 20.0);
        // line_height is fixed from base theme, not from config
        assert_eq!(theme.line_height, 1.4);
    }

    #[test]
    fn test_update_from_config_changes_theme() {
        let mut theme = TerminalTheme::dark();
        let original_bg = theme.background;

        // Update to light theme
        let config = create_custom_config("light", 16.0, "Consolas");
        theme.update_from_config(&config);

        // Background should change
        assert_ne!(theme.background.r, original_bg.r);

        // Light theme has white background
        let light = TerminalTheme::light();
        assert_eq!(theme.background.r, light.background.r);
    }

    #[test]
    fn test_update_from_config_changes_font() {
        let mut theme = TerminalTheme::dark();

        assert_eq!(theme.font_size, 14.0);

        let config = create_custom_config("dark", 24.0, "New Font");
        theme.update_from_config(&config);

        assert_eq!(theme.font_size, 24.0);
        assert_eq!(theme.font_family.as_ref(), "New Font");
    }

    #[test]
    fn test_update_from_config_preserves_ansi_colors() {
        let mut theme = TerminalTheme::dark();

        let config = create_custom_config("dracula", 14.0, "Mono");
        theme.update_from_config(&config);

        // ANSI colors should be from dracula theme
        let dracula = TerminalTheme::dracula();
        assert_eq!(theme.ansi_colors[0].r, dracula.ansi_colors[0].r);
        assert_eq!(theme.ansi_colors[1].r, dracula.ansi_colors[1].r);
    }

    #[test]
    fn test_hot_reload_scenario() {
        // Simulate a hot-reload scenario
        let initial_config = create_custom_config("dark", 14.0, "JetBrains Mono");
        let mut theme = TerminalTheme::from_config(&initial_config);

        // Initial state
        assert_eq!(theme.font_size, 14.0);
        let dark = TerminalTheme::dark();
        assert_eq!(theme.background.r, dark.background.r);

        // User changes config to dracula with larger font
        let updated_config = create_custom_config("dracula", 18.0, "Fira Code");
        theme.update_from_config(&updated_config);

        // Verify all changes applied
        assert_eq!(theme.font_size, 18.0);
        assert_eq!(theme.font_family.as_ref(), "Fira Code");
        let dracula = TerminalTheme::dracula();
        assert_eq!(theme.background.r, dracula.background.r);

        // User changes back to dark
        let final_config = create_custom_config("dark", 16.0, "Monaco");
        theme.update_from_config(&final_config);

        assert_eq!(theme.font_size, 16.0);
        assert_eq!(theme.font_family.as_ref(), "Monaco");
        assert_eq!(theme.background.r, dark.background.r);
    }

    #[test]
    fn test_line_height_uses_base_theme() {
        // line_height is determined by base theme, not config
        let mut theme = TerminalTheme::dark();
        assert_eq!(theme.line_height, 1.4);

        // Switching to dracula should keep line_height at 1.4 (base theme value)
        let config = create_custom_config("dracula", 14.0, "Mono");
        theme.update_from_config(&config);

        // line_height should remain the base theme value
        assert_eq!(theme.line_height, 1.4);
    }
}
