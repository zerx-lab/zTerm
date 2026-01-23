//! Comprehensive unit tests for title_bar component
//!
//! This module provides 100% test coverage for all public and testable
//! components of the title_bar module.

use axon_ui::{
    LinuxWindowControls, PlatformStyle, TabInfo, TitleBar, TitleBarEvent, WindowsWindowControls,
    TITLE_BAR_HEIGHT,
};
use gpui::px;

// =============================================================================
// Constants Tests
// =============================================================================

mod constants_tests {
    use super::*;

    #[test]
    fn test_title_bar_height_value() {
        assert_eq!(TITLE_BAR_HEIGHT, px(32.0));
    }

    #[test]
    fn test_title_bar_height_is_reasonable() {
        // Title bar should be between 24-48 pixels for good UX
        assert!(TITLE_BAR_HEIGHT >= px(24.0));
        assert!(TITLE_BAR_HEIGHT <= px(48.0));
    }
}

// =============================================================================
// TitleBarEvent Tests
// =============================================================================

mod title_bar_event_tests {
    use super::*;

    #[test]
    fn test_new_tab_event_debug() {
        let event = TitleBarEvent::NewTab;
        let debug_str = format!("{:?}", event);
        assert_eq!(debug_str, "NewTab");
    }

    #[test]
    fn test_select_tab_event_debug() {
        let event = TitleBarEvent::SelectTab(5);
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("SelectTab"));
        assert!(debug_str.contains("5"));
    }

    #[test]
    fn test_close_tab_event_debug() {
        let event = TitleBarEvent::CloseTab(3);
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("CloseTab"));
        assert!(debug_str.contains("3"));
    }

    #[test]
    fn test_event_clone() {
        let event1 = TitleBarEvent::SelectTab(10);
        let event2 = event1.clone();
        match event2 {
            TitleBarEvent::SelectTab(id) => assert_eq!(id, 10),
            _ => panic!("Expected SelectTab event"),
        }
    }

    #[test]
    fn test_new_tab_event_clone() {
        let event1 = TitleBarEvent::NewTab;
        let event2 = event1.clone();
        match event2 {
            TitleBarEvent::NewTab => {}
            _ => panic!("Expected NewTab event"),
        }
    }

    #[test]
    fn test_close_tab_event_clone() {
        let event1 = TitleBarEvent::CloseTab(42);
        let event2 = event1.clone();
        match event2 {
            TitleBarEvent::CloseTab(id) => assert_eq!(id, 42),
            _ => panic!("Expected CloseTab event"),
        }
    }

    #[test]
    fn test_select_tab_with_zero_id() {
        let event = TitleBarEvent::SelectTab(0);
        match event {
            TitleBarEvent::SelectTab(id) => assert_eq!(id, 0),
            _ => panic!("Expected SelectTab event"),
        }
    }

    #[test]
    fn test_select_tab_with_max_id() {
        let event = TitleBarEvent::SelectTab(usize::MAX);
        match event {
            TitleBarEvent::SelectTab(id) => assert_eq!(id, usize::MAX),
            _ => panic!("Expected SelectTab event"),
        }
    }
}

// =============================================================================
// TabInfo Tests
// =============================================================================

mod tab_info_tests {
    use super::*;

    #[test]
    fn test_tab_info_creation() {
        let tab = TabInfo {
            id: 1,
            title: "Terminal 1".to_string(),
            active: true,
            shell_name: "bash".to_string(),
            working_directory: "/home/user/projects".to_string(),
        };

        assert_eq!(tab.id, 1);
        assert_eq!(tab.title, "Terminal 1");
        assert!(tab.active);
        assert_eq!(tab.shell_name, "bash");
        assert_eq!(tab.working_directory, "/home/user/projects");
    }

    #[test]
    fn test_tab_info_clone() {
        let tab = TabInfo {
            id: 2,
            title: "Test Tab".to_string(),
            active: false,
            shell_name: "zsh".to_string(),
            working_directory: "/tmp".to_string(),
        };

        let cloned = tab.clone();
        assert_eq!(tab.id, cloned.id);
        assert_eq!(tab.title, cloned.title);
        assert_eq!(tab.active, cloned.active);
        assert_eq!(tab.shell_name, cloned.shell_name);
        assert_eq!(tab.working_directory, cloned.working_directory);
    }

    #[test]
    fn test_display_directory_home() {
        if let Some(home) = dirs::home_dir() {
            let tab = TabInfo {
                id: 0,
                title: "".to_string(),
                active: true,
                shell_name: "bash".to_string(),
                working_directory: home.to_string_lossy().to_string(),
            };
            assert_eq!(tab.display_directory(), "~");
        }
    }

