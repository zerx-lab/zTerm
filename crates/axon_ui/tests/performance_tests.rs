//! Performance tests for axon_ui
//!
//! These tests verify the performance characteristics of UI components.
//! Following Zed editor's testing approach with benchmark-style tests.
//!
//! Run with: cargo test --package axon_ui --test performance_tests -- --nocapture
//!
//! Performance targets (approximate):
//! - Selection operations: < 1ms for 100000 contains checks
//! - Theme operations: < 10ms for 10000 theme creations
//! - TabInfo operations: < 5ms for 10000 operations
//! - Scrollbar state changes: < 1ms for 100000 state changes

use axon_ui::{
    GridPosition, ImeState, ScrollbarState, Selection, SharedBounds, TabInfo, TerminalTabBar,
    TerminalTheme, ThumbState,
};
use std::time::{Duration, Instant};

// ============================================================================
// Performance Test Utilities
// ============================================================================

/// Helper to measure execution time and return duration
fn measure_time<F, T>(f: F) -> (T, Duration)
where
    F: FnOnce() -> T,
{
    let start = Instant::now();
    let result = f();
    (result, start.elapsed())
}

/// Assert that an operation completes within the given time limit
fn assert_performance<F, T>(name: &str, max_duration: Duration, f: F) -> T
where
    F: FnOnce() -> T,
{
    let (result, elapsed) = measure_time(f);
    println!(
        "[PERF] {}: {:?} (limit: {:?}) {}",
        name,
        elapsed,
        max_duration,
        if elapsed <= max_duration { "✓" } else { "✗" }
    );
    assert!(
        elapsed <= max_duration,
        "{} took {:?}, expected < {:?}",
        name,
        elapsed,
        max_duration
    );
    result
}

/// Run a function multiple times and return average duration
fn benchmark<F>(name: &str, iterations: u32, mut f: F) -> Duration
where
    F: FnMut(),
{
    // Warmup
    for _ in 0..3 {
        f();
    }

    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    let total = start.elapsed();
    let avg = total / iterations;
    println!(
        "[BENCH] {}: {:?} avg over {} iterations (total: {:?})",
        name, avg, iterations, total
    );
    avg
}

// ============================================================================
// Selection Performance Tests
// ============================================================================
mod selection_performance_tests {
    use super::*;

    #[test]
    fn test_selection_creation_performance() {
        assert_performance(
            "Create Selection 100000 times",
            Duration::from_millis(10),
            || {
                for i in 0..100000 {
                    let _ = Selection {
                        start_col: i % 80,
                        start_row: i / 80,
                        end_col: (i % 80) + 10,
                        end_row: (i / 80) + 5,
                    };
                }
            },
        );
    }

    #[test]
    fn test_selection_contains_single_row() {
        let sel = Selection {
            start_col: 10,
            start_row: 5,
            end_col: 70,
            end_row: 5,
        };

        assert_performance(
            "Selection.contains (single row) 100000 times",
            Duration::from_millis(10),
            || {
                for i in 0..100000 {
                    let _ = sel.contains(i % 80, 5);
                }
            },
        );
    }

    #[test]
    fn test_selection_contains_multi_row() {
        let sel = Selection {
            start_col: 10,
            start_row: 5,
            end_col: 70,
            end_row: 20,
        };

        assert_performance(
            "Selection.contains (multi row) 100000 times",
            Duration::from_millis(10),
            || {
                for i in 0..100000 {
                    let col = i % 80;
                    let row = 5 + (i / 80) % 15;
                    let _ = sel.contains(col, row);
                }
            },
        );
    }

    #[test]
    fn test_selection_contains_large_selection() {
        let sel = Selection {
            start_col: 0,
            start_row: 0,
            end_col: 1000,
            end_row: 1000,
        };

        assert_performance(
            "Selection.contains (large area) 100000 times",
            Duration::from_millis(10),
            || {
                for i in 0..100000 {
                    let _ = sel.contains(i % 1001, i / 1001);
                }
            },
        );
    }

    #[test]
    fn test_selection_contains_benchmark() {
        let sel = Selection {
            start_col: 5,
            start_row: 2,
            end_col: 75,
            end_row: 22,
        };

        let avg = benchmark("Selection.contains", 100000, || {
            let _ = sel.contains(40, 12);
        });

        assert!(avg < Duration::from_nanos(1000), "Selection.contains too slow: {:?}", avg);
    }

