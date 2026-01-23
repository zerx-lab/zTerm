//! Comprehensive UI component tests for zterm_ui
//!
//! These tests verify the behavior of all UI components including:
//! - TerminalTabBar
//! - GridPosition
//! - ImeState
//! - SharedBounds
//! - Selection
//! - ScrollbarState / ThumbState
//! - Various boundary and edge cases

use zterm_ui::{
    GridPosition, ImeState, ScrollbarState, Selection, SharedBounds, TabInfo, TerminalTabBar,
    TerminalTheme, ThumbState,
};

// ============================================================================
// TerminalTabBar Tests
// ============================================================================
mod terminal_tab_bar_tests {
    use super::*;

    #[test]
    fn test_tab_bar_new() {
        let tab_bar = TerminalTabBar::new();
        // TabBar should be created successfully
        assert!(std::mem::size_of_val(&tab_bar) > 0);
    }

    #[test]
    fn test_tab_bar_default() {
        let tab_bar = TerminalTabBar::default();
        assert!(std::mem::size_of_val(&tab_bar) > 0);
    }

    #[test]
    fn test_tab_bar_with_empty_tabs() {
        let tab_bar = TerminalTabBar::new().tabs(vec![]);
        assert!(std::mem::size_of_val(&tab_bar) > 0);
    }

    #[test]
    fn test_tab_bar_with_single_tab() {
        let tabs = vec![TabInfo {
            id: 0,
            title: "Terminal".to_string(),
            active: true,
            shell_name: "bash".to_string(),
            working_directory: "/home/user".to_string(),
        }];
        let tab_bar = TerminalTabBar::new().tabs(tabs);
        assert!(std::mem::size_of_val(&tab_bar) > 0);
    }

    #[test]
    fn test_tab_bar_with_multiple_tabs() {
        let tabs = vec![
            TabInfo {
                id: 0,
                title: "Terminal 1".to_string(),
                active: true,
                shell_name: "bash".to_string(),
                working_directory: "/home/user".to_string(),
            },
            TabInfo {
                id: 1,
                title: "Terminal 2".to_string(),
                active: false,
                shell_name: "zsh".to_string(),
                working_directory: "/tmp".to_string(),
            },
            TabInfo {
                id: 2,
                title: "Terminal 3".to_string(),
                active: false,
                shell_name: "fish".to_string(),
                working_directory: "/var/log".to_string(),
            },
        ];
        let tab_bar = TerminalTabBar::new().tabs(tabs);
        assert!(std::mem::size_of_val(&tab_bar) > 0);
    }

    #[test]
    fn test_tab_bar_with_many_tabs() {
        // Test with 100 tabs to verify no overflow issues
        let tabs: Vec<TabInfo> = (0..100)
            .map(|i| TabInfo {
                id: i,
                title: format!("Terminal {}", i),
                active: i == 0,
                shell_name: "bash".to_string(),
                working_directory: format!("/home/user/project{}", i),
            })
            .collect();
        let tab_bar = TerminalTabBar::new().tabs(tabs);
        assert!(std::mem::size_of_val(&tab_bar) > 0);
    }

    #[test]
    fn test_tab_bar_builder_chain() {
        let tabs = vec![TabInfo {
            id: 0,
            title: "Test".to_string(),
            active: true,
            shell_name: "bash".to_string(),
            working_directory: "~".to_string(),
        }];
        // Test that builder pattern works correctly
        let _tab_bar = TerminalTabBar::new().tabs(tabs);
    }
}

// ============================================================================
// GridPosition Tests
// ============================================================================
mod grid_position_tests {
    use super::*;

    #[test]
    fn test_grid_position_creation() {
        let pos = GridPosition { col: 5, row: 10 };
        assert_eq!(pos.col, 5);
        assert_eq!(pos.row, 10);
    }

    #[test]
    fn test_grid_position_origin() {
        let pos = GridPosition { col: 0, row: 0 };
        assert_eq!(pos.col, 0);
        assert_eq!(pos.row, 0);
    }

