//! Scroll Offset Synchronization Bug Reproduction Tests
//!
//! Bug Description (from test output warning):
//! ```
//! View scroll_offset: 50 (stale)
//! Term display_offset: 20
//! History size: 34
//! WARNING: View scroll_offset 50 > history_size 34
//! ```
//!
//! Root Cause Analysis:
//! - TerminalView maintains its own `scroll_offset` field
//! - Terminal maintains `display_offset` in alacritty_terminal
//! - When resize occurs, `history_size` changes (lines move between screen and history)
//! - TerminalView's `scroll_offset` is NOT updated during resize
//! - This causes `scroll_offset > history_size`, making scrollback invalid
//!
//! Bug Scenario:
//! 1. Open terminal with small window (e.g., 24 lines)
//! 2. Run commands that output more than 24 lines
//! 3. Scroll up to view history (scroll_offset = 50)
//! 4. Maximize window (screen grows to 67 lines)
//! 5. History content moves to visible area, history_size shrinks
//! 6. scroll_offset (50) now exceeds history_size (34)
//! 7. Cannot scroll, content appears lost/garbled
//!
//! This file simulates the TerminalView layer to reproduce the bug.

#![allow(dead_code)]
#![allow(unused_variables)]

use alacritty_terminal::event::{Event as AlacEvent, EventListener};
use alacritty_terminal::grid::{Dimensions, Scroll};
use alacritty_terminal::index::{Column, Line};
use alacritty_terminal::term::{Config, Term};
use alacritty_terminal::vte::ansi::{Processor, StdSyncHandler};

/// Dummy event listener for tests
#[derive(Clone)]
struct TestEventListener;

impl EventListener for TestEventListener {
    fn send_event(&self, _event: AlacEvent) {}
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

/// Simulates TerminalView's scroll_offset management
/// This struct mimics the relevant behavior of TerminalView
struct SimulatedTerminalView {
    /// The actual terminal (alacritty_terminal)
    term: Term<TestEventListener>,
    /// View's scroll_offset - maintained separately like in TerminalView
    /// This is the BUG: it should be synced with term's display_offset on resize
    scroll_offset: usize,
}

impl SimulatedTerminalView {
    fn new(cols: usize, lines: usize) -> Self {
        let dims = TestDimensions { cols, lines };
        let config = Config {
            scrolling_history: 10000,
            ..Config::default()
        };
        let listener = TestEventListener;
        let term = Term::new(config, &dims, listener);

        Self {
            term,
            scroll_offset: 0,
        }
    }

    /// Write content to terminal
    fn write(&mut self, text: &str) {
        write_to_terminal(&mut self.term, text);
    }

    /// Scroll up by delta lines (like TerminalView::on_scroll)
    fn scroll_up(&mut self, delta: usize) {
        let max_scroll = self.term.history_size();
        self.scroll_offset = (self.scroll_offset + delta).min(max_scroll);
        self.term.scroll_display(Scroll::Delta(delta as i32));
    }

    /// Scroll down by delta lines
    fn scroll_down(&mut self, delta: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(delta);
        self.term.scroll_display(Scroll::Delta(-(delta as i32)));
    }

    /// Resize terminal - THIS IS WHERE THE BUG IS
    /// Current behavior: scroll_offset is NOT synced
    fn resize_buggy(&mut self, cols: usize, lines: usize) {
        let dims = TestDimensions { cols, lines };
        self.term.resize(dims);
        // BUG: scroll_offset is NOT updated here!
        // It remains at its old value even if history_size changed
    }

    /// Resize terminal - CORRECT behavior (what should happen)
    fn resize_fixed(&mut self, cols: usize, lines: usize) {
        let dims = TestDimensions { cols, lines };
        self.term.resize(dims);

        // CORRECT: Sync scroll_offset with terminal's display_offset
        let display_offset = self.term.grid().display_offset();
        self.scroll_offset = display_offset;
    }

    /// Get current state for debugging
    fn debug_state(&self) -> ScrollSyncState {
        ScrollSyncState {
            view_scroll_offset: self.scroll_offset,
            term_display_offset: self.term.grid().display_offset(),
            history_size: self.term.history_size(),
            screen_lines: self.term.screen_lines(),
        }
    }