    #[test]
    fn test_selection_clone_performance() {
        let sel = Selection {
            start_col: 10,
            start_row: 5,
            end_col: 70,
            end_row: 20,
        };

        assert_performance(
            "Clone Selection 100000 times",
            Duration::from_millis(5),
            || {
                for _ in 0..100000 {
                    let _ = sel.clone();
                }
            },
        );
    }

    #[test]
    fn test_selection_comparison_performance() {
        let sel1 = Selection {
            start_col: 10,
            start_row: 5,
            end_col: 70,
            end_row: 20,
        };
        let sel2 = Selection {
            start_col: 20,
            start_row: 10,
            end_col: 60,
            end_row: 15,
        };

        assert_performance(
            "Compare Selections 100000 times",
            Duration::from_millis(5),
            || {
                for _ in 0..100000 {
                    let _ = sel1 == sel2;
                    let _ = sel1 != sel2;
                }
            },
        );
    }

    #[test]
    fn test_selection_boundary_detection_performance() {
        // Test performance at selection boundaries
        let sel = Selection {
            start_col: 50,
            start_row: 50,
            end_col: 100,
            end_row: 100,
        };

        assert_performance(
            "Selection boundary checks 100000 times",
            Duration::from_millis(15),
            || {
                for i in 0..100000 {
                    // Check around boundaries
                    let _ = sel.contains(49, 50 + i % 50); // Just outside start col
                    let _ = sel.contains(50, 50 + i % 50); // At start col
                    let _ = sel.contains(100, 50 + i % 50); // At end col
                    let _ = sel.contains(101, 50 + i % 50); // Just outside end col
                }
            },
        );
    }
}

// ============================================================================
// GridPosition Performance Tests
// ============================================================================
mod grid_position_performance_tests {
    use super::*;

    #[test]
    fn test_grid_position_creation_performance() {
        assert_performance(
            "Create GridPosition 100000 times",
            Duration::from_millis(5),
            || {
                for i in 0..100000 {
                    let _ = GridPosition {
                        col: i % 200,
                        row: i / 200,
                    };
                }
            },
        );
    }

    #[test]
    fn test_grid_position_clone_performance() {
        let pos = GridPosition { col: 40, row: 12 };

        assert_performance(
            "Clone GridPosition 100000 times",
            Duration::from_millis(3),
            || {
                for _ in 0..100000 {
                    let _ = pos.clone();
                }
            },
        );
    }

    #[test]
    fn test_grid_position_comparison_performance() {
        let pos1 = GridPosition { col: 40, row: 12 };
        let pos2 = GridPosition { col: 60, row: 20 };

        assert_performance(
            "Compare GridPosition 100000 times",
            Duration::from_millis(3),
            || {
                for _ in 0..100000 {
                    let _ = pos1 == pos2;
                }
            },
        );
    }

    #[test]
    fn test_grid_position_collection_performance() {
        assert_performance(
            "Create Vec of 10000 GridPositions",
            Duration::from_millis(5),
            || {
                let positions: Vec<GridPosition> = (0..10000)
                    .map(|i| GridPosition {
                        col: i % 200,
                        row: i / 200,
                    })
                    .collect();
                positions
            },
        );
    }

    #[test]
    fn test_grid_position_tuple_comparison_performance() {
        let positions: Vec<GridPosition> = (0..1000)
            .map(|i| GridPosition {
                col: i % 100,
                row: i / 100,
            })
            .collect();

        assert_performance(
            "Lexicographic comparison 100000 times",
            Duration::from_millis(20),
            || {
                for i in 0..100000 {
                    let p1 = &positions[i % 1000];
                    let p2 = &positions[(i + 1) % 1000];
                    let _ = (p1.row, p1.col) < (p2.row, p2.col);
                }
            },
        );
    }
}

// ============================================================================
// ImeState Performance Tests
// ============================================================================
mod ime_state_performance_tests {
    use super::*;

    #[test]
    fn test_ime_state_creation_ascii() {
        assert_performance(
            "Create ImeState (ASCII) 100000 times",
            Duration::from_millis(50),
            || {
                for i in 0..100000 {
                    let _ = ImeState {
                        marked_text: format!("typing{}", i),
                    };
                }
            },
        );
    }