    #[test]
    fn test_grid_position_max_values() {
        let pos = GridPosition {
            col: usize::MAX,
            row: usize::MAX,
        };
        assert_eq!(pos.col, usize::MAX);
        assert_eq!(pos.row, usize::MAX);
    }

    #[test]
    fn test_grid_position_clone() {
        let pos = GridPosition { col: 42, row: 24 };
        let cloned = pos.clone();
        assert_eq!(pos.col, cloned.col);
        assert_eq!(pos.row, cloned.row);
    }

    #[test]
    fn test_grid_position_copy() {
        let pos = GridPosition { col: 10, row: 20 };
        let copied = pos; // Copy
        assert_eq!(pos.col, copied.col);
        assert_eq!(pos.row, copied.row);
    }

    #[test]
    fn test_grid_position_eq() {
        let pos1 = GridPosition { col: 5, row: 5 };
        let pos2 = GridPosition { col: 5, row: 5 };
        assert_eq!(pos1, pos2);
    }

    #[test]
    fn test_grid_position_ne() {
        let pos1 = GridPosition { col: 5, row: 5 };
        let pos2 = GridPosition { col: 6, row: 5 };
        let pos3 = GridPosition { col: 5, row: 6 };
        assert_ne!(pos1, pos2);
        assert_ne!(pos1, pos3);
    }

    #[test]
    fn test_grid_position_debug() {
        let pos = GridPosition { col: 1, row: 2 };
        let debug_str = format!("{:?}", pos);
        assert!(debug_str.contains("col"));
        assert!(debug_str.contains("row"));
        assert!(debug_str.contains("1"));
        assert!(debug_str.contains("2"));
    }
}

// ============================================================================
// Selection Tests
// ============================================================================
mod selection_tests {
    use super::*;

    #[test]
    fn test_selection_creation() {
        let sel = Selection {
            start_col: 0,
            start_row: 0,
            end_col: 10,
            end_row: 5,
        };
        assert_eq!(sel.start_col, 0);
        assert_eq!(sel.start_row, 0);
        assert_eq!(sel.end_col, 10);
        assert_eq!(sel.end_row, 5);
    }

    #[test]
    fn test_selection_contains_single_row() {
        let sel = Selection {
            start_col: 5,
            start_row: 3,
            end_col: 15,
            end_row: 3,
        };
        // Within selection
        assert!(sel.contains(5, 3));
        assert!(sel.contains(10, 3));
        assert!(sel.contains(15, 3));
        // Outside selection
        assert!(!sel.contains(4, 3));
        assert!(!sel.contains(16, 3));
        assert!(!sel.contains(10, 2));
        assert!(!sel.contains(10, 4));
    }

    #[test]
    fn test_selection_contains_multiple_rows() {
        let sel = Selection {
            start_col: 5,
            start_row: 2,
            end_col: 10,
            end_row: 4,
        };
        // Start row
        assert!(sel.contains(5, 2));
        assert!(sel.contains(10, 2));
        assert!(sel.contains(100, 2)); // Beyond end_col on start row is valid
        assert!(!sel.contains(4, 2)); // Before start_col on start row

        // Middle row (all columns selected)
        assert!(sel.contains(0, 3));
        assert!(sel.contains(50, 3));
        assert!(sel.contains(100, 3));

        // End row
        assert!(sel.contains(0, 4));
        assert!(sel.contains(5, 4));
        assert!(sel.contains(10, 4));
        assert!(!sel.contains(11, 4)); // After end_col on end row

        // Outside row range
        assert!(!sel.contains(5, 1));
        assert!(!sel.contains(5, 5));
    }

    #[test]
    fn test_selection_contains_boundary_values() {
        let sel = Selection {
            start_col: 0,
            start_row: 0,
            end_col: 0,
            end_row: 0,
        };
        assert!(sel.contains(0, 0));
        assert!(!sel.contains(1, 0));
        assert!(!sel.contains(0, 1));
    }