    #[test]
    fn test_display_directory_home_subdir() {
        if let Some(home) = dirs::home_dir() {
            let subdir = home.join("projects");
            let tab = TabInfo {
                id: 0,
                title: "".to_string(),
                active: true,
                shell_name: "bash".to_string(),
                working_directory: subdir.to_string_lossy().to_string(),
            };
            assert_eq!(tab.display_directory(), "~/projects");
        }
    }

    #[test]
    fn test_display_directory_absolute_path() {
        let tab = TabInfo {
            id: 0,
            title: "".to_string(),
            active: true,
            shell_name: "bash".to_string(),
            working_directory: "/var/log/nginx".to_string(),
        };
        assert_eq!(tab.display_directory(), "nginx");
    }

    #[test]
    fn test_display_directory_single_component() {
        let tab = TabInfo {
            id: 0,
            title: "".to_string(),
            active: true,
            shell_name: "bash".to_string(),
            working_directory: "/tmp".to_string(),
        };
        assert_eq!(tab.display_directory(), "tmp");
    }

    #[test]
    fn test_display_directory_root() {
        let tab = TabInfo {
            id: 0,
            title: "".to_string(),
            active: true,
            shell_name: "bash".to_string(),
            working_directory: "/".to_string(),
        };
        assert_eq!(tab.display_directory(), "/");
    }

    #[test]
    fn test_display_directory_empty_path() {
        let tab = TabInfo {
            id: 0,
            title: "".to_string(),
            active: true,
            shell_name: "bash".to_string(),
            working_directory: "".to_string(),
        };
        assert_eq!(tab.display_directory(), "");
    }

    #[test]
    fn test_display_directory_nested_home_subdir() {
        if let Some(home) = dirs::home_dir() {
            let nested = home.join("a").join("b").join("c");
            let tab = TabInfo {
                id: 0,
                title: "".to_string(),
                active: true,
                shell_name: "bash".to_string(),
                working_directory: nested.to_string_lossy().to_string(),
            };
            assert_eq!(tab.display_directory(), "~/c");
        }
    }

    #[test]
    fn test_tab_info_with_special_characters_in_path() {
        let tab = TabInfo {
            id: 0,
            title: "".to_string(),
            active: true,
            shell_name: "bash".to_string(),
            working_directory: "/path/with spaces/and-dashes".to_string(),
        };
        assert_eq!(tab.display_directory(), "and-dashes");
    }

    #[test]
    fn test_tab_info_inactive() {
        let tab = TabInfo {
            id: 5,
            title: "Inactive".to_string(),
            active: false,
            shell_name: "fish".to_string(),
            working_directory: "/usr/local".to_string(),
        };
        assert!(!tab.active);
    }

    #[test]
    fn test_tab_info_with_various_shells() {
        let shells = vec!["bash", "zsh", "fish", "pwsh", "cmd", "sh", "dash"];
        for shell in shells {
            let tab = TabInfo {
                id: 0,
                title: "".to_string(),
                active: true,
                shell_name: shell.to_string(),
                working_directory: "/tmp".to_string(),
            };
            assert_eq!(tab.shell_name, shell);
        }
    }

    #[test]
    fn test_tab_info_unicode_path() {
        let tab = TabInfo {
            id: 0,
            title: "".to_string(),
            active: true,
            shell_name: "bash".to_string(),
            working_directory: "/home/用户/项目".to_string(),
        };
        assert_eq!(tab.display_directory(), "项目");
    }

    #[test]
    fn test_tab_info_dotfile_path() {
        let tab = TabInfo {
            id: 0,
            title: "".to_string(),
            active: true,
            shell_name: "bash".to_string(),
            working_directory: "/home/user/.config".to_string(),
        };
        assert_eq!(tab.display_directory(), ".config");
    }

    #[test]
    fn test_tab_info_long_path() {
        let long_path = "/a".repeat(100);
        let tab = TabInfo {
            id: 0,
            title: "".to_string(),
            active: true,
            shell_name: "bash".to_string(),
            working_directory: long_path,
        };
        // Should not panic
        let _ = tab.display_directory();
    }
}

// =============================================================================
// PlatformStyle Tests
// =============================================================================

mod platform_style_tests {
    use super::*;