    #[test]
    fn test_ime_state_creation_unicode() {
        assert_performance(
            "Create ImeState (Unicode) 100000 times",
            Duration::from_millis(100),
            || {
                for i in 0..100000 {
                    let _ = ImeState {
                        marked_text: format!("你好{}世界", i),
                    };
                }
            },
        );
    }

    #[test]
    fn test_ime_state_long_text() {
        let long_text = "你好世界".repeat(100);

        assert_performance(
            "Create ImeState (long text) 10000 times",
            Duration::from_millis(100),
            || {
                for _ in 0..10000 {
                    let _ = ImeState {
                        marked_text: long_text.clone(),
                    };
                }
            },
        );
    }

    #[test]
    fn test_ime_state_char_count_performance() {
        let ime = ImeState {
            marked_text: "你好世界Hello こんにちは".to_string(),
        };

        assert_performance(
            "ImeState char count 100000 times",
            Duration::from_millis(50),
            || {
                for _ in 0..100000 {
                    let _ = ime.marked_text.chars().count();
                }
            },
        );
    }

    #[test]
    fn test_ime_state_utf16_encoding_performance() {
        let ime = ImeState {
            marked_text: "你好世界 Hello 🎉".to_string(),
        };

        assert_performance(
            "ImeState UTF-16 encoding 100000 times",
            Duration::from_millis(100),
            || {
                for _ in 0..100000 {
                    let _ = ime.marked_text.encode_utf16().count();
                }
            },
        );
    }

    #[test]
    fn test_ime_state_clone_performance() {
        let ime = ImeState {
            marked_text: "你好世界 Hello World".to_string(),
        };

        assert_performance(
            "Clone ImeState 100000 times",
            Duration::from_millis(50),
            || {
                for _ in 0..100000 {
                    let _ = ime.clone();
                }
            },
        );
    }
}

// ============================================================================
// TabInfo Performance Tests
// ============================================================================
mod tab_info_performance_tests {
    use super::*;

    #[test]
    fn test_tab_info_creation_performance() {
        assert_performance(
            "Create TabInfo 10000 times",
            Duration::from_millis(50),
            || {
                for i in 0..10000 {
                    let _ = TabInfo {
                        id: i,
                        title: format!("Terminal {}", i),
                        active: i == 0,
                        shell_name: "bash".to_string(),
                        working_directory: format!("/home/user/project{}", i),
                    };
                }
            },
        );
    }

    #[test]
    fn test_tab_info_display_directory_performance() {
        let tabs: Vec<TabInfo> = (0..1000)
            .map(|i| TabInfo {
                id: i,
                title: format!("Terminal {}", i),
                active: i == 0,
                shell_name: "bash".to_string(),
                working_directory: format!("/home/user/projects/deep/nested/path/project{}", i),
            })
            .collect();

        assert_performance(
            "TabInfo.display_directory 100000 times",
            Duration::from_millis(100),
            || {
                for i in 0..100000 {
                    let _ = tabs[i % 1000].display_directory();
                }
            },
        );
    }

    #[test]
    fn test_tab_info_display_directory_edge_cases() {
        let edge_case_tabs = vec![
            TabInfo {
                id: 0,
                title: "Root".to_string(),
                active: true,
                shell_name: "bash".to_string(),
                working_directory: "/".to_string(),
            },
            TabInfo {
                id: 1,
                title: "Home".to_string(),
                active: false,
                shell_name: "bash".to_string(),
                working_directory: "/home/user".to_string(),
            },
            TabInfo {
                id: 2,
                title: "Trailing".to_string(),
                active: false,
                shell_name: "bash".to_string(),
                working_directory: "/home/user/project/".to_string(),
            },
            TabInfo {
                id: 3,
                title: "Unicode".to_string(),
                active: false,
                shell_name: "bash".to_string(),
                working_directory: "/home/用户/文档/项目".to_string(),
            },
        ];

        assert_performance(
            "TabInfo.display_directory (edge cases) 100000 times",
            Duration::from_millis(100),
            || {
                for i in 0..100000 {
                    let _ = edge_case_tabs[i % 4].display_directory();
                }
            },
        );
    }

    #[test]
    fn test_tab_info_clone_performance() {
        let tab = TabInfo {
            id: 42,
            title: "Test Terminal".to_string(),
            active: true,
            shell_name: "zsh".to_string(),
            working_directory: "/home/user/my-project".to_string(),
        };

        assert_performance(
            "Clone TabInfo 100000 times",
            Duration::from_millis(100),
            || {
                for _ in 0..100000 {
                    let _ = tab.clone();
                }
            },
        );
    }