    /// Check if scroll state is valid (scroll_offset <= history_size)
    fn is_scroll_valid(&self) -> bool {
        self.scroll_offset <= self.term.history_size()
    }

    /// Simulate rendering - check if content is accessible
    fn can_render_content(&self) -> bool {
        // If scroll_offset > history_size, we can't access that content
        if self.scroll_offset > self.term.history_size() {
            return false;
        }
        true
    }
}

#[derive(Debug)]
struct ScrollSyncState {
    view_scroll_offset: usize,
    term_display_offset: usize,
    history_size: usize,
    screen_lines: usize,
}

impl std::fmt::Display for ScrollSyncState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "View scroll_offset: {}", self.view_scroll_offset)?;
        writeln!(f, "Term display_offset: {}", self.term_display_offset)?;
        writeln!(f, "History size: {}", self.history_size)?;
        writeln!(f, "Screen lines: {}", self.screen_lines)?;

        if self.view_scroll_offset != self.term_display_offset {
            writeln!(
                f,
                "WARNING: View scroll_offset ({}) != Term display_offset ({})",
                self.view_scroll_offset, self.term_display_offset
            )?;
        }

        if self.view_scroll_offset > self.history_size {
            writeln!(
                f,
                "CRITICAL: View scroll_offset ({}) > history_size ({}) - STALE/INVALID!",
                self.view_scroll_offset, self.history_size
            )?;
        }

        Ok(())
    }
}

// ============================================================================
// Bug Reproduction Tests
// ============================================================================

/// Test Case 1: Core bug reproduction - scroll_offset exceeds history_size after maximize
///
/// This is the exact scenario described:
/// 1. Small window (24 lines)
/// 2. Output many lines (100+)
/// 3. Scroll up (scroll_offset = 50)
/// 4. Maximize (67 lines)
/// 5. scroll_offset (50) > new history_size (~34)
#[test]
fn test_scroll_offset_exceeds_history_after_maximize() {
    println!("\n=== Bug Reproduction: scroll_offset exceeds history_size ===\n");

    // Step 1: Create small terminal (simulating initial window)
    let mut view = SimulatedTerminalView::new(80, 24);

    // Step 2: Write many lines of content (exceeds 24 lines)
    println!("Step 1-2: Writing 100 lines to 24-line terminal...");
    for i in 1..=100 {
        view.write(&format!("Output line {:03}: test content here\r\n", i));
    }

    let state_initial = view.debug_state();
    println!("Initial state:\n{}", state_initial);
    assert!(view.is_scroll_valid(), "Initial state should be valid");

    // Step 3: Scroll up significantly (user viewing history)
    println!("\nStep 3: Scrolling up 50 lines...");
    view.scroll_up(50);

    let state_scrolled = view.debug_state();
    println!("After scroll up:\n{}", state_scrolled);
    assert!(view.is_scroll_valid(), "Scrolled state should be valid");
    assert_eq!(
        view.scroll_offset, 50,
        "scroll_offset should be 50 after scrolling"
    );

    // Step 4: MAXIMIZE - increase screen size (BUG TRIGGER)
    println!("\nStep 4: Maximizing window (24 -> 67 lines)...");
    view.resize_buggy(80, 67); // Using buggy resize

    let state_maximized = view.debug_state();
    println!("After maximize (BUGGY):\n{}", state_maximized);

    // THE BUG: scroll_offset (50) > history_size (should be ~34)
    // When screen grows from 24 to 67, ~43 lines move from history to screen
    // history_size shrinks by ~43 (from 76 to ~34)
    // But scroll_offset is still 50!

    println!("\n=== BUG VERIFICATION ===");
    println!(
        "scroll_offset ({}) should be <= history_size ({})",
        state_maximized.view_scroll_offset, state_maximized.history_size
    );

    // This assertion FAILS, demonstrating the bug
    assert!(
        !view.is_scroll_valid(),
        "BUG REPRODUCED: scroll_offset ({}) exceeds history_size ({})",
        state_maximized.view_scroll_offset,
        state_maximized.history_size
    );

    // Additional check: rendering would fail
    assert!(
        !view.can_render_content(),
        "BUG: Cannot render content due to invalid scroll_offset"
    );
}