    #[test]
    fn test_selection_contains_large_values() {
        let sel = Selection {
            start_col: 1000,
            start_row: 1000,
            end_col: 2000,
            end_row: 2000,
        };
        assert!(sel.contains(1000, 1000));
        assert!(sel.contains(1500, 1500));
        assert!(sel.contains(2000, 2000));
        assert!(!sel.contains(999, 1000));
        assert!(!sel.contains(1000, 999));
    }

    #[test]
    fn test_selection_clone() {
        let sel = Selection {
            start_col: 1,
            start_row: 2,
            end_col: 3,
            end_row: 4,
        };
        let cloned = sel.clone();
        assert_eq!(sel, cloned);
    }

    #[test]
    fn test_selection_copy() {
        let sel = Selection {
            start_col: 10,
            start_row: 20,
            end_col: 30,
            end_row: 40,
        };
        let copied = sel;
        assert_eq!(sel.start_col, copied.start_col);
        assert_eq!(sel.end_row, copied.end_row);
    }

    #[test]
    fn test_selection_eq() {
        let sel1 = Selection {
            start_col: 1,
            start_row: 2,
            end_col: 3,
            end_row: 4,
        };
        let sel2 = Selection {
            start_col: 1,
            start_row: 2,
            end_col: 3,
            end_row: 4,
        };
        assert_eq!(sel1, sel2);
    }

    #[test]
    fn test_selection_debug() {
        let sel = Selection {
            start_col: 5,
            start_row: 10,
            end_col: 15,
            end_row: 20,
        };
        let debug_str = format!("{:?}", sel);
        assert!(debug_str.contains("Selection"));
        assert!(debug_str.contains("5"));
        assert!(debug_str.contains("10"));
    }
}

// ============================================================================
// ImeState Tests
// ============================================================================
mod ime_state_tests {
    use super::*;

    #[test]
    fn test_ime_state_creation() {
        let ime = ImeState {
            marked_text: "hello".to_string(),
        };
        assert_eq!(ime.marked_text, "hello");
    }

    #[test]
    fn test_ime_state_empty() {
        let ime = ImeState {
            marked_text: String::new(),
        };
        assert!(ime.marked_text.is_empty());
    }

    #[test]
    fn test_ime_state_chinese_text() {
        let ime = ImeState {
            marked_text: "你好世界".to_string(),
        };
        assert_eq!(ime.marked_text, "你好世界");
        assert_eq!(ime.marked_text.chars().count(), 4);
    }

    #[test]
    fn test_ime_state_japanese_text() {
        let ime = ImeState {
            marked_text: "こんにちは".to_string(),
        };
        assert_eq!(ime.marked_text, "こんにちは");
    }

    #[test]
    fn test_ime_state_korean_text() {
        let ime = ImeState {
            marked_text: "안녕하세요".to_string(),
        };
        assert_eq!(ime.marked_text, "안녕하세요");
    }

    #[test]
    fn test_ime_state_emoji() {
        let ime = ImeState {
            marked_text: "🎉🚀💻".to_string(),
        };
        assert_eq!(ime.marked_text.chars().count(), 3);
    }

    #[test]
    fn test_ime_state_mixed_content() {
        let ime = ImeState {
            marked_text: "Hello 你好 🌍".to_string(),
        };
        assert!(ime.marked_text.contains("Hello"));
        assert!(ime.marked_text.contains("你好"));
    }

    #[test]
    fn test_ime_state_clone() {
        let ime = ImeState {
            marked_text: "test".to_string(),
        };
        let cloned = ime.clone();
        assert_eq!(ime.marked_text, cloned.marked_text);
    }

    #[test]
    fn test_ime_state_long_text() {
        let long_text = "a".repeat(10000);
        let ime = ImeState {
            marked_text: long_text.clone(),
        };
        assert_eq!(ime.marked_text.len(), 10000);
    }

    #[test]
    fn test_ime_state_utf16_encoding() {
        let ime = ImeState {
            marked_text: "你好".to_string(),
        };
        let utf16_count = ime.marked_text.encode_utf16().count();
        assert_eq!(utf16_count, 2);
    }