    #[test]
    fn test_tab_info_collection_performance() {
        assert_performance(
            "Create Vec of 1000 TabInfos",
            Duration::from_millis(20),
            || {
                let tabs: Vec<TabInfo> = (0..1000)
                    .map(|i| TabInfo {
                        id: i,
                        title: format!("Terminal {}", i),
                        active: i == 0,
                        shell_name: "bash".to_string(),
                        working_directory: format!("/home/user/project{}", i),
                    })
                    .collect();
                tabs
            },
        );
    }

    #[test]
    fn test_tab_info_benchmark() {
        let tab = TabInfo {
            id: 0,
            title: "Terminal".to_string(),
            active: true,
            shell_name: "bash".to_string(),
            working_directory: "/home/user/project".to_string(),
        };

        let avg = benchmark("TabInfo.display_directory", 100000, || {
            let _ = tab.display_directory();
        });

        assert!(avg < Duration::from_micros(10), "display_directory too slow: {:?}", avg);
    }
}

// ============================================================================
// TerminalTabBar Performance Tests
// ============================================================================
mod terminal_tab_bar_performance_tests {
    use super::*;

    #[test]
    fn test_tab_bar_creation_performance() {
        assert_performance(
            "Create TerminalTabBar 10000 times",
            Duration::from_millis(10),
            || {
                for _ in 0..10000 {
                    let _ = TerminalTabBar::new();
                }
            },
        );
    }

    #[test]
    fn test_tab_bar_with_tabs_performance() {
        let tabs: Vec<TabInfo> = (0..10)
            .map(|i| TabInfo {
                id: i,
                title: format!("Terminal {}", i),
                active: i == 0,
                shell_name: "bash".to_string(),
                working_directory: format!("/home/user/project{}", i),
            })
            .collect();

        assert_performance(
            "Create TerminalTabBar with 10 tabs 10000 times",
            Duration::from_millis(100),
            || {
                for _ in 0..10000 {
                    let _ = TerminalTabBar::new().tabs(tabs.clone());
                }
            },
        );
    }

    #[test]
    fn test_tab_bar_with_many_tabs_performance() {
        let tabs: Vec<TabInfo> = (0..100)
            .map(|i| TabInfo {
                id: i,
                title: format!("Terminal {}", i),
                active: i == 0,
                shell_name: "bash".to_string(),
                working_directory: format!("/home/user/project{}", i),
            })
            .collect();

        assert_performance(
            "Create TerminalTabBar with 100 tabs 1000 times",
            Duration::from_millis(200),
            || {
                for _ in 0..1000 {
                    let _ = TerminalTabBar::new().tabs(tabs.clone());
                }
            },
        );
    }

    #[test]
    fn test_tab_bar_default_performance() {
        assert_performance(
            "TerminalTabBar::default 10000 times",
            Duration::from_millis(10),
            || {
                for _ in 0..10000 {
                    let _ = TerminalTabBar::default();
                }
            },
        );
    }
}

// ============================================================================
// TerminalTheme Performance Tests
// ============================================================================
mod terminal_theme_performance_tests {
    use super::*;

    #[test]
    fn test_theme_creation_dark() {
        assert_performance(
            "Create dark theme 10000 times",
            Duration::from_millis(50),
            || {
                for _ in 0..10000 {
                    let _ = TerminalTheme::dark();
                }
            },
        );
    }

    #[test]
    fn test_theme_creation_light() {
        assert_performance(
            "Create light theme 10000 times",
            Duration::from_millis(50),
            || {
                for _ in 0..10000 {
                    let _ = TerminalTheme::light();
                }
            },
        );
    }

    #[test]
    fn test_theme_creation_all_themes() {
        assert_performance(
            "Create all themes 2000 times each",
            Duration::from_millis(100),
            || {
                for _ in 0..2000 {
                    let _ = TerminalTheme::dark();
                    let _ = TerminalTheme::light();
                    let _ = TerminalTheme::dracula();
                    let _ = TerminalTheme::one_dark();
                    let _ = TerminalTheme::nord();
                }
            },
        );
    }

    #[test]
    fn test_theme_default_performance() {
        assert_performance(
            "TerminalTheme::default 10000 times",
            Duration::from_millis(50),
            || {
                for _ in 0..10000 {
                    let _ = TerminalTheme::default();
                }
            },
        );
    }

