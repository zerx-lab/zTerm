//! Terminal View Resize Bug Reproduction Tests
//!
//! Bug Description:
//! - Open terminal and enter a command that outputs more lines than the view height
//! - Click maximize or double-click the titlebar
//! - Content outside the view area disappears and cannot scroll up
//! - After restoring window size, history content is lost but can scroll up
//! - Scrolling content appears garbled with wrong background colors
//!
//! This test aims to reproduce the bug at the alacritty_terminal level
//! to understand the root cause of scrollback buffer issues during resize.

use alacritty_terminal::event::{Event as AlacEvent, EventListener};
use alacritty_terminal::grid::{Dimensions, Scroll};
use alacritty_terminal::index::{Column, Line, Point};
use alacritty_terminal::term::cell::Flags;
use alacritty_terminal::term::{Config, Term};
use alacritty_terminal::vte::ansi::{Processor, StdSyncHandler};

/// Dummy event listener for tests
#[derive(Clone)]
struct TestEventListener;

impl EventListener for TestEventListener {
    fn send_event(&self, _event: AlacEvent) {
        // Do nothing in tests
    }
}

/// Custom dimensions for testing resize behavior
#[derive(Debug, Clone, Copy)]
struct TestDimensions {
    cols: usize,
    lines: usize,
}

impl Dimensions for TestDimensions {
    fn total_lines(&self) -> usize {
        self.lines
    }

    fn screen_lines(&self) -> usize {
        self.lines
    }

    fn columns(&self) -> usize {
        self.cols
    }
}

/// Helper function to write text to terminal
fn write_to_terminal(term: &mut Term<TestEventListener>, text: &str) {
    let mut processor: Processor<StdSyncHandler> = Processor::new();
    processor.advance(term, text.as_bytes());
}

/// Helper function to get cell content at a position
fn get_cell_char(term: &Term<TestEventListener>, line: i32, col: usize) -> char {
    let point = Point::new(Line(line), Column(col));
    term.grid()[point].c
}

/// Helper function to count non-empty cells in terminal
fn count_visible_content(term: &Term<TestEventListener>) -> usize {
    let mut count = 0;
    for line in 0..term.screen_lines() as i32 {
        for col in 0..term.columns() {
            let c = get_cell_char(term, line, col);
            if c != ' ' && c != '\0' {
                count += 1;
            }
        }
    }
    count
}

/// Helper function to get all visible text from terminal
fn get_visible_text(term: &Term<TestEventListener>) -> String {
    let mut result = String::new();
    for line in 0..term.screen_lines() as i32 {
        for col in 0..term.columns() {
            let c = get_cell_char(term, line, col);
            if c != '\0' {
                result.push(c);
            }
        }
        result.push('\n');
    }
    result
}

/// Helper function to get scrollback + visible text
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
            let c = get_cell_char(term, line, col);
            if c != '\0' {
                result.push(c);
            }
        }
        result.push('\n');
    }
    result
}

// ============================================================================
// Bug Reproduction Tests
// ============================================================================

