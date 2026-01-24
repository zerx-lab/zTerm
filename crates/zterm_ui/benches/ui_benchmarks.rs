//! Criterion-based benchmarks for zterm_ui
//!
//! These benchmarks provide statistically accurate performance measurements using Criterion.
//! Following Zed editor's benchmarking approach with:
//! - Bootstrap confidence intervals
//! - Noise detection
//! - Regression detection
//! - HTML reports
//!
//! Run with: cargo bench --package zterm_ui
//! View reports in: target/criterion/report/index.html

use criterion::{black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};

use zterm_ui::{
    GridPosition, ImeState, ScrollbarState, Selection, SharedBounds, TabInfo, TerminalTabBar,
    TerminalTheme, ThumbState,
};

// ============================================================================
// Selection Benchmarks
// ============================================================================

fn bench_selection(c: &mut Criterion) {
    let mut group = c.benchmark_group("selection");

    group.bench_function("new", |b| {
        let mut i = 0usize;
        b.iter(|| {
            i = i.wrapping_add(1);
            black_box(Selection {
                start_col: i % 80,
                start_row: i / 80,
                end_col: (i % 80) + 10,
                end_row: (i / 80) + 5,
            });
        });
    });

    group.bench_function("contains_single_row", |b| {
        let sel = Selection {
            start_col: 10,
            start_row: 5,
            end_col: 70,
            end_row: 5,
        };
        let mut col = 0;
        b.iter(|| {
            col = (col + 1) % 80;
            black_box(sel.contains(col, 5));
        });
    });

    group.bench_function("contains_multi_row", |b| {
        let sel = Selection {
            start_col: 10,
            start_row: 5,
            end_col: 70,
            end_row: 20,
        };
        let mut col = 0;
        let mut row = 5;
        b.iter(|| {
            col = (col + 1) % 80;
            if col == 0 {
                row = 5 + (row - 4) % 16;
            }
            black_box(sel.contains(col, row));
        });
    });

    group.bench_function("contains_large_area", |b| {
        let sel = Selection {
            start_col: 0,
            start_row: 0,
            end_col: 1000,
            end_row: 1000,
        };
        let mut i = 0usize;
        b.iter(|| {
            i = i.wrapping_add(1);
            black_box(sel.contains(i % 1001, i / 1001 % 1001));
        });
    });

    group.bench_function("clone", |b| {
        let sel = Selection {
            start_col: 10,
            start_row: 5,
            end_col: 70,
            end_row: 20,
        };
        b.iter(|| black_box(sel.clone()));
    });

    group.bench_function("comparison", |b| {
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
        b.iter(|| {
            black_box(sel1 == sel2);
            black_box(sel1 != sel2);
        });
    });

    group.finish();
}

// ============================================================================
// GridPosition Benchmarks
// ============================================================================

fn bench_grid_position(c: &mut Criterion) {
    let mut group = c.benchmark_group("grid_position");

    group.bench_function("new", |b| {
        let mut i = 0usize;
        b.iter(|| {
            i = i.wrapping_add(1);
            black_box(GridPosition {
                col: i % 200,
                row: i / 200,
            });
        });
    });

    group.bench_function("clone", |b| {
        let pos = GridPosition { col: 40, row: 12 };
        b.iter(|| black_box(pos.clone()));
    });

    group.bench_function("comparison", |b| {
        let pos1 = GridPosition { col: 40, row: 12 };
        let pos2 = GridPosition { col: 60, row: 20 };
        b.iter(|| black_box(pos1 == pos2));
    });

    group.bench_function("lexicographic_comparison", |b| {
        let pos1 = GridPosition { col: 40, row: 12 };
        let pos2 = GridPosition { col: 60, row: 20 };
        b.iter(|| black_box((pos1.row, pos1.col) < (pos2.row, pos2.col)));
    });

    group.finish();
}

// ============================================================================
// ImeState Benchmarks
// ============================================================================

