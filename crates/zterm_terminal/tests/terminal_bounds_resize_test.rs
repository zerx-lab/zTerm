//! Terminal Bounds Resize Bug Reproduction Tests
//!
//! This test file reproduces the bug scenario more closely to how zterm uses alacritty_terminal.
//!
//! The key difference from terminal_view_resize_bug_test.rs is that this test:
//! 1. Uses TerminalBounds with pixel-based dimensions (like the actual UI)
//! 2. Tests the interaction between terminal resize and display_offset
//! 3. Simulates the prepaint/paint cycle where bounds are updated
//!
//! Bug Description:
//! - When window is maximized, the terminal bounds change
//! - set_bounds() is called with new dimensions
//! - Content outside the new view area disappears
//! - After restoring, history content appears garbled

#![allow(clippy::assertions_on_constants)]
#![allow(unused_variables)]
#![allow(unused_assignments)]

use alacritty_terminal::event::{Event as AlacEvent, EventListener};
use alacritty_terminal::grid::{Dimensions, Scroll};
use alacritty_terminal::index::{Column, Direction, Line, Point};
use alacritty_terminal::selection::{Selection, SelectionType};
use alacritty_terminal::term::{Config, Term};
use alacritty_terminal::vte::ansi::{Processor, StdSyncHandler};

/// Dummy event listener for tests
#[derive(Clone)]
struct TestEventListener;

impl EventListener for TestEventListener {
    fn send_event(&self, _event: AlacEvent) {}
}

/// Simulates zterm's TerminalBounds structure
/// This is the key structure used in terminal_element.rs
#[derive(Debug, Clone, Copy)]
struct SimulatedTerminalBounds {
    cell_width: f32,
    line_height: f32,
    width_px: f32,
    height_px: f32,
}

impl SimulatedTerminalBounds {
    fn new(width_px: f32, height_px: f32, cell_width: f32, line_height: f32) -> Self {
        Self {
            cell_width,
            line_height,
            width_px,
            height_px,
        }
    }

    fn num_lines(&self) -> usize {
        (self.height_px / self.line_height).floor() as usize
    }

    fn num_columns(&self) -> usize {
        (self.width_px / self.cell_width).floor() as usize
    }

    /// Simulates a typical small window (800x600)
    fn small_window() -> Self {
        // Cell: ~8px wide, ~16px tall
        Self::new(800.0, 600.0, 8.0, 16.0)
        // = 100 cols, 37 lines
    }

    /// Simulates a maximized window (1920x1080)
    fn maximized() -> Self {
        Self::new(1920.0, 1080.0, 8.0, 16.0)
        // = 240 cols, 67 lines
    }

    /// Simulates restoring to original size
    fn restored() -> Self {
        Self::small_window()
    }
}

impl Dimensions for SimulatedTerminalBounds {
    fn total_lines(&self) -> usize {
        self.num_lines()
    }

    fn screen_lines(&self) -> usize {
        self.num_lines()
    }

    fn columns(&self) -> usize {
        self.num_columns()
    }
}

/// Helper function to write text to terminal
fn write_to_terminal(term: &mut Term<TestEventListener>, text: &str) {
    let mut processor: Processor<StdSyncHandler> = Processor::new();
    processor.advance(term, text.as_bytes());
}

/// Helper function to get all text including scrollback
fn get_all_text_with_scrollback(term: &Term<TestEventListener>) -> String {
    let mut result = String::new();
    let history = term.history_size();

    // Get scrollback content
    for line in (-(history as i32))..0 {
        for col in 0..term.columns() {
            let point = Point::new(Line(line), Column(col));
            let c = term.grid()[point].c;
            if c != '\0' {
                result.push(c);
            }
        }
        result.push('\n');
    }

    // Get visible content
    for line in 0..term.screen_lines() as i32 {
        for col in 0..term.columns() {
            let point = Point::new(Line(line), Column(col));
            let c = term.grid()[point].c;
            if c != '\0' {
                result.push(c);
            }
        }
        result.push('\n');
    }
    result
}