/// Test Case 1: Basic resize with scrollback content
///
/// This test reproduces the scenario where:
/// 1. Terminal has content exceeding view height (in scrollback)
/// 2. Window is maximized (terminal gets more lines)
/// 3. Window is restored (terminal gets fewer lines)
///
/// Bug: After restore, scrollback content may be lost or corrupted
#[test]
fn test_resize_with_scrollback_content_loss() {
    // Create a small terminal (10 lines, 80 cols)
    let initial_dims = TestDimensions { cols: 80, lines: 10 };
    let config = Config {
        scrolling_history: 1000, // Enable scrollback
        ..Config::default()
    };
    let listener = TestEventListener;
    let mut term = Term::new(config, &initial_dims, listener);

    // Write more lines than the view can display (20 lines)
    for i in 1..=20 {
        write_to_terminal(&mut term, &format!("Line {:02}: This is test content that should be preserved\r\n", i));
    }

    // Record state before resize
    let history_before = term.history_size();
    let content_before = get_all_text_with_scrollback(&term);

    println!("=== Before Maximize ===");
    println!("History size: {}", history_before);
    println!("Screen lines: {}", term.screen_lines());
    println!("Display offset: {}", term.grid().display_offset());

    // BUG CONDITION 1: History should contain lines that scrolled off
    assert!(history_before > 0, "Should have scrollback history");

    // Simulate maximize: increase terminal size significantly
    let maximized_dims = TestDimensions { cols: 80, lines: 40 };
    term.resize(maximized_dims);

    let history_after_max = term.history_size();
    let content_after_max = get_all_text_with_scrollback(&term);

    println!("\n=== After Maximize (40 lines) ===");
    println!("History size: {}", history_after_max);
    println!("Screen lines: {}", term.screen_lines());
    println!("Display offset: {}", term.grid().display_offset());

    // Content should still be accessible (possibly moved from scrollback to visible)
    // When screen grows, scrollback content may become visible

    // Simulate restore: decrease terminal size back to original
    term.resize(initial_dims);

    let history_after_restore = term.history_size();
    let content_after_restore = get_all_text_with_scrollback(&term);

    println!("\n=== After Restore (10 lines) ===");
    println!("History size: {}", history_after_restore);
    println!("Screen lines: {}", term.screen_lines());
    println!("Display offset: {}", term.grid().display_offset());

    // BUG CHECK: Content should be preserved after resize cycle
    // The bug is that content gets lost or corrupted

    // Check that we still have all the content
    // This assertion may FAIL, demonstrating the bug
    let line_01_present = content_after_restore.contains("Line 01:");
    let line_10_present = content_after_restore.contains("Line 10:");
    let line_20_present = content_after_restore.contains("Line 20:");

    println!("\n=== Content Verification ===");
    println!("Line 01 present: {}", line_01_present);
    println!("Line 10 present: {}", line_10_present);
    println!("Line 20 present: {}", line_20_present);

    // These assertions demonstrate the expected behavior
    // If they fail, it confirms the bug
    assert!(line_01_present, "BUG: Line 01 was lost during resize");
    assert!(line_10_present, "BUG: Line 10 was lost during resize");
    assert!(line_20_present, "BUG: Line 20 was lost during resize");
}

/// Test Case 2: Scroll position after resize
///
/// This test checks if display_offset is correctly maintained during resize
#[test]
fn test_scroll_position_after_resize() {
    let initial_dims = TestDimensions { cols: 80, lines: 10 };
    let config = Config {
        scrolling_history: 1000,
        ..Config::default()
    };
    let listener = TestEventListener;
    let mut term = Term::new(config, &initial_dims, listener);

    // Write many lines
    for i in 1..=50 {
        write_to_terminal(&mut term, &format!("Line {:02}\r\n", i));
    }

    // Scroll up to view history
    let history_size = term.history_size();
    println!("History size: {}", history_size);

    term.scroll_display(Scroll::Delta(10)); // Scroll up 10 lines
    let offset_before = term.grid().display_offset();
    println!("Display offset before resize: {}", offset_before);

    // Simulate maximize
    let maximized_dims = TestDimensions { cols: 80, lines: 40 };
    term.resize(maximized_dims);

    let offset_after_max = term.grid().display_offset();
    println!("Display offset after maximize: {}", offset_after_max);

    // Simulate restore
    term.resize(initial_dims);

    let offset_after_restore = term.grid().display_offset();
    println!("Display offset after restore: {}", offset_after_restore);

    // BUG CHECK: The display offset might become invalid after resize
    // This can cause "cannot scroll" or "garbled content" issues

    // Verify scroll offset is within valid range
    let max_valid_offset = term.history_size();
    assert!(
        offset_after_restore <= max_valid_offset,
        "BUG: Display offset {} exceeds history size {} after resize",
        offset_after_restore,
        max_valid_offset
    );
}