fn bench_ime_state(c: &mut Criterion) {
    let mut group = c.benchmark_group("ime_state");

    group.bench_function("new_ascii", |b| {
        b.iter(|| {
            black_box(ImeState {
                marked_text: black_box("typing".to_string()),
            });
        });
    });

    group.bench_function("new_unicode", |b| {
        b.iter(|| {
            black_box(ImeState {
                marked_text: black_box("你好世界".to_string()),
            });
        });
    });

    group.bench_function("char_count", |b| {
        let ime = ImeState {
            marked_text: "你好世界Hello こんにちは".to_string(),
        };
        b.iter(|| black_box(ime.marked_text.chars().count()));
    });

    group.bench_function("utf16_encoding", |b| {
        let ime = ImeState {
            marked_text: "你好世界 Hello 🎉".to_string(),
        };
        b.iter(|| black_box(ime.marked_text.encode_utf16().count()));
    });

    group.bench_function("clone", |b| {
        let ime = ImeState {
            marked_text: "你好世界 Hello World".to_string(),
        };
        b.iter(|| black_box(ime.clone()));
    });

    group.finish();
}

// ============================================================================
// TabInfo Benchmarks
// ============================================================================

fn bench_tab_info(c: &mut Criterion) {
    let mut group = c.benchmark_group("tab_info");

    group.bench_function("new", |b| {
        let mut i = 0usize;
        b.iter(|| {
            i = i.wrapping_add(1);
            black_box(TabInfo {
                id: i,
                title: format!("Terminal {}", i),
                active: i == 0,
                shell_name: "bash".to_string(),
                working_directory: format!("/home/user/project{}", i),
            });
        });
    });

    group.bench_function("display_directory", |b| {
        let tab = TabInfo {
            id: 0,
            title: "Terminal".to_string(),
            active: true,
            shell_name: "bash".to_string(),
            working_directory: "/home/user/projects/deep/nested/path/project".to_string(),
        };
        b.iter(|| black_box(tab.display_directory()));
    });

    group.bench_function("display_directory_root", |b| {
        let tab = TabInfo {
            id: 0,
            title: "Terminal".to_string(),
            active: true,
            shell_name: "bash".to_string(),
            working_directory: "/".to_string(),
        };
        b.iter(|| black_box(tab.display_directory()));
    });

    group.bench_function("display_directory_unicode", |b| {
        let tab = TabInfo {
            id: 0,
            title: "Terminal".to_string(),
            active: true,
            shell_name: "bash".to_string(),
            working_directory: "/home/用户/文档/项目".to_string(),
        };
        b.iter(|| black_box(tab.display_directory()));
    });

    group.bench_function("clone", |b| {
        let tab = TabInfo {
            id: 42,
            title: "Test Terminal".to_string(),
            active: true,
            shell_name: "zsh".to_string(),
            working_directory: "/home/user/my-project".to_string(),
        };
        b.iter(|| black_box(tab.clone()));
    });

    group.finish();
}

// ============================================================================
// TerminalTabBar Benchmarks
// ============================================================================

fn bench_terminal_tab_bar(c: &mut Criterion) {
    let mut group = c.benchmark_group("terminal_tab_bar");

    group.bench_function("new", |b| {
        b.iter(|| black_box(TerminalTabBar::new()));
    });

    group.bench_function("default", |b| {
        b.iter(|| black_box(TerminalTabBar::default()));
    });

    for num_tabs in [1, 10, 50, 100] {
        group.bench_with_input(
            BenchmarkId::new("with_tabs", num_tabs),
            &num_tabs,
            |b, &num_tabs| {
                let tabs: Vec<TabInfo> = (0..num_tabs)
                    .map(|i| TabInfo {
                        id: i,
                        title: format!("Terminal {}", i),
                        active: i == 0,
                        shell_name: "bash".to_string(),
                        working_directory: format!("/home/user/project{}", i),
                    })
                    .collect();
                b.iter(|| black_box(TerminalTabBar::new().tabs(tabs.clone())));
            },
        );
    }

    group.finish();
}

// ============================================================================
// TerminalTheme Benchmarks
// ============================================================================