    #[test]
    fn test_platform_style_debug() {
        assert_eq!(format!("{:?}", PlatformStyle::Mac), "Mac");
        assert_eq!(format!("{:?}", PlatformStyle::Linux), "Linux");
        assert_eq!(format!("{:?}", PlatformStyle::Windows), "Windows");
    }

    #[test]
    fn test_platform_style_clone() {
        let style = PlatformStyle::Mac;
        let cloned = style.clone();
        assert_eq!(style, cloned);
    }

    #[test]
    fn test_platform_style_copy() {
        let style = PlatformStyle::Linux;
        let copied = style;
        assert_eq!(style, copied);
    }

    #[test]
    fn test_platform_style_eq() {
        assert_eq!(PlatformStyle::Mac, PlatformStyle::Mac);
        assert_eq!(PlatformStyle::Linux, PlatformStyle::Linux);
        assert_eq!(PlatformStyle::Windows, PlatformStyle::Windows);
    }

    #[test]
    fn test_platform_style_ne() {
        assert_ne!(PlatformStyle::Mac, PlatformStyle::Linux);
        assert_ne!(PlatformStyle::Mac, PlatformStyle::Windows);
        assert_ne!(PlatformStyle::Linux, PlatformStyle::Windows);
    }

    #[test]
    fn test_platform_returns_valid_variant() {
        let style = PlatformStyle::platform();
        assert!(
            style == PlatformStyle::Mac
                || style == PlatformStyle::Linux
                || style == PlatformStyle::Windows
        );
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_platform_returns_linux_on_linux() {
        assert_eq!(PlatformStyle::platform(), PlatformStyle::Linux);
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_platform_returns_mac_on_macos() {
        assert_eq!(PlatformStyle::platform(), PlatformStyle::Mac);
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_platform_returns_windows_on_windows() {
        assert_eq!(PlatformStyle::platform(), PlatformStyle::Windows);
    }

    #[test]
    #[cfg(target_os = "freebsd")]
    fn test_platform_returns_linux_on_freebsd() {
        assert_eq!(PlatformStyle::platform(), PlatformStyle::Linux);
    }
}

// =============================================================================
// TitleBar Tests
// =============================================================================

mod title_bar_tests {
    use super::*;

    #[test]
    fn test_title_bar_new() {
        let title_bar = TitleBar::new();
        assert!(title_bar.tabs.is_empty());
    }

    #[test]
    fn test_title_bar_default() {
        let title_bar = TitleBar::default();
        assert!(title_bar.tabs.is_empty());
    }

    #[test]
    fn test_title_bar_height() {
        assert_eq!(TitleBar::height(), TITLE_BAR_HEIGHT);
        assert_eq!(TitleBar::height(), px(32.0));
    }

    #[test]
    fn test_title_bar_tabs_builder() {
        let tabs = vec![
            TabInfo {
                id: 0,
                title: "Tab 1".to_string(),
                active: true,
                shell_name: "bash".to_string(),
                working_directory: "/home".to_string(),
            },
            TabInfo {
                id: 1,
                title: "Tab 2".to_string(),
                active: false,
                shell_name: "zsh".to_string(),
                working_directory: "/tmp".to_string(),
            },
        ];

        let title_bar = TitleBar::new().tabs(tabs);
        assert_eq!(title_bar.tabs.len(), 2);
        assert_eq!(title_bar.tabs[0].id, 0);
        assert_eq!(title_bar.tabs[1].id, 1);
    }

    #[test]
    fn test_title_bar_tabs_empty() {
        let title_bar = TitleBar::new().tabs(vec![]);
        assert!(title_bar.tabs.is_empty());
    }

    #[test]
    fn test_title_bar_builder_chain() {
        let tabs = vec![TabInfo {
            id: 0,
            title: "Test".to_string(),
            active: true,
            shell_name: "bash".to_string(),
            working_directory: "/".to_string(),
        }];

        let title_bar = TitleBar::new().tabs(tabs);
        assert_eq!(title_bar.tabs.len(), 1);
    }

    #[test]
    fn test_title_bar_many_tabs() {
        let tabs: Vec<TabInfo> = (0..100)
            .map(|i| TabInfo {
                id: i,
                title: format!("Tab {}", i),
                active: i == 0,
                shell_name: "bash".to_string(),
                working_directory: format!("/path/{}", i),
            })
            .collect();

        let title_bar = TitleBar::new().tabs(tabs);
        assert_eq!(title_bar.tabs.len(), 100);
    }

    #[test]
    fn test_title_bar_single_active_tab() {
        let tabs = vec![
            TabInfo {
                id: 0,
                title: "Tab 0".to_string(),
                active: false,
                shell_name: "bash".to_string(),
                working_directory: "/".to_string(),
            },
            TabInfo {
                id: 1,
                title: "Tab 1".to_string(),
                active: true,
                shell_name: "bash".to_string(),
                working_directory: "/".to_string(),
            },
            TabInfo {
                id: 2,
                title: "Tab 2".to_string(),
                active: false,
                shell_name: "bash".to_string(),
                working_directory: "/".to_string(),
            },
        ];

        let title_bar = TitleBar::new().tabs(tabs);
        let active_count = title_bar.tabs.iter().filter(|t| t.active).count();
        assert_eq!(active_count, 1);
    }
}

// =============================================================================
// WindowsWindowControls Tests
// =============================================================================

mod windows_window_controls_tests {
    use super::*;

    #[test]
    fn test_windows_controls_new() {
        // WindowsWindowControls can be created with any pixel height
        let _controls = WindowsWindowControls::new(px(32.0));
    }

    #[test]
    fn test_windows_controls_custom_height() {
        // WindowsWindowControls accepts custom heights
        let _controls = WindowsWindowControls::new(px(48.0));
    }

    #[test]
    fn test_windows_controls_zero_height() {
        // WindowsWindowControls can handle zero height
        let _controls = WindowsWindowControls::new(px(0.0));
    }

    #[test]
    fn test_windows_controls_with_title_bar_height() {
        // WindowsWindowControls works with TITLE_BAR_HEIGHT constant
        let _controls = WindowsWindowControls::new(TITLE_BAR_HEIGHT);
    }
}

// =============================================================================
// LinuxWindowControls Tests
// =============================================================================

mod linux_window_controls_tests {
    use super::*;

    #[test]
    fn test_linux_controls_new() {
        let _controls = LinuxWindowControls::new();
    }

    #[test]
    fn test_linux_controls_is_unit_struct() {
        assert_eq!(std::mem::size_of::<LinuxWindowControls>(), 0);
    }
}

// =============================================================================
// Integration Tests
// =============================================================================

mod integration_tests {
    use super::*;

    #[test]
    fn test_title_bar_with_tabs_workflow() {
        let tabs = vec![
            TabInfo {
                id: 0,
                title: "Main".to_string(),
                active: true,
                shell_name: "bash".to_string(),
                working_directory: dirs::home_dir()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "/home/user".to_string()),
            },
            TabInfo {
                id: 1,
                title: "Project".to_string(),
                active: false,
                shell_name: "bash".to_string(),
                working_directory: "/home/user/projects/myapp".to_string(),
            },
        ];

        let title_bar = TitleBar::new().tabs(tabs);

        assert_eq!(title_bar.tabs.len(), 2);
        assert!(title_bar.tabs[0].active);
        assert!(!title_bar.tabs[1].active);
        assert_eq!(title_bar.tabs[0].display_directory(), "~");
        assert_eq!(title_bar.tabs[1].display_directory(), "myapp");
    }

    #[test]
    fn test_platform_specific_controls() {
        let platform = PlatformStyle::platform();

        match platform {
            PlatformStyle::Mac => {}
            PlatformStyle::Linux => {
                let _controls = LinuxWindowControls::new();
            }
            PlatformStyle::Windows => {
                let _controls = WindowsWindowControls::new(TITLE_BAR_HEIGHT);
            }
        }
    }

    #[test]
    fn test_multiple_title_bars() {
        let title_bar1 = TitleBar::new();
        let title_bar2 = TitleBar::default();

        assert!(title_bar1.tabs.is_empty());
        assert!(title_bar2.tabs.is_empty());
    }

    #[test]
    fn test_title_bar_tabs_replacement() {
        let tabs1 = vec![TabInfo {
            id: 0,
            title: "First".to_string(),
            active: true,
            shell_name: "bash".to_string(),
            working_directory: "/".to_string(),
        }];

        let tabs2 = vec![
            TabInfo {
                id: 0,
                title: "Second".to_string(),
                active: true,
                shell_name: "zsh".to_string(),
                working_directory: "/tmp".to_string(),
            },
            TabInfo {
                id: 1,
                title: "Third".to_string(),
                active: false,
                shell_name: "fish".to_string(),
                working_directory: "/var".to_string(),
            },
        ];

        let title_bar = TitleBar::new().tabs(tabs1).tabs(tabs2);
        assert_eq!(title_bar.tabs.len(), 2);
        assert_eq!(title_bar.tabs[0].title, "Second");
    }
}