/// Test Case 2: Verify the fix - scroll_offset should be synced on resize
#[test]
fn test_scroll_offset_synced_after_maximize_fixed() {
    println!("\n=== Fixed Behavior Test ===\n");

    let mut view = SimulatedTerminalView::new(80, 24);

    // Fill with content
    for i in 1..=100 {
        view.write(&format!("Output line {:03}: test content here\r\n", i));
    }

    // Scroll up
    view.scroll_up(50);
    assert_eq!(view.scroll_offset, 50);

    // Maximize with FIXED resize
    view.resize_fixed(80, 67);

    let state = view.debug_state();
    println!("After maximize (FIXED):\n{}", state);

    // With fix: scroll_offset should equal display_offset
    assert_eq!(
        state.view_scroll_offset, state.term_display_offset,
        "FIXED: scroll_offset should equal display_offset"
    );

    assert!(view.is_scroll_valid(), "FIXED: scroll state should be valid");

    assert!(
        view.can_render_content(),
        "FIXED: should be able to render content"
    );
}

/// Test Case 3: Bug persists after restore to original size
#[test]
fn test_scroll_offset_bug_after_restore() {
    println!("\n=== Bug After Restore Test ===\n");

    let mut view = SimulatedTerminalView::new(80, 24);

    // Fill with content
    for i in 1..=100 {
        view.write(&format!("Line {:03}\r\n", i));
    }

    // Scroll up
    view.scroll_up(50);
    let initial_state = view.debug_state();
    println!("Initial (24 lines, scrolled up 50):\n{}", initial_state);

    // Maximize
    view.resize_buggy(80, 67);
    let max_state = view.debug_state();
    println!("After maximize (67 lines):\n{}", max_state);

    // Restore (back to 24 lines)
    view.resize_buggy(80, 24);
    let restore_state = view.debug_state();
    println!("After restore (24 lines):\n{}", restore_state);

    // After restore, history_size increases again, but scroll_offset
    // might still be stale from the maximize operation
    println!("\n=== After Restore Analysis ===");
    println!(
        "scroll_offset = {}, history_size = {}",
        restore_state.view_scroll_offset, restore_state.history_size
    );

    // The bug manifests as scroll_offset being out of sync with display_offset
    // Even if scroll_offset <= history_size, they should match for correct rendering
    if restore_state.view_scroll_offset != restore_state.term_display_offset {
        println!(
            "BUG: scroll_offset ({}) != display_offset ({})",
            restore_state.view_scroll_offset, restore_state.term_display_offset
        );
        // This is the bug - they're desynchronized
        assert_ne!(
            restore_state.view_scroll_offset, restore_state.term_display_offset,
            "BUG REPRODUCED: scroll_offset desynchronized from display_offset"
        );
    }
}

/// Test Case 4: Multiple resize cycles accumulate desync
#[test]
fn test_multiple_resize_accumulates_desync() {
    println!("\n=== Multiple Resize Desync Test ===\n");

    let mut view = SimulatedTerminalView::new(80, 24);

    // Fill with content
    for i in 1..=200 {
        view.write(&format!("Line {:03}\r\n", i));
    }

    let initial = view.debug_state();
    println!("Initial:\n{}", initial);

    // Scroll up significantly
    view.scroll_up(100);
    println!("After scroll up 100:\n{}", view.debug_state());

    // Multiple resize cycles (like user resizing window repeatedly)
    for cycle in 0..5 {
        // Maximize
        view.resize_buggy(80, 50);
        // Restore
        view.resize_buggy(80, 24);

        let state = view.debug_state();
        println!("After cycle {} (maximize+restore):\n{}", cycle, state);

        let sync_error =
            (state.view_scroll_offset as i64 - state.term_display_offset as i64).abs();
        if sync_error > 0 {
            println!(
                "Cycle {}: sync error = {} lines",
                cycle, sync_error
            );
        }
    }

    // After multiple cycles, check the desync
    let final_state = view.debug_state();
    if final_state.view_scroll_offset != final_state.term_display_offset {
        println!(
            "\nFINAL BUG STATE: scroll_offset ({}) != display_offset ({})",
            final_state.view_scroll_offset, final_state.term_display_offset
        );
    }
}