/// Helper function to get visible text
fn get_visible_text(term: &Term<TestEventListener>) -> String {
    let mut result = String::new();
    for line in 0..term.screen_lines() as i32 {
        for col in 0..term.columns() {
            let point = Point::new(Line(line), Column(col));
            let c = term.grid()[point].c;
            if c != '\0' {
                result.push(c);
            }
        }
        result.push('\n');
    }
    result
}

// ============================================================================
// Bug Reproduction Tests Using Simulated Terminal Bounds
// ============================================================================

/// Test Case 1: Simulate the exact bug scenario with pixel-based bounds
///
/// This test more accurately simulates what happens in terminal_element.rs prepaint()
#[test]
fn test_pixel_based_resize_scenario() {
    let initial_bounds = SimulatedTerminalBounds::small_window();

    println!("=== Initial Window ===");
    println!(
        "Dimensions: {}x{} px",
        initial_bounds.width_px, initial_bounds.height_px
    );
    println!(
        "Grid: {} cols x {} lines",
        initial_bounds.num_columns(),
        initial_bounds.num_lines()
    );

    let config = Config {
        scrolling_history: 10000,
        ..Config::default()
    };
    let listener = TestEventListener;
    let mut term = Term::new(config, &initial_bounds, listener);

    // Simulate shell session with lots of output
    write_to_terminal(
        &mut term,
        "user@host:~$ find / -name '*.rs' 2>/dev/null\r\n",
    );

    // Simulate 200 lines of output (well exceeds 37-line view)
    for i in 1..=200 {
        write_to_terminal(
            &mut term,
            &format!(
                "/home/user/project{}/src/module{}/file{:03}.rs\r\n",
                i % 10,
                i % 5,
                i
            ),
        );
    }

    write_to_terminal(&mut term, "user@host:~$ ");

    println!("\n=== After Command Output ===");
    println!("History size: {}", term.history_size());
    println!("Display offset: {}", term.grid().display_offset());

    // Verify first output is in scrollback
    let all_content = get_all_text_with_scrollback(&term);
    assert!(
        all_content.contains("file001.rs"),
        "file001.rs should be in scrollback"
    );

    // STEP 1: User scrolls up to view history
    term.scroll_display(Scroll::Delta(100)); // Scroll up 100 lines
    let scroll_offset_before = term.grid().display_offset();
    println!("\n=== After User Scrolls Up ===");
    println!("Display offset: {}", scroll_offset_before);

    // Get what user is currently viewing
    let visible_before_maximize = get_visible_text(&term);
    println!(
        "First visible line: {}",
        visible_before_maximize.lines().next().unwrap_or("(empty)")
    );

    // STEP 2: User double-clicks titlebar (MAXIMIZE)
    let maximized_bounds = SimulatedTerminalBounds::maximized();
    println!(
        "\n=== MAXIMIZE: {}x{} px ({} cols x {} lines) ===",
        maximized_bounds.width_px,
        maximized_bounds.height_px,
        maximized_bounds.num_columns(),
        maximized_bounds.num_lines()
    );

    term.resize(maximized_bounds);

    let scroll_offset_after_max = term.grid().display_offset();
    println!("History size after maximize: {}", term.history_size());
    println!("Display offset after maximize: {}", scroll_offset_after_max);

    // Get visible content after maximize
    let visible_after_maximize = get_visible_text(&term);

    // STEP 3: User restores window (clicks titlebar again)
    let restored_bounds = SimulatedTerminalBounds::restored();
    println!(
        "\n=== RESTORE: {}x{} px ({} cols x {} lines) ===",
        restored_bounds.width_px,
        restored_bounds.height_px,
        restored_bounds.num_columns(),
        restored_bounds.num_lines()
    );

    term.resize(restored_bounds);

    let scroll_offset_after_restore = term.grid().display_offset();
    println!("History size after restore: {}", term.history_size());
    println!(
        "Display offset after restore: {}",
        scroll_offset_after_restore
    );

    // BUG CHECK: Verify content is preserved
    let all_content_after = get_all_text_with_scrollback(&term);

    let has_file001 = all_content_after.contains("file001.rs");
    let has_file100 = all_content_after.contains("file100.rs");
    let has_file200 = all_content_after.contains("file200.rs");
    let has_prompt = all_content_after.contains("user@host:~$");

    println!("\n=== Content Verification ===");
    println!("file001.rs present: {}", has_file001);
    println!("file100.rs present: {}", has_file100);
    println!("file200.rs present: {}", has_file200);
    println!("Prompt present: {}", has_prompt);

    assert!(
        has_file001,
        "BUG: file001.rs was lost during maximize/restore"
    );
    assert!(
        has_file100,
        "BUG: file100.rs was lost during maximize/restore"
    );
    assert!(
        has_file200,
        "BUG: file200.rs was lost during maximize/restore"
    );
    assert!(
        has_prompt,
        "BUG: Shell prompt was lost during maximize/restore"
    );

    // BUG CHECK: User should be able to scroll to view old content
    term.scroll_display(Scroll::Delta(100)); // Try to scroll up
    let scroll_after_second_scroll = term.grid().display_offset();
    println!("\n=== After Second Scroll Attempt ===");
    println!("Display offset: {}", scroll_after_second_scroll);

    // Should be able to scroll
    // If scroll_after_second_scroll is 0, user can't scroll (BUG!)
    if term.history_size() > 0 {
        assert!(
            scroll_after_second_scroll > 0,
            "BUG: Cannot scroll up after restore (display_offset=0 but history_size={})",
            term.history_size()
        );
    }
}