/// Test Case 3: Content integrity during rapid resize
///
/// This simulates the user rapidly maximizing and restoring the window
#[test]
fn test_rapid_resize_content_integrity() {
    let small_dims = TestDimensions { cols: 80, lines: 10 };
    let large_dims = TestDimensions { cols: 80, lines: 40 };
    let config = Config {
        scrolling_history: 1000,
        ..Config::default()
    };
    let listener = TestEventListener;
    let mut term = Term::new(config, &small_dims, listener);

    // Write content that exceeds view
    for i in 1..=30 {
        write_to_terminal(&mut term, &format!("MARKER_{:02}_CONTENT\r\n", i));
    }

    let initial_content = get_all_text_with_scrollback(&term);

    // Rapid resize cycles (simulating quick maximize/restore)
    for cycle in 0..5 {
        term.resize(large_dims);
        term.resize(small_dims);

        let current_content = get_all_text_with_scrollback(&term);

        // Check if content is preserved after each cycle
        let marker_01 = current_content.contains("MARKER_01_CONTENT");
        let marker_15 = current_content.contains("MARKER_15_CONTENT");
        let marker_30 = current_content.contains("MARKER_30_CONTENT");

        println!("Cycle {}: markers present: 01={}, 15={}, 30={}",
                 cycle, marker_01, marker_15, marker_30);

        // BUG: Content may degrade over multiple resize cycles
        assert!(marker_01, "BUG: MARKER_01 lost after cycle {}", cycle);
        assert!(marker_15, "BUG: MARKER_15 lost after cycle {}", cycle);
        assert!(marker_30, "BUG: MARKER_30 lost after cycle {}", cycle);
    }
}

/// Test Case 4: Cell attributes preservation during resize
///
/// This test checks if cell attributes (colors, flags) are preserved
/// Bug: "background colors are garbled" after resize
#[test]
fn test_cell_attributes_after_resize() {
    let initial_dims = TestDimensions { cols: 80, lines: 10 };
    let config = Config {
        scrolling_history: 1000,
        ..Config::default()
    };
    let listener = TestEventListener;
    let mut term = Term::new(config, &initial_dims, listener);

    // Write content with ANSI colors and attributes
    // Red text: \x1b[31m, Bold: \x1b[1m, Reset: \x1b[0m
    for i in 1..=20 {
        write_to_terminal(&mut term, &format!("\x1b[31;1mRed Line {:02}\x1b[0m Normal\r\n", i));
    }

    // Get cell flags before resize
    let mut flags_before: Vec<Flags> = Vec::new();
    for line in (-(term.history_size() as i32))..term.screen_lines() as i32 {
        for col in 0..10.min(term.columns()) {
            let point = Point::new(Line(line), Column(col));
            flags_before.push(term.grid()[point].flags);
        }
    }

    // Resize cycle
    let large_dims = TestDimensions { cols: 80, lines: 40 };
    term.resize(large_dims);
    term.resize(initial_dims);

    // Get cell flags after resize
    let mut flags_after: Vec<Flags> = Vec::new();
    for line in (-(term.history_size() as i32))..term.screen_lines() as i32 {
        for col in 0..10.min(term.columns()) {
            let point = Point::new(Line(line), Column(col));
            flags_after.push(term.grid()[point].flags);
        }
    }

    println!("Flags before: {} entries", flags_before.len());
    println!("Flags after: {} entries", flags_after.len());

    // Count BOLD flags
    let bold_before = flags_before.iter().filter(|f| f.contains(Flags::BOLD)).count();
    let bold_after = flags_after.iter().filter(|f| f.contains(Flags::BOLD)).count();

    println!("BOLD flags before: {}", bold_before);
    println!("BOLD flags after: {}", bold_after);

    // BUG CHECK: Cell attributes may be lost during resize
    // This can manifest as "garbled colors"
    if bold_before > 0 {
        assert!(
            bold_after > 0,
            "BUG: All BOLD attributes were lost during resize (before: {}, after: {})",
            bold_before,
            bold_after
        );
    }
}