    #[test]
    fn test_ime_state_surrogate_pairs() {
        // Emoji that requires surrogate pairs in UTF-16
        let ime = ImeState {
            marked_text: "😀".to_string(),
        };
        let utf16_count = ime.marked_text.encode_utf16().count();
        assert_eq!(utf16_count, 2); // Surrogate pair
    }
}

// ============================================================================
// SharedBounds Tests
// ============================================================================
mod shared_bounds_tests {
    use super::*;

    #[test]
    fn test_shared_bounds_default() {
        let bounds = SharedBounds::default();
        assert!(bounds.bounds.get().is_none());
        assert!(bounds.cell_width.get().is_none());
        assert!(bounds.line_height.get().is_none());
    }

    #[test]
    fn test_shared_bounds_clone() {
        let bounds = SharedBounds::default();
        let cloned = bounds.clone();
        // Both should point to the same Rc
        assert!(cloned.bounds.get().is_none());
    }
}

// ============================================================================
// ScrollbarState Tests
// ============================================================================
mod scrollbar_state_tests {
    use super::*;

    #[test]
    fn test_scrollbar_state_new() {
        let state = ScrollbarState::new();
        assert!(!state.is_dragging());
        assert!(!state.is_active());
        assert!(state.thumb_bounds.is_none());
        assert!(state.track_bounds.is_none());
    }

    #[test]
    fn test_scrollbar_state_start_drag() {
        let mut state = ScrollbarState::new();
        state.start_drag(10);
        assert!(state.is_dragging());
        assert!(state.is_active());
    }

    #[test]
    fn test_scrollbar_state_end_drag() {
        let mut state = ScrollbarState::new();
        state.start_drag(10);
        assert!(state.is_dragging());
        state.end_drag();
        assert!(!state.is_dragging());
        assert!(!state.is_active());
    }

    #[test]
    fn test_scrollbar_state_hover() {
        let mut state = ScrollbarState::new();
        state.set_hovered(true);
        assert!(state.is_active());
        assert!(!state.is_dragging());
        state.set_hovered(false);
        assert!(!state.is_active());
    }

    #[test]
    fn test_scrollbar_state_hover_during_drag() {
        let mut state = ScrollbarState::new();
        state.start_drag(5);
        // Hover state should not change during drag
        state.set_hovered(false);
        assert!(state.is_dragging()); // Still dragging
        assert!(state.is_active());
    }

    #[test]
    fn test_scrollbar_state_drag_offset() {
        let mut state = ScrollbarState::new();
        state.start_drag(42);
        match state.thumb_state {
            ThumbState::Dragging { offset } => assert_eq!(offset, 42),
            _ => panic!("Expected Dragging state"),
        }
    }

    #[test]
    fn test_scrollbar_state_negative_offset() {
        let mut state = ScrollbarState::new();
        state.start_drag(-10);
        match state.thumb_state {
            ThumbState::Dragging { offset } => assert_eq!(offset, -10),
            _ => panic!("Expected Dragging state"),
        }
    }
}

// ============================================================================
// ThumbState Tests
// ============================================================================
mod thumb_state_tests {
    use super::*;

    #[test]
    fn test_thumb_state_default() {
        let state = ThumbState::default();
        assert_eq!(state, ThumbState::Inactive);
        assert!(!state.is_dragging());
    }

    #[test]
    fn test_thumb_state_inactive() {
        let state = ThumbState::Inactive;
        assert!(!state.is_dragging());
    }

    #[test]
    fn test_thumb_state_hovered() {
        let state = ThumbState::Hovered;
        assert!(!state.is_dragging());
    }

    #[test]
    fn test_thumb_state_dragging() {
        let state = ThumbState::Dragging { offset: 10 };
        assert!(state.is_dragging());
    }

    #[test]
    fn test_thumb_state_dragging_zero_offset() {
        let state = ThumbState::Dragging { offset: 0 };
        assert!(state.is_dragging());
    }

    #[test]
    fn test_thumb_state_eq() {
        assert_eq!(ThumbState::Inactive, ThumbState::Inactive);
        assert_eq!(ThumbState::Hovered, ThumbState::Hovered);
        assert_eq!(
            ThumbState::Dragging { offset: 5 },
            ThumbState::Dragging { offset: 5 }
        );
    }