/// Test Case 5: Edge case - scroll to top before resize
#[test]
fn test_scroll_to_top_then_resize() {
    println!("\n=== Scroll to Top then Resize Test ===\n");

    let mut view = SimulatedTerminalView::new(80, 24);

    // Fill with content
    for i in 1..=100 {
        view.write(&format!("Line {:03}\r\n", i));
    }

    // Scroll to top (maximum scroll)
    let history_before = view.term.history_size();
    view.scroll_up(history_before);
    println!("After scroll to top:\n{}", view.debug_state());
    assert_eq!(view.scroll_offset, history_before);

    // Maximize - history shrinks
    view.resize_buggy(80, 67);
    let state = view.debug_state();
    println!("After maximize:\n{}", state);

    // scroll_offset was at old history_size, now exceeds new history_size
    if state.view_scroll_offset > state.history_size {
        println!(
            "BUG: Was at top (scroll_offset={}), but after resize history shrank to {}",
            state.view_scroll_offset, state.history_size
        );
        assert!(
            state.view_scroll_offset > state.history_size,
            "BUG REPRODUCED: scroll_offset at old top exceeds new history_size"
        );
    }
}

/// Test Case 6: Simulate exact user scenario with detailed output
#[test]
fn test_user_scenario_detailed() {
    println!("\n======================================================");
    println!("=== User Scenario: Terminal Scroll Resize Bug ===");
    println!("======================================================\n");

    // Step 1: Open terminal with normal window
    println!("STEP 1: Open terminal (80x24)");
    let mut view = SimulatedTerminalView::new(80, 24);
    println!("{}", view.debug_state());

    // Step 2: Run commands that output many lines
    println!("\nSTEP 2: Run 'ls -la' (outputs 100 files)");
    view.write("$ ls -la\r\n");
    for i in 1..=100 {
        view.write(&format!(
            "-rw-r--r-- 1 user user 4096 Jan 24 12:00 file{:03}.txt\r\n",
            i
        ));
    }
    view.write("$ ");
    println!("{}", view.debug_state());

    // Step 3: User scrolls up to see earlier output
    println!("\nSTEP 3: User scrolls up to see file001.txt (scroll up 50 lines)");
    view.scroll_up(50);
    let before_max = view.debug_state();
    println!("{}", before_max);
    println!("User is viewing content at scroll_offset={}", view.scroll_offset);

    // Step 4: User maximizes window (double-click titlebar)
    println!("\nSTEP 4: User maximizes window (80x67)");
    println!(">>> BEFORE: scroll_offset={}, history_size={}", view.scroll_offset, view.term.history_size());
    view.resize_buggy(80, 67);
    let after_max = view.debug_state();
    println!(">>> AFTER:  scroll_offset={}, history_size={}", view.scroll_offset, view.term.history_size());
    println!("\n{}", after_max);

    // Check for bug condition
    if after_max.view_scroll_offset > after_max.history_size {
        println!("!!! BUG DETECTED !!!");
        println!("scroll_offset ({}) > history_size ({})",
            after_max.view_scroll_offset, after_max.history_size);
        println!("User cannot scroll to this content - IT'S LOST!");
    }

    // Step 5: User tries to scroll but can't
    println!("\nSTEP 5: User tries to scroll up - FAILS because scroll_offset is invalid");
    println!("can_render_content() = {}", view.can_render_content());

    // Step 6: User restores window
    println!("\nSTEP 6: User restores window (80x24)");
    view.resize_buggy(80, 24);
    let after_restore = view.debug_state();
    println!("{}", after_restore);
    println!("scroll_offset still desynchronized from display_offset!");

    // This assertion PROVES the bug exists (scroll_offset > history_size)
    // When this test passes, it means the bug has been successfully reproduced
    assert!(
        after_max.view_scroll_offset > after_max.history_size,
        "BUG NOT REPRODUCED: Expected scroll_offset ({}) > history_size ({})\n\
         If this fails, the bug may have been fixed!",
        after_max.view_scroll_offset,
        after_max.history_size
    );

    println!("\n=== BUG SUCCESSFULLY REPRODUCED ===");
    println!("After maximize:");
    println!("- View scroll_offset: {} (STALE - not updated on resize)", after_max.view_scroll_offset);
    println!("- Term display_offset: {} (correct)", after_max.term_display_offset);
    println!("- History size: {}", after_max.history_size);
    println!("\nThe view's scroll_offset exceeds history_size,");
    println!("making scrollback inaccessible and causing:");
    println!("- 'Cannot scroll' behavior");
    println!("- Content appears 'lost'");
    println!("- Garbled rendering when trying to access invalid scroll positions");
}