/// Test Case 5: Cursor position during resize
///
/// Verify cursor position is correctly maintained
#[test]
fn test_cursor_position_during_resize() {
    let initial_dims = TestDimensions { cols: 80, lines: 10 };
    let config = Config {
        scrolling_history: 1000,
        ..Config::default()
    };
    let listener = TestEventListener;
    let mut term = Term::new(config, &initial_dims, listener);

    // Write content
    for i in 1..=5 {
        write_to_terminal(&mut term, &format!("Line {}\r\n", i));
    }
    write_to_terminal(&mut term, "Cursor here>");

    let cursor_before = term.renderable_content().cursor.point;
    println!("Cursor before resize: {:?}", cursor_before);

    // Maximize
    let large_dims = TestDimensions { cols: 80, lines: 40 };
    term.resize(large_dims);
    let cursor_after_max = term.renderable_content().cursor.point;
    println!("Cursor after maximize: {:?}", cursor_after_max);

    // Restore
    term.resize(initial_dims);
    let cursor_after_restore = term.renderable_content().cursor.point;
    println!("Cursor after restore: {:?}", cursor_after_restore);

    // Cursor column should be preserved
    assert_eq!(
        cursor_before.column, cursor_after_restore.column,
        "BUG: Cursor column changed during resize"
    );

    // Cursor line might change if content was reflowed
    // But should still be valid
    assert!(
        cursor_after_restore.line.0 >= 0 && cursor_after_restore.line.0 < term.screen_lines() as i32,
        "BUG: Cursor line {} is out of bounds after resize (screen_lines: {})",
        cursor_after_restore.line.0,
        term.screen_lines()
    );
}

/// Test Case 6: History truncation during resize
///
/// When terminal grows, scrollback content becomes visible
/// When terminal shrinks, visible content goes to scrollback
/// Bug: Content may be lost in this transition
#[test]
fn test_history_content_transition() {
    let small_dims = TestDimensions { cols: 80, lines: 5 };
    let config = Config {
        scrolling_history: 100,
        ..Config::default()
    };
    let listener = TestEventListener;
    let mut term = Term::new(config, &small_dims, listener);

    // Write 10 lines into a 5-line terminal
    // This means 5 lines should go into scrollback
    for i in 1..=10 {
        write_to_terminal(&mut term, &format!("History test line {:02}\r\n", i));
    }

    let history_small = term.history_size();
    println!("History with 5 lines: {}", history_small);

    // Grow terminal to 20 lines - some scrollback should become visible
    let medium_dims = TestDimensions { cols: 80, lines: 20 };
    term.resize(medium_dims);
    let history_medium = term.history_size();
    println!("History with 20 lines: {}", history_medium);

    // Shrink back to 5 lines - visible content should go back to scrollback
    term.resize(small_dims);
    let history_back = term.history_size();
    println!("History back to 5 lines: {}", history_back);

    // Verify all original content is still accessible
    let all_content = get_all_text_with_scrollback(&term);

    for i in 1..=10 {
        let expected = format!("History test line {:02}", i);
        assert!(
            all_content.contains(&expected),
            "BUG: Line '{}' was lost during resize transitions\nContent:\n{}",
            expected,
            all_content
        );
    }
}

/// Test Case 7: Very long lines during resize
///
/// Lines longer than the terminal width are wrapped
/// Resize can affect line wrapping behavior
#[test]
fn test_long_line_wrapping_during_resize() {
    let narrow_dims = TestDimensions { cols: 40, lines: 10 };
    let config = Config {
        scrolling_history: 1000,
        ..Config::default()
    };
    let listener = TestEventListener;
    let mut term = Term::new(config, &narrow_dims, listener);

    // Write a long line that will wrap multiple times
    let long_line = "A".repeat(100);
    write_to_terminal(&mut term, &long_line);
    write_to_terminal(&mut term, "\r\n");

    let visible_before = get_visible_text(&term);
    let a_count_before = visible_before.chars().filter(|c| *c == 'A').count();
    println!("'A' count before resize: {}", a_count_before);

    // Make terminal wider - line should unwrap
    let wide_dims = TestDimensions { cols: 120, lines: 10 };
    term.resize(wide_dims);

    // Make terminal narrow again
    term.resize(narrow_dims);

    let visible_after = get_visible_text(&term);
    let all_after = get_all_text_with_scrollback(&term);
    let a_count_after = all_after.chars().filter(|c| *c == 'A').count();
    println!("'A' count after resize: {}", a_count_after);

    // BUG CHECK: Characters should not be lost during rewrapping
    assert_eq!(
        a_count_before, a_count_after,
        "BUG: Characters were lost during line rewrapping (before: {}, after: {})",
        a_count_before, a_count_after
    );
}

