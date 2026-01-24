//! Comprehensive unit tests for title_bar component
//!
//! This module provides 100% test coverage for all public and testable
//! components of the title_bar module.

use gpui::px;
use zterm_ui::{
    LinuxWindowControls, PlatformStyle, TITLE_BAR_HEIGHT, TabInfo, TitleBar, TitleBarEvent,
    WindowsWindowControls,
};

/// Helper function to create TabInfo for tests
fn create_tab(id: usize, title: &str, active: bool, shell: &str, dir: &str) -> TabInfo {
    TabInfo::new(
        id,
        title.to_string(),
        active,
        shell.to_string(),
        dir.to_string(),
    )
}

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
        let tab = create_tab(1, "Terminal 1", true, "bash", "/home/user/projects");

        assert_eq!(tab.id, 1);
        assert_eq!(tab.title, "Terminal 1");
        assert!(tab.active);
        assert_eq!(tab.shell_name, "bash");
        assert_eq!(tab.working_directory, "/home/user/projects");
    }

    #[test]
    fn test_tab_info_clone() {
        let tab = create_tab(2, "Test Tab", false, "zsh", "/tmp");

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
            let tab = create_tab(0, "", true, "bash", &home.to_string_lossy());
            assert_eq!(tab.display_directory(), "~");
        }
    }

    #[test]
    fn test_display_directory_home_subdir() {
        if let Some(home) = dirs::home_dir() {
            let subdir = home.join("projects");
            let tab = create_tab(0, "", true, "bash", &subdir.to_string_lossy());
            assert_eq!(tab.display_directory(), "~/projects");
        }
    }

    #[test]
    fn test_display_directory_absolute_path() {
        let tab = create_tab(0, "", true, "bash", "/var/log/nginx");
        assert_eq!(tab.display_directory(), "nginx");
    }

    #[test]
    fn test_display_directory_single_component() {
        let tab = create_tab(0, "", true, "bash", "/tmp");
        assert_eq!(tab.display_directory(), "tmp");
    }

    #[test]
    fn test_display_directory_root() {
        let tab = create_tab(0, "", true, "bash", "/");
        assert_eq!(tab.display_directory(), "/");
    }

    #[test]
    fn test_display_directory_empty_path() {
        let tab = create_tab(0, "", true, "bash", "");
        assert_eq!(tab.display_directory(), "");
    }

    #[test]
    fn test_display_directory_nested_home_subdir() {
        if let Some(home) = dirs::home_dir() {
            let nested = home.join("a").join("b").join("c");
            let tab = create_tab(0, "", true, "bash", &nested.to_string_lossy());
            assert_eq!(tab.display_directory(), "~/c");
        }
    }

    #[test]
    fn test_tab_info_with_special_characters_in_path() {
        let tab = create_tab(0, "", true, "bash", "/path/with spaces/and-dashes");
        assert_eq!(tab.display_directory(), "and-dashes");
    }

    #[test]
    fn test_tab_info_inactive() {
        let tab = create_tab(5, "Inactive", false, "fish", "/usr/local");
        assert!(!tab.active);
    }

    #[test]
    fn test_tab_info_with_various_shells() {
        let shells = vec!["bash", "zsh", "fish", "pwsh", "cmd", "sh", "dash"];
        for shell in shells {
            let tab = create_tab(0, "", true, shell, "/tmp");
            assert_eq!(tab.shell_name, shell);
        }
    }

    #[test]
    fn test_tab_info_unicode_path() {
        let tab = create_tab(0, "", true, "bash", "/home/用户/项目");
        assert_eq!(tab.display_directory(), "项目");
    }

    #[test]
    fn test_tab_info_dotfile_path() {
        let tab = create_tab(0, "", true, "bash", "/home/user/.config");
        assert_eq!(tab.display_directory(), ".config");
    }

    #[test]
    fn test_tab_info_long_path() {
        let long_path = "/a".repeat(100);
        let tab = create_tab(0, "", true, "bash", &long_path);
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
            create_tab(0, "Tab 1", true, "bash", "/home"),
            create_tab(1, "Tab 2", false, "zsh", "/tmp"),
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
        let tabs = vec![create_tab(0, "Test", true, "bash", "/")];

        let title_bar = TitleBar::new().tabs(tabs);
        assert_eq!(title_bar.tabs.len(), 1);
    }

    #[test]
    fn test_title_bar_many_tabs() {
        let tabs: Vec<TabInfo> = (0..100)
            .map(|i| {
                create_tab(
                    i,
                    &format!("Tab {}", i),
                    i == 0,
                    "bash",
                    &format!("/path/{}", i),
                )
            })
            .collect();

        let title_bar = TitleBar::new().tabs(tabs);
        assert_eq!(title_bar.tabs.len(), 100);
    }

    #[test]
    fn test_title_bar_single_active_tab() {
        let tabs = vec![
            create_tab(0, "Tab 0", false, "bash", "/"),
            create_tab(1, "Tab 1", true, "bash", "/"),
            create_tab(2, "Tab 2", false, "bash", "/"),
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

// =============================================================================
// Scroll To Tab Tests
// =============================================================================

mod scroll_to_tab_tests {
    use super::*;

    #[test]
    fn test_scroll_to_tab_does_not_panic_with_empty_tabs() {
        let title_bar = TitleBar::new();
        // Should not panic even with no tabs
        title_bar.scroll_to_tab(0);
    }

    #[test]
    fn test_scroll_to_tab_with_single_tab() {
        let tabs = vec![create_tab(0, "Tab 0", true, "bash", "/")];
        let title_bar = TitleBar::new().tabs(tabs);
        // Should not panic
        title_bar.scroll_to_tab(0);
    }

    #[test]
    fn test_scroll_to_tab_with_multiple_tabs() {
        let tabs: Vec<TabInfo> = (0..10)
            .map(|i| {
                create_tab(
                    i,
                    &format!("Tab {}", i),
                    i == 0,
                    "bash",
                    &format!("/path/{}", i),
                )
            })
            .collect();
        let title_bar = TitleBar::new().tabs(tabs);

        // Test scrolling to various positions
        title_bar.scroll_to_tab(0);
        title_bar.scroll_to_tab(5);
        title_bar.scroll_to_tab(9);
    }

    #[test]
    fn test_scroll_to_tab_with_out_of_bounds_index() {
        let tabs = vec![create_tab(0, "Tab 0", true, "bash", "/")];
        let title_bar = TitleBar::new().tabs(tabs);

        // Should not panic with out of bounds index
        // ScrollHandle::scroll_to_item handles this gracefully
        title_bar.scroll_to_tab(100);
        title_bar.scroll_to_tab(usize::MAX);
    }

    #[test]
    fn test_scroll_to_tab_boundary_first() {
        let tabs: Vec<TabInfo> = (0..5)
            .map(|i| create_tab(i, &format!("Tab {}", i), false, "bash", "/"))
            .collect();
        let title_bar = TitleBar::new().tabs(tabs);

        // Scroll to first tab
        title_bar.scroll_to_tab(0);
    }

    #[test]
    fn test_scroll_to_tab_boundary_last() {
        let tabs: Vec<TabInfo> = (0..5)
            .map(|i| create_tab(i, &format!("Tab {}", i), false, "bash", "/"))
            .collect();
        let title_bar = TitleBar::new().tabs(tabs);

        // Scroll to last tab
        title_bar.scroll_to_tab(4);
    }

    #[test]
    fn test_scroll_to_tab_many_times() {
        let tabs: Vec<TabInfo> = (0..20)
            .map(|i| create_tab(i, &format!("Tab {}", i), false, "bash", "/"))
            .collect();
        let title_bar = TitleBar::new().tabs(tabs);

        // Simulate rapid tab switching
        for i in 0..20 {
            title_bar.scroll_to_tab(i);
        }
        // Reverse
        for i in (0..20).rev() {
            title_bar.scroll_to_tab(i);
        }
    }

    #[test]
    fn test_scroll_to_tab_with_large_tab_count() {
        // Stress test with many tabs
        let tabs: Vec<TabInfo> = (0..1000)
            .map(|i| {
                create_tab(
                    i,
                    &format!("Tab {}", i),
                    i == 0,
                    "bash",
                    &format!("/path/{}", i),
                )
            })
            .collect();
        let title_bar = TitleBar::new().tabs(tabs);

        // Should handle large tab counts
        title_bar.scroll_to_tab(0);
        title_bar.scroll_to_tab(500);
        title_bar.scroll_to_tab(999);
    }
}

mod integration_tests {
    use super::*;

    #[test]
    fn test_title_bar_with_tabs_workflow() {
        let home_dir = dirs::home_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "/home/user".to_string());
        let tabs = vec![
            create_tab(0, "Main", true, "bash", &home_dir),
            create_tab(1, "Project", false, "bash", "/home/user/projects/myapp"),
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
        let tabs1 = vec![create_tab(0, "First", true, "bash", "/")];

        let tabs2 = vec![
            create_tab(0, "Second", true, "zsh", "/tmp"),
            create_tab(1, "Third", false, "fish", "/var"),
        ];

        let title_bar = TitleBar::new().tabs(tabs1).tabs(tabs2);
        assert_eq!(title_bar.tabs.len(), 2);
        assert_eq!(title_bar.tabs[0].title, "Second");
    }

    #[test]
    fn test_tab_switching_with_scroll() {
        // Simulate a typical tab switching workflow
        let tabs: Vec<TabInfo> = (0..15)
            .map(|i| {
                create_tab(
                    i,
                    &format!("Terminal {}", i + 1),
                    i == 0,
                    "bash",
                    &format!("/home/user/project{}", i),
                )
            })
            .collect();

        let title_bar = TitleBar::new().tabs(tabs);

        // Simulate switching from first to last
        title_bar.scroll_to_tab(0);
        title_bar.scroll_to_tab(14);

        // Simulate switching back
        title_bar.scroll_to_tab(0);
    }

    #[test]
    fn test_scroll_after_new_tab() {
        // Simulate adding a new tab and scrolling to it
        let mut tabs: Vec<TabInfo> = (0..5)
            .map(|i| create_tab(i, &format!("Tab {}", i), false, "bash", "/"))
            .collect();

        let title_bar = TitleBar::new().tabs(tabs.clone());
        title_bar.scroll_to_tab(4);

        // Simulate new tab added
        tabs.push(create_tab(5, "New Tab", true, "bash", "/"));

        let title_bar = TitleBar::new().tabs(tabs);
        // Scroll to the newly added tab
        title_bar.scroll_to_tab(5);
    }

    #[test]
    fn test_scroll_after_close_tab() {
        // Simulate closing a tab and scrolling to the new active tab
        let tabs: Vec<TabInfo> = (0..5)
            .map(|i| create_tab(i, &format!("Tab {}", i), i == 2, "bash", "/"))
            .collect();

        let title_bar = TitleBar::new().tabs(tabs);
        title_bar.scroll_to_tab(2);

        // Simulate tab 2 closed, now tab 1 is active
        let remaining_tabs: Vec<TabInfo> = vec![0, 1, 3, 4]
            .into_iter()
            .map(|i| create_tab(i, &format!("Tab {}", i), i == 1, "bash", "/"))
            .collect();

        let title_bar = TitleBar::new().tabs(remaining_tabs);
        // Scroll to the new active tab (now at index 1)
        title_bar.scroll_to_tab(1);
    }
}

// =============================================================================
// Performance Tests
// =============================================================================

mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_scroll_performance_many_iterations() {
        let tabs: Vec<TabInfo> = (0..100)
            .map(|i| {
                create_tab(
                    i,
                    &format!("Tab {}", i),
                    i == 0,
                    "bash",
                    &format!("/path/{}", i),
                )
            })
            .collect();
        let title_bar = TitleBar::new().tabs(tabs);

        let start = Instant::now();
        for _ in 0..10000 {
            title_bar.scroll_to_tab(50);
        }
        let duration = start.elapsed();

        // Should complete in reasonable time (less than 1 second for 10k iterations)
        assert!(
            duration.as_millis() < 1000,
            "scroll_to_tab took too long: {:?}",
            duration
        );
    }

    #[test]
    fn test_title_bar_creation_performance() {
        let start = Instant::now();

        for _ in 0..1000 {
            let tabs: Vec<TabInfo> = (0..50)
                .map(|i| {
                    create_tab(
                        i,
                        &format!("Tab {}", i),
                        i == 0,
                        "bash",
                        &format!("/path/{}", i),
                    )
                })
                .collect();
            let _title_bar = TitleBar::new().tabs(tabs);
        }

        let duration = start.elapsed();

        // Creating 1000 title bars with 50 tabs each should be fast
        assert!(
            duration.as_millis() < 500,
            "TitleBar creation took too long: {:?}",
            duration
        );
    }

    #[test]
    fn test_tab_info_display_directory_performance() {
        let tabs: Vec<TabInfo> = (0..1000)
            .map(|i| {
                create_tab(
                    i,
                    &format!("Tab {}", i),
                    i == 0,
                    "bash",
                    &format!("/home/user/projects/subdir{}/deep/nested/path", i),
                )
            })
            .collect();

        let start = Instant::now();

        for tab in &tabs {
            let _ = tab.display_directory();
        }

        let duration = start.elapsed();

        // Calling display_directory 1000 times should be fast
        assert!(
            duration.as_millis() < 100,
            "display_directory took too long: {:?}",
            duration
        );
    }

    #[test]
    fn test_rapid_tab_switching_simulation() {
        let tabs: Vec<TabInfo> = (0..20)
            .map(|i| create_tab(i, &format!("Tab {}", i), false, "bash", "/"))
            .collect();
        let title_bar = TitleBar::new().tabs(tabs);

        let start = Instant::now();

        // Simulate rapid tab switching (like holding down next/prev tab shortcut)
        for _ in 0..1000 {
            for i in 0..20 {
                title_bar.scroll_to_tab(i);
            }
        }

        let duration = start.elapsed();

        // 20,000 scroll operations should complete quickly
        assert!(
            duration.as_millis() < 500,
            "Rapid tab switching took too long: {:?}",
            duration
        );
    }
}

// =============================================================================
// Edge Case Tests
// =============================================================================

mod edge_case_tests {
    use super::*;

    #[test]
    fn test_scroll_with_special_unicode_titles() {
        let tabs = vec![
            create_tab(0, "日本語タブ", true, "bash", "/home/ユーザー"),
            create_tab(1, "🚀 Rocket", false, "zsh", "/tmp/🎉"),
            create_tab(2, "العربية", false, "fish", "/مجلد"),
        ];

        let title_bar = TitleBar::new().tabs(tabs);

        // Should handle unicode tabs without issue
        title_bar.scroll_to_tab(0);
        title_bar.scroll_to_tab(1);
        title_bar.scroll_to_tab(2);

        assert_eq!(title_bar.tabs[0].display_directory(), "ユーザー");
        assert_eq!(title_bar.tabs[1].display_directory(), "🎉");
        assert_eq!(title_bar.tabs[2].display_directory(), "مجلد");
    }

    #[test]
    fn test_scroll_with_very_long_paths() {
        let long_path = "/".to_string() + &"a".repeat(1000);
        let tabs = vec![create_tab(0, "Long Path", true, "bash", &long_path)];

        let title_bar = TitleBar::new().tabs(tabs);
        title_bar.scroll_to_tab(0);
        // Should not panic
    }

    #[test]
    fn test_scroll_consecutive_same_index() {
        let tabs = vec![create_tab(0, "Tab", true, "bash", "/")];

        let title_bar = TitleBar::new().tabs(tabs);

        // Scrolling to the same index multiple times should be fine
        for _ in 0..100 {
            title_bar.scroll_to_tab(0);
        }
    }

    #[test]
    fn test_title_bar_with_all_inactive_tabs() {
        // Edge case: no active tab (unusual but possible during state transition)
        let tabs: Vec<TabInfo> = (0..5)
            .map(|i| create_tab(i, &format!("Tab {}", i), false, "bash", "/"))
            .collect();

        let title_bar = TitleBar::new().tabs(tabs);
        let active_count = title_bar.tabs.iter().filter(|t| t.active).count();
        assert_eq!(active_count, 0);

        // Scrolling should still work
        title_bar.scroll_to_tab(2);
    }

    #[test]
    fn test_title_bar_with_multiple_active_tabs() {
        // Edge case: multiple active tabs (unusual but should not crash)
        let tabs: Vec<TabInfo> = (0..5)
            .map(|i| create_tab(i, &format!("Tab {}", i), true, "bash", "/"))
            .collect();

        let title_bar = TitleBar::new().tabs(tabs);
        let active_count = title_bar.tabs.iter().filter(|t| t.active).count();
        assert_eq!(active_count, 5);

        title_bar.scroll_to_tab(2);
    }

    #[test]
    fn test_tab_info_with_empty_strings() {
        let tab = create_tab(0, "", true, "", "");

        assert_eq!(tab.display_directory(), "");
    }

    #[test]
    fn test_scroll_alternating_first_last() {
        let tabs: Vec<TabInfo> = (0..10)
            .map(|i| create_tab(i, &format!("Tab {}", i), false, "bash", "/"))
            .collect();

        let title_bar = TitleBar::new().tabs(tabs);

        // Rapidly alternate between first and last
        for _ in 0..100 {
            title_bar.scroll_to_tab(0);
            title_bar.scroll_to_tab(9);
        }
    }
}