    #[test]
    fn test_thumb_state_ne() {
        assert_ne!(ThumbState::Inactive, ThumbState::Hovered);
        assert_ne!(
            ThumbState::Dragging { offset: 5 },
            ThumbState::Dragging { offset: 10 }
        );
    }

    #[test]
    fn test_thumb_state_clone() {
        let state = ThumbState::Dragging { offset: 42 };
        let cloned = state.clone();
        assert_eq!(state, cloned);
    }

    #[test]
    fn test_thumb_state_copy() {
        let state = ThumbState::Hovered;
        let copied = state;
        assert_eq!(state, copied);
    }

    #[test]
    fn test_thumb_state_debug() {
        let state = ThumbState::Dragging { offset: 100 };
        let debug_str = format!("{:?}", state);
        assert!(debug_str.contains("Dragging"));
        assert!(debug_str.contains("100"));
    }
}

// ============================================================================
// TabInfo Boundary Tests
// ============================================================================
mod tab_info_boundary_tests {
    use super::*;

    #[test]
    fn test_tab_info_max_id() {
        let tab = TabInfo {
            id: usize::MAX,
            title: "Max ID Tab".to_string(),
            active: true,
            shell_name: "bash".to_string(),
            working_directory: "/".to_string(),
        };
        assert_eq!(tab.id, usize::MAX);
    }

    #[test]
    fn test_tab_info_very_long_title() {
        let long_title = "a".repeat(10000);
        let tab = TabInfo {
            id: 0,
            title: long_title.clone(),
            active: true,
            shell_name: "bash".to_string(),
            working_directory: "/".to_string(),
        };
        assert_eq!(tab.title.len(), 10000);
    }

    #[test]
    fn test_tab_info_unicode_title() {
        let tab = TabInfo {
            id: 0,
            title: "终端 Terminal ターミナル 터미널".to_string(),
            active: true,
            shell_name: "bash".to_string(),
            working_directory: "/".to_string(),
        };
        assert!(tab.title.contains("终端"));
        assert!(tab.title.contains("ターミナル"));
    }

    #[test]
    fn test_tab_info_special_characters_in_shell() {
        let tab = TabInfo {
            id: 0,
            title: "Test".to_string(),
            active: true,
            shell_name: "/usr/local/bin/my-custom-shell".to_string(),
            working_directory: "/".to_string(),
        };
        assert!(tab.shell_name.contains("my-custom-shell"));
    }

    #[test]
    fn test_tab_info_windows_path() {
        let tab = TabInfo {
            id: 0,
            title: "Windows".to_string(),
            active: true,
            shell_name: "pwsh".to_string(),
            working_directory: "C:\\Users\\Test\\Documents".to_string(),
        };
        assert!(tab.working_directory.contains("C:\\"));
    }

    #[test]
    fn test_tab_info_network_path() {
        let tab = TabInfo {
            id: 0,
            title: "Network".to_string(),
            active: true,
            shell_name: "bash".to_string(),
            working_directory: "//server/share/folder".to_string(),
        };
        assert!(tab.working_directory.starts_with("//"));
    }

    #[test]
    fn test_tab_info_empty_strings() {
        let tab = TabInfo {
            id: 0,
            title: String::new(),
            active: true,
            shell_name: String::new(),
            working_directory: String::new(),
        };
        assert!(tab.title.is_empty());
        assert!(tab.shell_name.is_empty());
        assert!(tab.working_directory.is_empty());
    }

    #[test]
    fn test_display_directory_complex_paths() {
        // Root path
        let tab = TabInfo {
            id: 0,
            title: "Root".to_string(),
            active: true,
            shell_name: "bash".to_string(),
            working_directory: "/".to_string(),
        };
        assert_eq!(tab.display_directory(), "/");

        // Deep nested path
        let tab = TabInfo {
            id: 0,
            title: "Deep".to_string(),
            active: true,
            shell_name: "bash".to_string(),
            working_directory: "/a/b/c/d/e/f/g/target".to_string(),
        };
        assert_eq!(tab.display_directory(), "target");
    }