/// Test Case 8: Resize at scrollback limit
///
/// Test behavior when scrollback is at or near its limit
#[test]
fn test_resize_at_scrollback_limit() {
    let dims = TestDimensions { cols: 80, lines: 10 };
    let config = Config {
        scrolling_history: 20, // Small scrollback limit
        ..Config::default()
    };
    let listener = TestEventListener;
    let mut term = Term::new(config, &dims, listener);

    // Fill up the scrollback completely (30 lines into 10-line terminal with 20 line history)
    for i in 1..=35 {
        write_to_terminal(&mut term, &format!("Scrollback limit test {:02}\r\n", i));
    }

    let history_before = term.history_size();
    println!("History before (should be at limit): {}", history_before);

    // Resize
    let large_dims = TestDimensions { cols: 80, lines: 40 };
    term.resize(large_dims);
    term.resize(dims);

    let history_after = term.history_size();
    println!("History after resize: {}", history_after);

    // History should not exceed the configured limit
    assert!(
        history_after <= 20,
        "History exceeded limit after resize: {} > 20",
        history_after
    );

    // Most recent content should be preserved
    let all_content = get_all_text_with_scrollback(&term);
    assert!(
        all_content.contains("Scrollback limit test 35"),
        "BUG: Most recent content was lost"
    );
}

/// Test Case 9: Empty terminal resize
///
/// Edge case: resize an empty terminal
#[test]
fn test_empty_terminal_resize() {
    let initial_dims = TestDimensions { cols: 80, lines: 10 };
    let config = Config::default();
    let listener = TestEventListener;
    let mut term = Term::new(config, &initial_dims, listener);

    // Don't write anything - terminal is empty

    // Resize should not panic
    let large_dims = TestDimensions { cols: 80, lines: 40 };
    term.resize(large_dims);
    term.resize(initial_dims);

    // Terminal should still be functional
    write_to_terminal(&mut term, "After resize");
    let content = get_visible_text(&term);
    assert!(content.contains("After resize"), "Terminal not functional after empty resize");
}

/// Test Case 10: Simulate the exact bug scenario
///
/// This test simulates the exact user scenario:
/// 1. Open terminal with small window
/// 2. Run a command that outputs many lines (e.g., `ls -la` on a large directory)
/// 3. Maximize window
/// 4. Restore window
/// 5. Try to scroll up - content should be accessible
#[test]
fn test_exact_bug_scenario_maximize_restore() {
    let initial_dims = TestDimensions { cols: 80, lines: 24 }; // Typical small window
    let config = Config {
        scrolling_history: 10000, // Large scrollback like real terminal
        ..Config::default()
    };
    let listener = TestEventListener;
    let mut term = Term::new(config, &initial_dims, listener);

    // Simulate shell prompt
    write_to_terminal(&mut term, "user@host:~$ ls -la\r\n");

    // Simulate large output (100 lines, exceeds 24-line view)
    for i in 1..=100 {
        write_to_terminal(&mut term, &format!(
            "-rw-r--r--  1 user user  4096 Jan {} {:02}:00 file{:03}.txt\r\n",
            i % 28 + 1, i % 24, i
        ));
    }

    // Simulate next prompt
    write_to_terminal(&mut term, "user@host:~$ ");

    println!("=== Initial State (24 lines) ===");
    println!("History size: {}", term.history_size());
    println!("Display offset: {}", term.grid().display_offset());

    // Record first line content (should be in scrollback)
    let first_line_present = get_all_text_with_scrollback(&term).contains("file001.txt");
    println!("file001.txt present before maximize: {}", first_line_present);
    assert!(first_line_present, "First output line should be in scrollback");

    // Step 1: MAXIMIZE (simulate double-click titlebar)
    let maximized_dims = TestDimensions { cols: 80, lines: 50 }; // Larger window
    term.resize(maximized_dims);

    println!("\n=== After Maximize (50 lines) ===");
    println!("History size: {}", term.history_size());
    println!("Display offset: {}", term.grid().display_offset());

    // Step 2: RESTORE (simulate restore button or double-click again)
    term.resize(initial_dims);

    println!("\n=== After Restore (24 lines) ===");
    println!("History size: {}", term.history_size());
    println!("Display offset: {}", term.grid().display_offset());

    // BUG CHECK 1: "视图区域外的内容会消失，无法向上滚动"
    // Content outside view should still be in scrollback
    let content_after = get_all_text_with_scrollback(&term);

    // Check if early content is still accessible
    let file001_present = content_after.contains("file001.txt");
    let file050_present = content_after.contains("file050.txt");
    let file100_present = content_after.contains("file100.txt");
    let prompt_present = content_after.contains("user@host:~$");

    println!("\n=== Content Check After Restore ===");
    println!("file001.txt present: {}", file001_present);
    println!("file050.txt present: {}", file050_present);
    println!("file100.txt present: {}", file100_present);
    println!("Prompt present: {}", prompt_present);

    // These assertions may FAIL, demonstrating the bug
    assert!(file001_present, "BUG: First output file001.txt was lost after maximize/restore");
    assert!(file050_present, "BUG: Middle output file050.txt was lost after maximize/restore");
    assert!(file100_present, "BUG: Last output file100.txt was lost after maximize/restore");
    assert!(prompt_present, "BUG: Shell prompt was lost after maximize/restore");

    // BUG CHECK 2: Try to scroll up (simulate user scrolling)
    term.scroll_display(Scroll::Delta(50)); // Scroll up 50 lines

    let scroll_offset = term.grid().display_offset();
    println!("\n=== After Scroll Up ===");
    println!("Display offset after scroll: {}", scroll_offset);

    // BUG CHECK 3: "滚动出现的内容是渲染错乱的"
    // Get content after scrolling
    let scrolled_content = get_visible_text(&term);
    println!("Visible content after scroll:\n{}", &scrolled_content[..200.min(scrolled_content.len())]);

    // Check that scrolled content is coherent (contains expected file entries)
    let contains_file_entry = scrolled_content.contains("file") || scrolled_content.contains("-rw-");
    assert!(
        contains_file_entry || scrolled_content.contains("user@host"),
        "BUG: Scrolled content appears garbled - no recognizable content"
    );
}