fn bench_terminal_theme(c: &mut Criterion) {
    let mut group = c.benchmark_group("terminal_theme");

    group.bench_function("dark", |b| {
        b.iter(|| black_box(TerminalTheme::dark()));
    });

    group.bench_function("light", |b| {
        b.iter(|| black_box(TerminalTheme::light()));
    });

    group.bench_function("dracula", |b| {
        b.iter(|| black_box(TerminalTheme::dracula()));
    });

    group.bench_function("one_dark", |b| {
        b.iter(|| black_box(TerminalTheme::one_dark()));
    });

    group.bench_function("nord", |b| {
        b.iter(|| black_box(TerminalTheme::nord()));
    });

    group.bench_function("default", |b| {
        b.iter(|| black_box(TerminalTheme::default()));
    });

    group.bench_function("color_access", |b| {
        let theme = TerminalTheme::dark();
        let mut i = 0usize;
        b.iter(|| {
            i = (i + 1) % 16;
            black_box(theme.ansi_colors[i]);
        });
    });

    group.bench_function("clone", |b| {
        let theme = TerminalTheme::dark();
        b.iter(|| black_box(theme.clone()));
    });

    group.finish();
}

// ============================================================================
// ScrollbarState Benchmarks
// ============================================================================

fn bench_scrollbar_state(c: &mut Criterion) {
    let mut group = c.benchmark_group("scrollbar_state");

    group.bench_function("new", |b| {
        b.iter(|| black_box(ScrollbarState::new()));
    });

    group.bench_function("start_drag", |b| {
        let mut state = ScrollbarState::new();
        let mut i = 0i32;
        b.iter(|| {
            i = i.wrapping_add(1);
            state.start_drag(black_box(i));
        });
    });

    group.bench_function("end_drag", |b| {
        let mut state = ScrollbarState::new();
        state.start_drag(10);
        b.iter(|| {
            state.end_drag();
            state.start_drag(10); // Reset for next iteration
        });
    });

    group.bench_function("set_hovered", |b| {
        let mut state = ScrollbarState::new();
        let mut hovered = false;
        b.iter(|| {
            hovered = !hovered;
            state.set_hovered(black_box(hovered));
        });
    });

    group.bench_function("is_dragging", |b| {
        let mut state = ScrollbarState::new();
        state.start_drag(10);
        b.iter(|| black_box(state.is_dragging()));
    });

    group.bench_function("is_active", |b| {
        let mut state = ScrollbarState::new();
        state.set_hovered(true);
        b.iter(|| black_box(state.is_active()));
    });

    group.bench_function("drag_cycle", |b| {
        let mut state = ScrollbarState::new();
        let mut i = 0i32;
        b.iter(|| {
            i = i.wrapping_add(1);
            state.start_drag(i);
            black_box(state.is_dragging());
            state.end_drag();
        });
    });

    group.finish();
}

// ============================================================================
// ThumbState Benchmarks
// ============================================================================

fn bench_thumb_state(c: &mut Criterion) {
    let mut group = c.benchmark_group("thumb_state");

    group.bench_function("inactive", |b| {
        b.iter(|| black_box(ThumbState::Inactive));
    });

    group.bench_function("hovered", |b| {
        b.iter(|| black_box(ThumbState::Hovered));
    });

    group.bench_function("dragging", |b| {
        let mut i = 0i32;
        b.iter(|| {
            i = i.wrapping_add(1);
            black_box(ThumbState::Dragging { offset: i });
        });
    });

    group.bench_function("default", |b| {
        b.iter(|| black_box(ThumbState::default()));
    });

    group.bench_function("is_dragging", |b| {
        let states = [
            ThumbState::Inactive,
            ThumbState::Hovered,
            ThumbState::Dragging { offset: 10 },
        ];
        let mut i = 0usize;
        b.iter(|| {
            i = (i + 1) % 3;
            black_box(states[i].is_dragging());
        });
    });

    group.bench_function("comparison", |b| {
        let state1 = ThumbState::Dragging { offset: 10 };
        let state2 = ThumbState::Dragging { offset: 20 };
        b.iter(|| {
            black_box(state1 == state2);
        });
    });

    group.bench_function("clone", |b| {
        let state = ThumbState::Dragging { offset: 42 };
        b.iter(|| black_box(state.clone()));
    });

    group.finish();
}