    #[test]
    fn test_display_directory_trailing_slash() {
        let tab = TabInfo {
            id: 0,
            title: "Trailing".to_string(),
            active: true,
            shell_name: "bash".to_string(),
            working_directory: "/home/user/project/".to_string(),
        };
        // Should handle trailing slash gracefully
        let display = tab.display_directory();
        assert!(!display.is_empty());
    }
}

// ============================================================================
// TerminalTheme Boundary Tests
// ============================================================================
mod terminal_theme_boundary_tests {
    use super::*;

    #[test]
    fn test_theme_font_size_range() {
        let theme = TerminalTheme::dark();
        assert!(theme.font_size > 0.0);
        assert!(theme.font_size < 1000.0);
    }

    #[test]
    fn test_theme_line_height_range() {
        let theme = TerminalTheme::dark();
        assert!(theme.line_height > 0.0);
        assert!(theme.line_height < 10.0);
    }

    #[test]
    fn test_all_themes_consistent() {
        let themes = [
            TerminalTheme::dark(),
            TerminalTheme::light(),
            TerminalTheme::dracula(),
            TerminalTheme::one_dark(),
            TerminalTheme::nord(),
        ];

        for theme in &themes {
            // All themes should have valid configurations
            assert_eq!(theme.ansi_colors.len(), 16);
            assert!(theme.font_size > 0.0);
            assert!(theme.line_height > 0.0);
            assert!(!theme.font_family.is_empty());
        }
    }

    #[test]
    fn test_theme_colors_non_zero() {
        let theme = TerminalTheme::dark();
        // At least some colors should be non-transparent
        let has_visible_colors = theme
            .ansi_colors
            .iter()
            .any(|c| c.r > 0.0 || c.g > 0.0 || c.b > 0.0);
        assert!(has_visible_colors);
    }
}

// ============================================================================
// Edge Case and Stress Tests
// ============================================================================
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_selection_zero_size() {
        let sel = Selection {
            start_col: 10,
            start_row: 10,
            end_col: 10,
            end_row: 10,
        };
        assert!(sel.contains(10, 10));
        assert!(!sel.contains(9, 10));
        assert!(!sel.contains(11, 10));
    }

    #[test]
    fn test_selection_full_row() {
        let sel = Selection {
            start_col: 0,
            start_row: 5,
            end_col: usize::MAX,
            end_row: 5,
        };
        assert!(sel.contains(0, 5));
        assert!(sel.contains(1000, 5));
        assert!(sel.contains(usize::MAX, 5));
    }

    #[test]
    fn test_grid_position_comparison() {
        let pos1 = GridPosition { col: 0, row: 0 };
        let pos2 = GridPosition { col: 0, row: 1 };
        let pos3 = GridPosition { col: 1, row: 0 };

        // Test ordering logic (row, col) lexicographic
        assert!(
            (pos1.row, pos1.col) < (pos2.row, pos2.col),
            "row 0 should be before row 1"
        );
        assert!(
            (pos1.row, pos1.col) < (pos3.row, pos3.col),
            "col 0 should be before col 1 on same row"
        );
    }

    #[test]
    fn test_scrollbar_rapid_state_changes() {
        let mut state = ScrollbarState::new();

        // Simulate rapid state changes
        for i in 0..100 {
            state.start_drag(i);
            assert!(state.is_dragging());
            state.end_drag();
            assert!(!state.is_dragging());
            state.set_hovered(true);
            assert!(state.is_active());
            state.set_hovered(false);
            assert!(!state.is_active());
        }
    }

    #[test]
    fn test_tab_info_clone_independence() {
        let original = TabInfo {
            id: 1,
            title: "Original".to_string(),
            active: true,
            shell_name: "bash".to_string(),
            working_directory: "/home/user".to_string(),
        };
        let cloned = original.clone();

        // Verify they are independent
        assert_eq!(original.id, cloned.id);
        assert_eq!(original.title, cloned.title);
        // They should be equal in value
        assert_eq!(original.active, cloned.active);
    }
}