// ============================================================================
// Additional edge case tests
// ============================================================================

/// Test: Resize during active scroll position
#[test]
fn test_resize_while_scrolled_up() {
    let dims = TestDimensions { cols: 80, lines: 10 };
    let config = Config {
        scrolling_history: 1000,
        ..Config::default()
    };
    let listener = TestEventListener;
    let mut term = Term::new(config, &dims, listener);

    // Fill terminal with content
    for i in 1..=50 {
        write_to_terminal(&mut term, &format!("Scroll test line {:02}\r\n", i));
    }

    // Scroll up to view old content
    term.scroll_display(Scroll::Delta(30));
    let offset_before = term.grid().display_offset();
    println!("Scroll offset before resize: {}", offset_before);

    // Get currently visible content
    let visible_before = get_visible_text(&term);

    // Resize while scrolled
    let large_dims = TestDimensions { cols: 80, lines: 40 };
    term.resize(large_dims);

    let offset_after = term.grid().display_offset();
    println!("Scroll offset after grow: {}", offset_after);

    term.resize(dims);

    let offset_restored = term.grid().display_offset();
    println!("Scroll offset after restore: {}", offset_restored);

    // Scroll position might be adjusted, but content should still be accessible
    // Try to scroll to the content we were viewing
    term.scroll_display(Scroll::Delta(20));
    let visible_after = get_visible_text(&term);

    // Should still be able to see old content
    let all_content = get_all_text_with_scrollback(&term);
    assert!(
        all_content.contains("Scroll test line 01"),
        "BUG: Old content lost during resize while scrolled"
    );
}

/// Test: Very small terminal resize
#[test]
fn test_very_small_terminal() {
    let small_dims = TestDimensions { cols: 20, lines: 3 };
    let config = Config {
        scrolling_history: 100,
        ..Config::default()
    };
    let listener = TestEventListener;
    let mut term = Term::new(config, &small_dims, listener);

    // Write content
    for i in 1..=10 {
        write_to_terminal(&mut term, &format!("Small {:02}\r\n", i));
    }

    // Grow
    let normal_dims = TestDimensions { cols: 80, lines: 24 };
    term.resize(normal_dims);

    // Shrink back
    term.resize(small_dims);

    // Content should be preserved
    let all_content = get_all_text_with_scrollback(&term);
    assert!(
        all_content.contains("Small 01"),
        "BUG: Content lost in very small terminal resize"
    );
}