    #[test]
    fn test_theme_clone_performance() {
        let theme = TerminalTheme::dark();

        assert_performance(
            "Clone TerminalTheme 10000 times",
            Duration::from_millis(50),
            || {
                for _ in 0..10000 {
                    let _ = theme.clone();
                }
            },
        );
    }

    #[test]
    fn test_theme_color_access_performance() {
        let theme = TerminalTheme::dark();

        assert_performance(
            "Access theme colors 100000 times",
            Duration::from_millis(10),
            || {
                for i in 0..100000 {
                    let _ = theme.ansi_colors[i % 16];
                }
            },
        );
    }

    #[test]
    fn test_theme_benchmark() {
        let avg = benchmark("TerminalTheme::dark()", 10000, || {
            let _ = TerminalTheme::dark();
        });

        assert!(avg < Duration::from_micros(100), "Theme creation too slow: {:?}", avg);
    }
}

// ============================================================================
// ScrollbarState Performance Tests
// ============================================================================
mod scrollbar_state_performance_tests {
    use super::*;

    #[test]
    fn test_scrollbar_state_creation_performance() {
        assert_performance(
            "Create ScrollbarState 100000 times",
            Duration::from_millis(20),
            || {
                for _ in 0..100000 {
                    let _ = ScrollbarState::new();
                }
            },
        );
    }

    #[test]
    fn test_scrollbar_state_transitions_performance() {
        let mut state = ScrollbarState::new();

        assert_performance(
            "Scrollbar state transitions 100000 times",
            Duration::from_millis(20),
            || {
                for i in 0..100000 {
                    match i % 4 {
                        0 => state.set_hovered(true),
                        1 => state.start_drag(i as i32),
                        2 => state.end_drag(),
                        3 => state.set_hovered(false),
                        _ => {}
                    }
                }
            },
        );
    }

    #[test]
    fn test_scrollbar_state_queries_performance() {
        let mut state = ScrollbarState::new();
        state.start_drag(10);

        assert_performance(
            "Scrollbar state queries 100000 times",
            Duration::from_millis(10),
            || {
                for _ in 0..100000 {
                    let _ = state.is_dragging();
                    let _ = state.is_active();
                }
            },
        );
    }

    #[test]
    fn test_scrollbar_state_drag_cycle_performance() {
        let mut state = ScrollbarState::new();

        assert_performance(
            "Scrollbar drag cycle 50000 times",
            Duration::from_millis(20),
            || {
                for i in 0..50000 {
                    state.start_drag(i as i32);
                    let _ = state.is_dragging();
                    state.end_drag();
                }
            },
        );
    }

    #[test]
    fn test_scrollbar_state_benchmark() {
        let mut state = ScrollbarState::new();

        let avg = benchmark("ScrollbarState start_drag/end_drag", 100000, || {
            state.start_drag(10);
            state.end_drag();
        });

        assert!(avg < Duration::from_nanos(500), "State transitions too slow: {:?}", avg);
    }
}

// ============================================================================
// ThumbState Performance Tests
// ============================================================================
mod thumb_state_performance_tests {
    use super::*;

    #[test]
    fn test_thumb_state_creation_performance() {
        assert_performance(
            "Create ThumbState variants 100000 times",
            Duration::from_millis(10),
            || {
                for i in 0..100000 {
                    let _ = match i % 3 {
                        0 => ThumbState::Inactive,
                        1 => ThumbState::Hovered,
                        _ => ThumbState::Dragging { offset: i as i32 },
                    };
                }
            },
        );
    }

    #[test]
    fn test_thumb_state_is_dragging_performance() {
        let states = [
            ThumbState::Inactive,
            ThumbState::Hovered,
            ThumbState::Dragging { offset: 10 },
        ];

        assert_performance(
            "ThumbState.is_dragging 100000 times",
            Duration::from_millis(5),
            || {
                for i in 0..100000 {
                    let _ = states[i % 3].is_dragging();
                }
            },
        );
    }

    #[test]
    fn test_thumb_state_comparison_performance() {
        let state1 = ThumbState::Dragging { offset: 10 };
        let state2 = ThumbState::Dragging { offset: 20 };
        let state3 = ThumbState::Hovered;

        assert_performance(
            "ThumbState comparison 100000 times",
            Duration::from_millis(5),
            || {
                for _ in 0..100000 {
                    let _ = state1 == state2;
                    let _ = state1 == state3;
                    let _ = state2 != state3;
                }
            },
        );
    }