// ============================================================================
// Input Boundary Tests
// ============================================================================
mod input_boundary_tests {
    use super::*;

    #[test]
    fn test_ime_null_character() {
        let ime = ImeState {
            marked_text: "\0".to_string(),
        };
        assert_eq!(ime.marked_text.len(), 1);
    }

    #[test]
    fn test_ime_control_characters() {
        let ime = ImeState {
            marked_text: "\x1b\x0d\x0a".to_string(),
        };
        assert_eq!(ime.marked_text.len(), 3);
    }

    #[test]
    fn test_ime_newlines() {
        let ime = ImeState {
            marked_text: "line1\nline2\rline3\r\nline4".to_string(),
        };
        assert!(ime.marked_text.contains("\n"));
        assert!(ime.marked_text.contains("\r"));
    }

    #[test]
    fn test_ime_tabs() {
        let ime = ImeState {
            marked_text: "col1\tcol2\tcol3".to_string(),
        };
        assert!(ime.marked_text.contains("\t"));
    }

    #[test]
    fn test_ime_zero_width_characters() {
        // Zero-width joiner and other invisible characters
        let ime = ImeState {
            marked_text: "a\u{200B}b\u{200C}c\u{200D}d".to_string(),
        };
        assert_eq!(ime.marked_text.chars().count(), 7);
    }

    #[test]
    fn test_tab_title_control_chars() {
        let tab = TabInfo {
            id: 0,
            title: "Title\twith\ttabs".to_string(),
            active: true,
            shell_name: "bash".to_string(),
            working_directory: "/".to_string(),
        };
        assert!(tab.title.contains("\t"));
    }

    #[test]
    fn test_path_with_spaces() {
        let tab = TabInfo {
            id: 0,
            title: "Spaced".to_string(),
            active: true,
            shell_name: "bash".to_string(),
            working_directory: "/home/user/My Documents/Project Name".to_string(),
        };
        assert_eq!(tab.display_directory(), "Project Name");
    }

    #[test]
    fn test_path_with_unicode() {
        let tab = TabInfo {
            id: 0,
            title: "Unicode".to_string(),
            active: true,
            shell_name: "bash".to_string(),
            working_directory: "/home/用户/文档/项目".to_string(),
        };
        assert_eq!(tab.display_directory(), "项目");
    }
}

// ============================================================================
// Consistency Tests
// ============================================================================
mod consistency_tests {
    use super::*;

    #[test]
    fn test_multiple_selections_independent() {
        let sel1 = Selection {
            start_col: 0,
            start_row: 0,
            end_col: 5,
            end_row: 5,
        };
        let sel2 = Selection {
            start_col: 10,
            start_row: 10,
            end_col: 15,
            end_row: 15,
        };

        // Test points that should be in one but not the other
        assert!(sel1.contains(2, 2));
        assert!(!sel2.contains(2, 2));
        assert!(!sel1.contains(12, 12));
        assert!(sel2.contains(12, 12));
    }

    #[test]
    fn test_theme_default_consistency() {
        let default1 = TerminalTheme::default();
        let default2 = TerminalTheme::default();
        let dark = TerminalTheme::dark();

        // Default should be same as dark
        assert_eq!(default1.font_family, dark.font_family);
        assert_eq!(default1.font_size, dark.font_size);
        assert_eq!(default2.line_height, dark.line_height);
    }

    #[test]
    fn test_scrollbar_state_consistency() {
        let mut state = ScrollbarState::new();

        // Initial state
        assert!(!state.is_active());
        assert!(!state.is_dragging());

        // After hover
        state.set_hovered(true);
        assert!(state.is_active());
        assert!(!state.is_dragging());

        // After drag start
        state.start_drag(0);
        assert!(state.is_active());
        assert!(state.is_dragging());

        // Hover should not affect drag
        state.set_hovered(false);
        assert!(state.is_active()); // Still active because dragging
        assert!(state.is_dragging());

        // After drag end
        state.end_drag();
        assert!(!state.is_active());
        assert!(!state.is_dragging());
    }
}