// ============================================================================
// SharedBounds Benchmarks
// ============================================================================

fn bench_shared_bounds(c: &mut Criterion) {
    let mut group = c.benchmark_group("shared_bounds");

    group.bench_function("default", |b| {
        b.iter(|| black_box(SharedBounds::default()));
    });

    group.bench_function("clone", |b| {
        let bounds = SharedBounds::default();
        b.iter(|| black_box(bounds.clone()));
    });

    group.bench_function("get_bounds", |b| {
        let bounds = SharedBounds::default();
        b.iter(|| black_box(bounds.bounds.get()));
    });

    group.bench_function("get_cell_width", |b| {
        let bounds = SharedBounds::default();
        b.iter(|| black_box(bounds.cell_width.get()));
    });

    group.bench_function("get_line_height", |b| {
        let bounds = SharedBounds::default();
        b.iter(|| black_box(bounds.line_height.get()));
    });

    group.finish();
}

// ============================================================================
// Integration Benchmarks
// ============================================================================

fn bench_integration(c: &mut Criterion) {
    let mut group = c.benchmark_group("integration");
    group.sample_size(50);

    group.bench_function("full_tab_bar_workflow", |b| {
        b.iter(|| {
            // Create tabs
            let tabs: Vec<TabInfo> = (0..10)
                .map(|i| TabInfo {
                    id: i,
                    title: format!("Terminal {}", i),
                    active: i == 0,
                    shell_name: "bash".to_string(),
                    working_directory: format!("/home/user/project{}", i),
                })
                .collect();

            // Create tab bar
            let _tab_bar = TerminalTabBar::new().tabs(tabs.clone());

            // Access display directories
            for tab in &tabs {
                black_box(tab.display_directory());
            }
        });
    });

    group.bench_function("selection_with_positions", |b| {
        let sel = Selection {
            start_col: 10,
            start_row: 5,
            end_col: 70,
            end_row: 20,
        };
        let positions: Vec<GridPosition> = (0..100)
            .map(|i| GridPosition {
                col: i % 80,
                row: i / 80,
            })
            .collect();

        b.iter(|| {
            for pos in &positions {
                black_box(sel.contains(pos.col, pos.row));
            }
        });
    });

    group.bench_function("theme_scrollbar_workflow", |b| {
        b.iter(|| {
            // Create theme
            let theme = TerminalTheme::dark();

            // Access theme colors
            for i in 0..16 {
                black_box(theme.ansi_colors[i]);
            }
            black_box(theme.background);
            black_box(theme.foreground);

            // Update scrollbar state
            let mut state = ScrollbarState::new();
            state.set_hovered(true);
            state.start_drag(10);
            black_box(state.is_dragging());
            state.end_drag();
        });
    });

    group.bench_function("rapid_state_changes", |b| {
        let mut scrollbar = ScrollbarState::new();
        let selections: Vec<Selection> = (0..10)
            .map(|i| Selection {
                start_col: i * 2,
                start_row: i,
                end_col: i * 2 + 50,
                end_row: i + 10,
            })
            .collect();

        b.iter(|| {
            for i in 0..100 {
                // Scrollbar state changes
                scrollbar.set_hovered(i % 2 == 0);
                if i % 5 == 0 {
                    scrollbar.start_drag(i as i32);
                }
                if i % 7 == 0 {
                    scrollbar.end_drag();
                }

                // Selection checks
                let sel = &selections[i % 10];
                black_box(sel.contains(i % 80, i / 80));
            }
        });
    });

    group.finish();
}

// ============================================================================
// Criterion Groups
// ============================================================================

criterion_group!(
    selection_benches,
    bench_selection,
    bench_grid_position,
);

criterion_group!(
    state_benches,
    bench_ime_state,
    bench_scrollbar_state,
    bench_thumb_state,
    bench_shared_bounds,
);

criterion_group!(
    component_benches,
    bench_tab_info,
    bench_terminal_tab_bar,
    bench_terminal_theme,
);

criterion_group!(integration_benches, bench_integration,);

criterion_main!(
    selection_benches,
    state_benches,
    component_benches,
    integration_benches
);