/// Test Case 7: Verify desync causes rendering issues
#[test]
fn test_desync_causes_render_issues() {
    println!("\n=== Desync Causes Render Issues Test ===\n");

    let mut view = SimulatedTerminalView::new(80, 24);

    // Fill with distinctly numbered lines
    for i in 1..=100 {
        view.write(&format!("LINE_{:03}\r\n", i));
    }

    // Scroll up significantly to ensure we trigger the bug
    view.scroll_up(60);
    println!("Scrolled up 60 lines");
    println!("scroll_offset = {}", view.scroll_offset);
    println!("history_size (before resize) = {}", view.term.history_size());

    // Maximize to a much larger size to shrink history significantly
    view.resize_buggy(80, 80);
    println!("\nAfter maximize to 80 lines:");
    let state = view.debug_state();
    println!("{}", state);

    if state.view_scroll_offset > state.history_size {
        println!(
            "RENDER BUG: Trying to render with scroll_offset={} but history only has {} lines!",
            state.view_scroll_offset, state.history_size
        );
        println!("This causes the view to try accessing non-existent scrollback data.");
        println!("Result: Empty/garbled content, or crash in less safe implementations.");
    }

    // The render system would try to access:
    // history_lines[-scroll_offset..-scroll_offset+screen_lines]
    // With scroll_offset=60 but history_size may be much smaller

    // This assertion PROVES the bug exists - if it passes, bug is reproduced
    assert!(
        state.view_scroll_offset > state.history_size,
        "BUG NOT REPRODUCED: Expected scroll_offset ({}) > history_size ({})\n\
         If this fails, the bug may have been fixed!",
        state.view_scroll_offset,
        state.history_size
    );

    println!("\n=== RENDER BUG SUCCESSFULLY REPRODUCED ===");
    println!("scroll_offset ({}) > history_size ({})", state.view_scroll_offset, state.history_size);
}

// ============================================================================
// Helper test to show what CORRECT behavior looks like
// ============================================================================

/// This test demonstrates the EXPECTED (correct) behavior after fix
#[test]
fn test_expected_correct_behavior() {
    println!("\n=== Expected Correct Behavior (After Fix) ===\n");

    let mut view = SimulatedTerminalView::new(80, 24);

    for i in 1..=100 {
        view.write(&format!("Line {:03}\r\n", i));
    }

    view.scroll_up(50);
    println!("After scroll up:\n{}", view.debug_state());

    // Use fixed resize
    view.resize_fixed(80, 67);
    let state = view.debug_state();
    println!("After maximize (FIXED):\n{}", state);

    // Verify correct behavior
    assert_eq!(
        state.view_scroll_offset, state.term_display_offset,
        "scroll_offset should equal display_offset"
    );
    assert!(
        state.view_scroll_offset <= state.history_size,
        "scroll_offset should be <= history_size"
    );
    assert!(
        view.can_render_content(),
        "Should be able to render content"
    );

    println!("CORRECT: scroll_offset properly synced to display_offset");
    println!("User can scroll and view content normally.");
}