/// Test Case 2: Simulate rapid bounds updates during resize animation
///
/// When window is being resized (drag corner), many intermediate sizes are set
#[test]
fn test_rapid_bounds_changes_during_resize_animation() {
    let start_bounds = SimulatedTerminalBounds::small_window();

    let config = Config {
        scrolling_history: 10000,
        ..Config::default()
    };
    let listener = TestEventListener;
    let mut term = Term::new(config, &start_bounds, listener);

    // Fill with content
    for i in 1..=100 {
        write_to_terminal(&mut term, &format!("Animation test line {:03}\r\n", i));
    }

    // Record initial state
    let initial_history = term.history_size();
    println!("Initial history size: {}", initial_history);

    // Simulate resize animation (many intermediate sizes)
    // Width: 800 -> 1920 in 10 steps
    // Height: 600 -> 1080 in 10 steps
    for step in 0..=10 {
        let progress = step as f32 / 10.0;
        let width = 800.0 + (1920.0 - 800.0) * progress;
        let height = 600.0 + (1080.0 - 600.0) * progress;
        let bounds = SimulatedTerminalBounds::new(width, height, 8.0, 16.0);
        term.resize(bounds);
    }

    // Now resize back
    for step in (0..=10).rev() {
        let progress = step as f32 / 10.0;
        let width = 800.0 + (1920.0 - 800.0) * progress;
        let height = 600.0 + (1080.0 - 600.0) * progress;
        let bounds = SimulatedTerminalBounds::new(width, height, 8.0, 16.0);
        term.resize(bounds);
    }

    // Verify content integrity
    let all_content = get_all_text_with_scrollback(&term);
    assert!(
        all_content.contains("Animation test line 001"),
        "BUG: First line lost during resize animation"
    );
    assert!(
        all_content.contains("Animation test line 100"),
        "BUG: Last line lost during resize animation"
    );
}