    #[test]
    fn test_thumb_state_clone_performance() {
        let state = ThumbState::Dragging { offset: 42 };

        assert_performance(
            "Clone ThumbState 100000 times",
            Duration::from_millis(5),
            || {
                for _ in 0..100000 {
                    let _ = state.clone();
                }
            },
        );
    }

    #[test]
    fn test_thumb_state_default_performance() {
        assert_performance(
            "ThumbState::default 100000 times",
            Duration::from_millis(5),
            || {
                for _ in 0..100000 {
                    let _ = ThumbState::default();
                }
            },
        );
    }
}

// ============================================================================
// SharedBounds Performance Tests
// ============================================================================
mod shared_bounds_performance_tests {
    use super::*;

    #[test]
    fn test_shared_bounds_creation_performance() {
        assert_performance(
            "Create SharedBounds 10000 times",
            Duration::from_millis(50),
            || {
                for _ in 0..10000 {
                    let _ = SharedBounds::default();
                }
            },
        );
    }

    #[test]
    fn test_shared_bounds_clone_performance() {
        let bounds = SharedBounds::default();

        assert_performance(
            "Clone SharedBounds 10000 times",
            Duration::from_millis(20),
            || {
                for _ in 0..10000 {
                    let _ = bounds.clone();
                }
            },
        );
    }

    #[test]
    fn test_shared_bounds_access_performance() {
        let bounds = SharedBounds::default();

        assert_performance(
            "Access SharedBounds values 100000 times",
            Duration::from_millis(20),
            || {
                for _ in 0..100000 {
                    let _ = bounds.bounds.get();
                    let _ = bounds.cell_width.get();
                    let _ = bounds.line_height.get();
                }
            },
        );
    }
}

// ============================================================================
// Combined/Integration Performance Tests
// ============================================================================
mod integration_performance_tests {
    use super::*;

    #[test]
    fn test_full_tab_bar_workflow() {
        assert_performance(
            "Full tab bar workflow 1000 iterations",
            Duration::from_millis(200),
            || {
                for i in 0..1000 {
                    // Create tabs
                    let tabs: Vec<TabInfo> = (0..10)
                        .map(|j| TabInfo {
                            id: j + i * 10,
                            title: format!("Terminal {}", j),
                            active: j == i % 10,
                            shell_name: "bash".to_string(),
                            working_directory: format!("/home/user/project{}", j),
                        })
                        .collect();

                    // Create tab bar
                    let _tab_bar = TerminalTabBar::new().tabs(tabs.clone());

                    // Access display directories
                    for tab in &tabs {
                        let _ = tab.display_directory();
                    }
                }
            },
        );
    }

    #[test]
    fn test_selection_with_grid_positions() {
        let sel = Selection {
            start_col: 10,
            start_row: 5,
            end_col: 70,
            end_row: 20,
        };

        let positions: Vec<GridPosition> = (0..1000)
            .map(|i| GridPosition {
                col: i % 80,
                row: i / 80,
            })
            .collect();

        assert_performance(
            "Check selection for 1000 positions 100 times",
            Duration::from_millis(20),
            || {
                for _ in 0..100 {
                    for pos in &positions {
                        let _ = sel.contains(pos.col, pos.row);
                    }
                }
            },
        );
    }

    #[test]
    fn test_theme_with_scrollbar_state() {
        assert_performance(
            "Theme + scrollbar workflow 10000 times",
            Duration::from_millis(100),
            || {
                for i in 0..10000 {
                    // Create theme
                    let theme = match i % 5 {
                        0 => TerminalTheme::dark(),
                        1 => TerminalTheme::light(),
                        2 => TerminalTheme::dracula(),
                        3 => TerminalTheme::one_dark(),
                        _ => TerminalTheme::nord(),
                    };

                    // Access theme colors
                    let _ = theme.ansi_colors[i % 16];
                    let _ = theme.background;
                    let _ = theme.foreground;

                    // Update scrollbar state
                    let mut state = ScrollbarState::new();
                    state.set_hovered(i % 2 == 0);
                    if i % 3 == 0 {
                        state.start_drag(i as i32);
                        let _ = state.is_dragging();
                        state.end_drag();
                    }
                }
            },
        );
    }

