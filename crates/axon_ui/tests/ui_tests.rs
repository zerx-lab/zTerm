//! UI component tests for axon_ui
//!
//! These tests verify the behavior of UI components without requiring
//! a full GPUI window context.

use axon_ui::{TabInfo, TerminalTheme};

mod tab_info_tests {
    use super::*;

    #[test]
    fn test_tab_info_creation() {
        let tab = TabInfo {
            id: 1,
            title: "Terminal 1".to_string(),
            active: true,
        };
        assert_eq!(tab.id, 1);
        assert_eq!(tab.title, "Terminal 1");
        assert!(tab.active);
    }

    #[test]
    fn test_tab_info_clone() {
        let tab = TabInfo {
            id: 2,
            title: "Terminal 2".to_string(),
            active: false,
        };
        let cloned = tab.clone();
        assert_eq!(tab.id, cloned.id);
        assert_eq!(tab.title, cloned.title);
        assert_eq!(tab.active, cloned.active);
    }

    #[test]
    fn test_tab_info_inactive() {
        let tab = TabInfo {
            id: 0,
            title: "Inactive Tab".to_string(),
            active: false,
        };
        assert!(!tab.active);
    }

    #[test]
    fn test_tab_info_empty_title() {
        let tab = TabInfo {
            id: 0,
            title: String::new(),
            active: true,
        };
        assert!(tab.title.is_empty());
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