/// Test Case 3: Test the interaction between scroll_offset and resize
///
/// This simulates what happens in TerminalView when scroll_offset and
/// terminal's display_offset get out of sync
#[test]
fn test_scroll_offset_sync_after_resize() {
    let bounds = SimulatedTerminalBounds::small_window();

    let config = Config {
        scrolling_history: 10000,
        ..Config::default()
    };
    let listener = TestEventListener;
    let mut term = Term::new(config, &bounds, listener);

    // Fill with content
    for i in 1..=100 {
        write_to_terminal(&mut term, &format!("Sync test line {:03}\r\n", i));
    }

    // Simulate TerminalView's scroll_offset tracking
    // In TerminalView, scroll_offset is a separate variable from term.display_offset()
    let mut view_scroll_offset: usize = 0;

    // User scrolls up
    view_scroll_offset = 50;
    term.scroll_display(Scroll::Delta(50));

    println!("View scroll_offset: {}", view_scroll_offset);
    println!("Term display_offset: {}", term.grid().display_offset());

    // They should be in sync
    assert_eq!(
        view_scroll_offset,
        term.grid().display_offset(),
        "Scroll offsets should be in sync before resize"
    );

    // RESIZE happens (maximize)
    let max_bounds = SimulatedTerminalBounds::maximized();
    term.resize(max_bounds);

    let new_display_offset = term.grid().display_offset();
    let new_history_size = term.history_size();

    println!("\nAfter maximize:");
    println!("View scroll_offset: {} (stale)", view_scroll_offset);
    println!("Term display_offset: {}", new_display_offset);
    println!("History size: {}", new_history_size);

    // BUG SCENARIO: View's scroll_offset may now be stale!
    // If view_scroll_offset > new_history_size, there's a problem
    if view_scroll_offset > new_history_size {
        println!(
            "WARNING: View scroll_offset {} > history_size {}",
            view_scroll_offset, new_history_size
        );
    }

    // In the real code, TerminalView needs to clamp scroll_offset
    view_scroll_offset = view_scroll_offset.min(new_history_size);

    // RESIZE back (restore)
    term.resize(bounds);

    let restored_display_offset = term.grid().display_offset();
    let restored_history_size = term.history_size();

    println!("\nAfter restore:");
    println!(
        "View scroll_offset: {} (possibly stale)",
        view_scroll_offset
    );
    println!("Term display_offset: {}", restored_display_offset);
    println!("History size: {}", restored_history_size);

    // Again, scroll_offset may be invalid
    if view_scroll_offset > restored_history_size {
        println!(
            "BUG CONDITION: scroll_offset {} > history_size {} after restore",
            view_scroll_offset, restored_history_size
        );
    }

    // The view should sync its scroll_offset with the terminal
    view_scroll_offset = restored_display_offset;

    // Now try to scroll
    let max_scroll = restored_history_size;
    let scroll_delta = 10;
    let new_scroll = view_scroll_offset
        .saturating_add(scroll_delta)
        .min(max_scroll);
    term.scroll_display(Scroll::Delta(scroll_delta as i32));

    println!("\nAfter manual scroll:");
    println!("New view scroll_offset: {}", new_scroll);
    println!("Term display_offset: {}", term.grid().display_offset());

    // Verify content is still accessible
    let all_content = get_all_text_with_scrollback(&term);
    assert!(
        all_content.contains("Sync test line 001"),
        "BUG: Content lost due to scroll offset desync"
    );
}

/// Test Case 4: Test with fractional pixel dimensions
///
/// In real rendering, dimensions may have fractional values
#[test]
fn test_fractional_pixel_dimensions() {
    // Fractional dimensions that could cause rounding issues
    let bounds = SimulatedTerminalBounds::new(805.5, 603.7, 8.4, 16.2);

    let config = Config {
        scrolling_history: 1000,
        ..Config::default()
    };
    let listener = TestEventListener;
    let mut term = Term::new(config, &bounds, listener);

    // Fill with content
    for i in 1..=50 {
        write_to_terminal(&mut term, &format!("Fractional test {:03}\r\n", i));
    }

    // Different fractional bounds
    let new_bounds = SimulatedTerminalBounds::new(1920.3, 1079.8, 8.4, 16.2);
    term.resize(new_bounds);

    let restored_bounds = SimulatedTerminalBounds::new(806.1, 604.2, 8.4, 16.2);
    term.resize(restored_bounds);

    // Verify content
    let all_content = get_all_text_with_scrollback(&term);
    assert!(
        all_content.contains("Fractional test 001"),
        "BUG: Content lost with fractional pixel dimensions"
    );
}