    #[test]
    fn test_rapid_state_changes() {
        let mut scrollbar = ScrollbarState::new();
        let selections: Vec<Selection> = (0..100)
            .map(|i| Selection {
                start_col: i * 2,
                start_row: i,
                end_col: i * 2 + 50,
                end_row: i + 10,
            })
            .collect();

        assert_performance(
            "Rapid state changes 10000 cycles",
            Duration::from_millis(50),
            || {
                for i in 0..10000 {
                    // Scrollbar state changes
                    scrollbar.set_hovered(i % 2 == 0);
                    if i % 5 == 0 {
                        scrollbar.start_drag(i as i32);
                    }
                    if i % 7 == 0 {
                        scrollbar.end_drag();
                    }

                    // Selection checks
                    let sel = &selections[i % 100];
                    let _ = sel.contains(i % 80, i / 80);
                }
            },
        );
    }
}

// ============================================================================
// Memory Pattern Tests
// ============================================================================
mod memory_pattern_tests {
    use super::*;

    #[test]
    fn test_tab_info_vector_growth() {
        assert_performance(
            "TabInfo vector growth to 10000",
            Duration::from_millis(100),
            || {
                let mut tabs = Vec::new();
                for i in 0..10000 {
                    tabs.push(TabInfo {
                        id: i,
                        title: format!("Terminal {}", i),
                        active: i == 0,
                        shell_name: "bash".to_string(),
                        working_directory: format!("/home/user/project{}", i),
                    });
                }
            },
        );
    }

    #[test]
    fn test_selection_vector_operations() {
        let mut selections: Vec<Selection> = Vec::new();

        assert_performance(
            "Selection vector operations",
            Duration::from_millis(50),
            || {
                // Add selections
                for i in 0..1000 {
                    selections.push(Selection {
                        start_col: i,
                        start_row: i,
                        end_col: i + 50,
                        end_row: i + 10,
                    });
                }

                // Query all selections
                for sel in &selections {
                    let _ = sel.contains(25, 5);
                }

                // Clear and rebuild
                selections.clear();
                for i in 0..1000 {
                    selections.push(Selection {
                        start_col: i * 2,
                        start_row: i,
                        end_col: i * 2 + 30,
                        end_row: i + 5,
                    });
                }
            },
        );
    }

    #[test]
    fn test_theme_caching_pattern() {
        // Simulate caching pattern - create once, access many times
        let theme = TerminalTheme::dark();

        assert_performance(
            "Theme cached access pattern 1000000 times",
            Duration::from_millis(50),
            || {
                for i in 0..1000000 {
                    let _ = theme.ansi_colors[i % 16];
                }
            },
        );
    }
}

// ============================================================================
// Throughput Summary Test
// ============================================================================
#[test]
fn test_throughput_summary() {
    println!("\n========================================");
    println!("UI Performance Test Summary");
    println!("========================================\n");

    // Selection throughput
    let sel = Selection {
        start_col: 10,
        start_row: 5,
        end_col: 70,
        end_row: 20,
    };
    let start = Instant::now();
    for i in 0..1000000 {
        let _ = sel.contains(i % 80, (i / 80) % 24);
    }
    let selection_rate = 1000000.0 / start.elapsed().as_secs_f64();
    println!("Selection.contains throughput: {:.0} checks/sec", selection_rate);

    // Theme creation throughput
    let start = Instant::now();
    for _ in 0..100000 {
        let _ = TerminalTheme::dark();
    }
    let theme_rate = 100000.0 / start.elapsed().as_secs_f64();
    println!("Theme creation throughput: {:.0} themes/sec", theme_rate);

    // TabInfo.display_directory throughput
    let tab = TabInfo {
        id: 0,
        title: "Terminal".to_string(),
        active: true,
        shell_name: "bash".to_string(),
        working_directory: "/home/user/projects/my-project".to_string(),
    };
    let start = Instant::now();
    for _ in 0..1000000 {
        let _ = tab.display_directory();
    }
    let display_rate = 1000000.0 / start.elapsed().as_secs_f64();
    println!("TabInfo.display_directory throughput: {:.0} calls/sec", display_rate);

    // Scrollbar state transitions
    let mut state = ScrollbarState::new();
    let start = Instant::now();
    for i in 0..1000000 {
        state.start_drag(i);
        state.end_drag();
    }
    let state_rate = 1000000.0 / start.elapsed().as_secs_f64();
    println!("Scrollbar state transitions: {:.0} transitions/sec", state_rate);

    println!("\n========================================\n");
}