/// Test Case 5: Test edge case where screen grows larger than total content
///
/// When maximized, the screen might be larger than all existing content
#[test]
fn test_screen_larger_than_content() {
    let small_bounds = SimulatedTerminalBounds::new(400.0, 200.0, 8.0, 16.0); // ~12 lines

    let config = Config {
        scrolling_history: 1000,
        ..Config::default()
    };
    let listener = TestEventListener;
    let mut term = Term::new(config, &small_bounds, listener);

    // Write only 5 lines (less than screen can display)
    for i in 1..=5 {
        write_to_terminal(&mut term, &format!("Short content {:02}\r\n", i));
    }

    println!(
        "Initial: {} history, {} screen lines",
        term.history_size(),
        term.screen_lines()
    );

    // Maximize to very large screen
    let huge_bounds = SimulatedTerminalBounds::new(2560.0, 1440.0, 8.0, 16.0); // ~90 lines
    term.resize(huge_bounds);

    println!(
        "After maximize: {} history, {} screen lines",
        term.history_size(),
        term.screen_lines()
    );

    // All content should be visible, no scrollback needed
    assert_eq!(
        term.history_size(),
        0,
        "With large screen, all content should be visible (no scrollback)"
    );

    // Restore to small
    term.resize(small_bounds);

    println!(
        "After restore: {} history, {} screen lines",
        term.history_size(),
        term.screen_lines()
    );

    // Content should still be there
    let all_content = get_all_text_with_scrollback(&term);
    for i in 1..=5 {
        assert!(
            all_content.contains(&format!("Short content {:02}", i)),
            "BUG: Line {} lost when screen was larger than content",
            i
        );
    }
}

/// Test Case 6: Test resize with active selection
///
/// Selection state might affect resize behavior
#[test]
fn test_resize_with_active_selection() {
    let bounds = SimulatedTerminalBounds::small_window();

    let config = Config {
        scrolling_history: 1000,
        ..Config::default()
    };
    let listener = TestEventListener;
    let mut term = Term::new(config, &bounds, listener);

    // Fill with content
    for i in 1..=50 {
        write_to_terminal(&mut term, &format!("Selection test line {:02}\r\n", i));
    }

    // Start a selection
    let start_point = Point::new(Line(0), Column(0));
    let selection = Selection::new(SelectionType::Simple, start_point, Direction::Left);
    term.selection = Some(selection);

    // Verify selection exists
    assert!(term.selection.is_some(), "Selection should be active");

    // Resize with selection active
    let max_bounds = SimulatedTerminalBounds::maximized();
    term.resize(max_bounds);

    // Note: Selection might be cleared after resize (that's OK)
    // The important thing is content is preserved

    term.resize(bounds);

    // Verify content
    let all_content = get_all_text_with_scrollback(&term);
    assert!(
        all_content.contains("Selection test line 01"),
        "BUG: Content lost during resize with active selection"
    );
}

/// Test Case 7: Test display_offset clamping
///
/// The display_offset must be clamped to valid range after resize
#[test]
fn test_display_offset_clamping() {
    let bounds = SimulatedTerminalBounds::small_window();

    let config = Config {
        scrolling_history: 100, // Small scrollback for this test
        ..Config::default()
    };
    let listener = TestEventListener;
    let mut term = Term::new(config, &bounds, listener);

    // Fill scrollback
    for i in 1..=150 {
        write_to_terminal(&mut term, &format!("Clamp test {:03}\r\n", i));
    }

    // Scroll to max
    term.scroll_display(Scroll::Delta(100));
    let offset_before = term.grid().display_offset();
    let history_before = term.history_size();

    println!(
        "Before resize: offset={}, history={}",
        offset_before, history_before
    );

    // Maximize (will change history size)
    let max_bounds = SimulatedTerminalBounds::maximized();
    term.resize(max_bounds);

    let offset_after_max = term.grid().display_offset();
    let history_after_max = term.history_size();

    println!(
        "After maximize: offset={}, history={}",
        offset_after_max, history_after_max
    );

    // Display offset should be clamped to new history size
    assert!(
        offset_after_max <= history_after_max,
        "BUG: Display offset {} exceeds history size {} after maximize",
        offset_after_max,
        history_after_max
    );

    // Restore
    term.resize(bounds);

    let offset_after_restore = term.grid().display_offset();
    let history_after_restore = term.history_size();

    println!(
        "After restore: offset={}, history={}",
        offset_after_restore, history_after_restore
    );

    // Display offset should be clamped
    assert!(
        offset_after_restore <= history_after_restore,
        "BUG: Display offset {} exceeds history size {} after restore",
        offset_after_restore,
        history_after_restore
    );
}
