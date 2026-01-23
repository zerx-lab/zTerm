//! Performance tests for axon_terminal
//!
//! These tests verify the performance characteristics of terminal core components.
//! Following Zed editor's testing approach with benchmark-style tests.
//!
//! Run with: cargo test --package axon_terminal --test performance_tests -- --nocapture
//!
//! Performance targets (approximate):
//! - Grid operations: < 10ms for 1000x1000 operations
//! - ANSI parsing: > 10MB/s throughput
//! - Cell operations: < 1ms for 10000 operations
//!
//! Note: These tests focus on core terminal components (Grid, Cell, Row, Parser)
//! that don't require GPUI. TerminalBounds tests are commented out as they require
//! GPUI which needs additional system libraries.

use axon_terminal::buffer::{Cell, CellFlags, Color, Grid, Row};
use axon_terminal::parser::AnsiParser;
use axon_terminal::TerminalSize;
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
// Grid Performance Tests
// ============================================================================
mod grid_performance_tests {
    use super::*;

    #[test]
    fn test_grid_creation_small() {
        // Standard terminal size: 80x24
        let _grid = assert_performance(
            "Grid creation 80x24",
            Duration::from_millis(1),
            || Grid::new(80, 24),
        );
    }

    #[test]
    fn test_grid_creation_medium() {
        // Large terminal: 200x100
        let _grid = assert_performance(
            "Grid creation 200x100",
            Duration::from_millis(5),
            || Grid::new(200, 100),
        );
    }

    #[test]
    fn test_grid_creation_large() {
        // Very large grid: 500x500
        let _grid = assert_performance(
            "Grid creation 500x500",
            Duration::from_millis(50),
            || Grid::new(500, 500),
        );
    }

    #[test]
    fn test_grid_creation_huge() {
        // Extreme case: 1000x1000
        let _grid = assert_performance(
            "Grid creation 1000x1000",
            Duration::from_millis(200),
            || Grid::new(1000, 1000),
        );
    }

    #[test]
    fn test_grid_write_char_performance() {
        let mut grid = Grid::new(80, 24);

        assert_performance(
            "Write 10000 chars",
            Duration::from_millis(10),
            || {
                for _ in 0..10000 {
                    grid.write_char('A');
                }
            },
        );
    }

    #[test]
    fn test_grid_write_char_benchmark() {
        let mut grid = Grid::new(80, 100);

        let avg = benchmark("Write single char", 10000, || {
            grid.write_char('X');
            // Reset cursor periodically to avoid excessive scrolling
            if grid.cursor().1 >= 90 {
                grid.set_cursor(0, 0);
            }
        });

        assert!(avg < Duration::from_micros(100), "Single char write too slow: {:?}", avg);
    }

    #[test]
    fn test_grid_scroll_up_performance() {
        let mut grid = Grid::new(80, 24);
        // Fill with content
        for _ in 0..80 * 24 {
            grid.write_char('X');
        }

        assert_performance(
            "Scroll up 1000 times",
            Duration::from_millis(50),
            || {
                for _ in 0..1000 {
                    grid.scroll_up(1);
                }
            },
        );
    }

    #[test]
    fn test_grid_scroll_down_performance() {
        let mut grid = Grid::new(80, 24);

        assert_performance(
            "Scroll down 1000 times",
            Duration::from_millis(50),
            || {
                for _ in 0..1000 {
                    grid.scroll_down(1);
                }
            },
        );
    }

    #[test]
    fn test_grid_scroll_large_delta() {
        let mut grid = Grid::new(80, 1000);

        assert_performance(
            "Scroll up by 500 lines",
            Duration::from_millis(10),
            || {
                grid.scroll_up(500);
            },
        );
    }

    #[test]
    fn test_grid_resize_performance() {
        let mut grid = Grid::new(80, 24);

        assert_performance(
            "Resize grid 100 times",
            Duration::from_millis(50),
            || {
                for i in 0..100 {
                    let cols = 80 + (i % 20);
                    let rows = 24 + (i % 10);
                    grid.resize(cols, rows);
                }
            },
        );
    }

    #[test]
    fn test_grid_resize_grow_shrink_cycle() {
        let mut grid = Grid::new(80, 24);

        assert_performance(
            "Grow/shrink cycle 50 times",
            Duration::from_millis(100),
            || {
                for _ in 0..50 {
                    grid.resize(200, 100);
                    grid.resize(80, 24);
                }
            },
        );
    }

    #[test]
    fn test_grid_clear_performance() {
        let mut grid = Grid::new(200, 100);
        // Fill with content
        for _ in 0..1000 {
            grid.write_char('X');
        }

        assert_performance(
            "Clear grid 1000 times",
            Duration::from_millis(100),
            || {
                for _ in 0..1000 {
                    grid.clear();
                }
            },
        );
    }

    #[test]
    fn test_grid_cell_access_performance() {
        let grid = Grid::new(80, 24);

        assert_performance(
            "Cell access 100000 times",
            Duration::from_millis(50),
            || {
                for _ in 0..100000 {
                    let _ = grid.get_cell(40, 12);
                }
            },
        );
    }

    #[test]
    fn test_grid_cell_access_random() {
        let grid = Grid::new(80, 24);

        assert_performance(
            "Random cell access",
            Duration::from_millis(50),
            || {
                for i in 0..100000 {
                    let col = i % 80;
                    let row = (i / 80) % 24;
                    let _ = grid.get_cell(col, row);
                }
            },
        );
    }

    #[test]
    fn test_grid_row_iteration_performance() {
        let grid = Grid::new(200, 100);

        assert_performance(
            "Iterate all rows 100 times",
            Duration::from_millis(10),
            || {
                for _ in 0..100 {
                    for row in grid.iter_rows() {
                        let _ = row.len();
                    }
                }
            },
        );
    }

    #[test]
    fn test_grid_to_string_performance() {
        let mut grid = Grid::new(80, 24);
        // Fill with content
        for _ in 0..80 * 24 {
            grid.write_char('X');
        }

        assert_performance(
            "Grid to string 100 times",
            Duration::from_millis(50),
            || {
                for _ in 0..100 {
                    let _ = grid.to_string_content();
                }
            },
        );
    }

    #[test]
    fn test_grid_cursor_movement_performance() {
        let mut grid = Grid::new(80, 24);

        assert_performance(
            "Cursor movement 100000 times",
            Duration::from_millis(20),
            || {
                for i in 0..100000 {
                    grid.set_cursor(i % 80, (i / 80) % 24);
                }
            },
        );
    }
}

// ============================================================================
// Row Performance Tests
// ============================================================================
mod row_performance_tests {
    use super::*;

    #[test]
    fn test_row_creation_performance() {
        assert_performance(
            "Create 1000 rows of 80 cols",
            Duration::from_millis(20),
            || {
                let rows: Vec<Row> = (0..1000).map(|_| Row::new(80)).collect();
                rows
            },
        );
    }

    #[test]
    fn test_row_creation_wide() {
        assert_performance(
            "Create 100 rows of 500 cols",
            Duration::from_millis(20),
            || {
                let rows: Vec<Row> = (0..100).map(|_| Row::new(500)).collect();
                rows
            },
        );
    }

    #[test]
    fn test_row_resize_performance() {
        let mut row = Row::new(80);

        assert_performance(
            "Resize row 10000 times",
            Duration::from_millis(50),
            || {
                for i in 0..10000 {
                    row.resize(80 + (i % 100));
                }
            },
        );
    }

    #[test]
    fn test_row_clear_performance() {
        let mut row = Row::new(200);

        assert_performance(
            "Clear row 10000 times",
            Duration::from_millis(50),
            || {
                for _ in 0..10000 {
                    row.clear();
                }
            },
        );
    }

    #[test]
    fn test_row_cell_access_performance() {
        let mut row = Row::new(200);

        assert_performance(
            "Row cell access 100000 times",
            Duration::from_millis(20),
            || {
                for i in 0..100000 {
                    if let Some(cell) = row.get_mut(i % 200) {
                        cell.c = 'X';
                    }
                }
            },
        );
    }

    #[test]
    fn test_row_iteration_performance() {
        let row = Row::new(200);

        assert_performance(
            "Iterate row 10000 times",
            Duration::from_millis(20),
            || {
                for _ in 0..10000 {
                    for cell in row.iter() {
                        let _ = cell.c;
                    }
                }
            },
        );
    }

    #[test]
    fn test_row_clone_performance() {
        let row = Row::new(200);

        assert_performance(
            "Clone row 10000 times",
            Duration::from_millis(100),
            || {
                for _ in 0..10000 {
                    let _ = row.clone();
                }
            },
        );
    }
}

// ============================================================================
// Cell Performance Tests
// ============================================================================
mod cell_performance_tests {
    use super::*;

    #[test]
    fn test_cell_creation_performance() {
        assert_performance(
            "Create 100000 cells",
            Duration::from_millis(10),
            || {
                let cells: Vec<Cell> = (0..100000).map(|_| Cell::default()).collect();
                cells
            },
        );
    }

    #[test]
    fn test_cell_reset_performance() {
        let mut cells: Vec<Cell> = (0..10000).map(|_| Cell {
            c: 'X',
            fg: Color::Named(1),
            bg: Color::Named(2),
            flags: CellFlags {
                bold: true,
                underline: true,
                ..Default::default()
            },
            width: 1,
        }).collect();

        assert_performance(
            "Reset 10000 cells",
            Duration::from_millis(5),
            || {
                for cell in &mut cells {
                    cell.reset();
                }
            },
        );
    }

    #[test]
    fn test_cell_clone_performance() {
        let cell = Cell {
            c: 'X',
            fg: Color::Rgb(255, 128, 64),
            bg: Color::Indexed(100),
            flags: CellFlags {
                bold: true,
                italic: true,
                underline: true,
                strikethrough: true,
                ..Default::default()
            },
            width: 1,
        };

        assert_performance(
            "Clone cell 100000 times",
            Duration::from_millis(20),
            || {
                for _ in 0..100000 {
                    let _ = cell.clone();
                }
            },
        );
    }

    #[test]
    fn test_cell_flags_operations() {
        let mut flags = CellFlags::default();

        assert_performance(
            "Cell flags operations 100000 times",
            Duration::from_millis(10),
            || {
                for _ in 0..100000 {
                    flags.bold = !flags.bold;
                    flags.italic = !flags.italic;
                    flags.underline = !flags.underline;
                    let _ = flags.clone();
                }
            },
        );
    }

    #[test]
    fn test_color_operations() {
        assert_performance(
            "Color operations 100000 times",
            Duration::from_millis(20),
            || {
                for i in 0..100000 {
                    let _color1 = Color::Named((i % 16) as u8);
                    let _color2 = Color::Indexed((i % 256) as u8);
                    let _color3 = Color::Rgb((i % 256) as u8, ((i / 256) % 256) as u8, ((i / 65536) % 256) as u8);
                }
            },
        );
    }
}

// ============================================================================
// ANSI Parser Performance Tests
// ============================================================================
mod ansi_parser_performance_tests {
    use super::*;

    #[test]
    fn test_ansi_parser_plain_text() {
        let mut parser = AnsiParser::new();
        let mut grid = Grid::new(80, 24);
        let text = "Hello, World! ".repeat(1000);
        let bytes = text.as_bytes();

        let (_, elapsed) = measure_time(|| {
            parser.process(bytes, &mut grid);
        });

        let throughput = bytes.len() as f64 / elapsed.as_secs_f64() / 1_000_000.0;
        println!("[PERF] ANSI parser plain text: {:.2} MB/s", throughput);

        assert!(throughput > 10.0, "Parser throughput too low: {:.2} MB/s", throughput);
    }

    #[test]
    fn test_ansi_parser_colored_text() {
        let mut parser = AnsiParser::new();
        let mut grid = Grid::new(80, 100);

        // Generate colored text with ANSI sequences
        let mut colored_text = String::new();
        for i in 0..1000 {
            colored_text.push_str(&format!("\x1b[{}mText{}\x1b[0m ", 31 + (i % 7), i));
        }
        let bytes = colored_text.as_bytes();

        let (_, elapsed) = measure_time(|| {
            parser.process(bytes, &mut grid);
        });

        let throughput = bytes.len() as f64 / elapsed.as_secs_f64() / 1_000_000.0;
        println!("[PERF] ANSI parser colored text: {:.2} MB/s ({} bytes in {:?})", throughput, bytes.len(), elapsed);

        assert!(throughput > 1.0, "Parser throughput too low: {:.2} MB/s", throughput);
    }

    #[test]
    fn test_ansi_parser_sgr_sequences() {
        let mut parser = AnsiParser::new();
        let mut grid = Grid::new(80, 50);

        // Generate many SGR sequences
        let mut sgr_text = String::new();
        for _ in 0..500 {
            sgr_text.push_str("\x1b[1;4;31;42mBold underline red on green\x1b[0m ");
            sgr_text.push_str("\x1b[38;5;200;48;5;50m256 color\x1b[0m ");
            sgr_text.push_str("\x1b[38;2;255;128;64;48;2;0;64;128mTrue color\x1b[0m\n");
        }
        let bytes = sgr_text.as_bytes();

        assert_performance(
            &format!("Parse {} bytes of SGR sequences", bytes.len()),
            Duration::from_millis(100),
            || {
                parser.process(bytes, &mut grid);
            },
        );
    }

    #[test]
    fn test_ansi_parser_cursor_movements() {
        let mut parser = AnsiParser::new();
        let mut grid = Grid::new(80, 24);

        // Generate cursor movement sequences
        let mut cursor_text = String::new();
        for _ in 0..1000 {
            cursor_text.push_str("\x1b[5A"); // Up 5
            cursor_text.push_str("\x1b[3B"); // Down 3
            cursor_text.push_str("\x1b[10C"); // Forward 10
            cursor_text.push_str("\x1b[2D"); // Back 2
            cursor_text.push_str("\x1b[10;40H"); // Position
            cursor_text.push('X');
        }
        let bytes = cursor_text.as_bytes();

        assert_performance(
            &format!("Parse {} bytes of cursor movements", bytes.len()),
            Duration::from_millis(50),
            || {
                parser.process(bytes, &mut grid);
            },
        );
    }

    #[test]
    fn test_ansi_parser_erase_sequences() {
        let mut parser = AnsiParser::new();
        let mut grid = Grid::new(80, 24);

        // Generate erase sequences
        let mut erase_text = String::new();
        for _ in 0..500 {
            erase_text.push_str("Some text here\x1b[K"); // Erase to end of line
            erase_text.push_str("\x1b[1K"); // Erase to start of line
            erase_text.push_str("\x1b[2K"); // Erase entire line
            erase_text.push_str("\x1b[J"); // Erase to end of screen
            erase_text.push_str("\n");
        }
        let bytes = erase_text.as_bytes();

        assert_performance(
            &format!("Parse {} bytes of erase sequences", bytes.len()),
            Duration::from_millis(50),
            || {
                parser.process(bytes, &mut grid);
            },
        );
    }

    #[test]
    fn test_ansi_parser_control_chars() {
        let mut parser = AnsiParser::new();
        let mut grid = Grid::new(80, 24);

        // Generate control character sequences
        let mut control_text = String::new();
        for _ in 0..1000 {
            control_text.push_str("Line\r\nNew line\t");
            control_text.push_str("Tab\x08Back\x07Bell");
        }
        let bytes = control_text.as_bytes();

        assert_performance(
            &format!("Parse {} bytes of control chars", bytes.len()),
            Duration::from_millis(30),
            || {
                parser.process(bytes, &mut grid);
            },
        );
    }

    #[test]
    fn test_ansi_parser_mixed_content() {
        let mut parser = AnsiParser::new();
        let mut grid = Grid::new(120, 50);

        // Simulate real terminal output (like `ls -la --color`)
        let mut mixed_text = String::new();
        for i in 0..500 {
            mixed_text.push_str(&format!(
                "\x1b[34mdrwxr-xr-x\x1b[0m  \x1b[33m{}\x1b[0m user group {:>8} Jan {:>2} 12:00 \x1b[32mfile_{}.txt\x1b[0m\r\n",
                i % 10, i * 1024, i % 31 + 1, i
            ));
        }
        let bytes = mixed_text.as_bytes();

        let (_, elapsed) = measure_time(|| {
            parser.process(bytes, &mut grid);
        });

        let throughput = bytes.len() as f64 / elapsed.as_secs_f64() / 1_000_000.0;
        println!("[PERF] ANSI parser mixed content: {:.2} MB/s ({} bytes)", throughput, bytes.len());

        assert!(throughput > 1.0, "Parser throughput too low: {:.2} MB/s", throughput);
    }

    #[test]
    fn test_ansi_parser_unicode() {
        let mut parser = AnsiParser::new();
        let mut grid = Grid::new(80, 50);

        // Generate Unicode content
        let mut unicode_text = String::new();
        for _ in 0..200 {
            unicode_text.push_str("Hello 你好 こんにちは 안녕하세요 مرحبا 🎉🚀💻\r\n");
        }
        let bytes = unicode_text.as_bytes();

        assert_performance(
            &format!("Parse {} bytes of Unicode", bytes.len()),
            Duration::from_millis(50),
            || {
                parser.process(bytes, &mut grid);
            },
        );
    }

    #[test]
    fn test_ansi_parser_large_output() {
        let mut parser = AnsiParser::new();
        let mut grid = Grid::new(200, 100);

        // Simulate large file cat output (1MB)
        let line = "x".repeat(200) + "\n";
        let large_text = line.repeat(5000);
        let bytes = large_text.as_bytes();

        let (_, elapsed) = measure_time(|| {
            parser.process(bytes, &mut grid);
        });

        let throughput = bytes.len() as f64 / elapsed.as_secs_f64() / 1_000_000.0;
        println!(
            "[PERF] ANSI parser large output: {:.2} MB/s ({:.2} MB in {:?})",
            throughput,
            bytes.len() as f64 / 1_000_000.0,
            elapsed
        );

        assert!(throughput > 5.0, "Large output throughput too low: {:.2} MB/s", throughput);
    }

    #[test]
    fn test_ansi_parser_benchmark() {
        let mut parser = AnsiParser::new();
        let mut grid = Grid::new(80, 24);
        let text = "Hello World! \x1b[31mRed\x1b[0m Normal\r\n";
        let bytes = text.as_bytes();

        let avg = benchmark("Parse line with color", 10000, || {
            parser.process(bytes, &mut grid);
            grid.set_cursor(0, 0); // Reset for consistent testing
        });

        assert!(avg < Duration::from_micros(100), "Parse too slow: {:?}", avg);
    }
}

// ============================================================================
// TerminalSize Performance Tests
// ============================================================================
mod terminal_size_performance_tests {
    use super::*;

    #[test]
    fn test_terminal_size_creation() {
        assert_performance(
            "Create TerminalSize 100000 times",
            Duration::from_millis(10),
            || {
                for i in 0..100000 {
                    let _ = TerminalSize {
                        cols: (80 + i % 100) as u16,
                        rows: (24 + i % 50) as u16,
                    };
                }
            },
        );
    }

    #[test]
    fn test_terminal_size_default() {
        assert_performance(
            "TerminalSize::default 100000 times",
            Duration::from_millis(5),
            || {
                for _ in 0..100000 {
                    let _ = TerminalSize::default();
                }
            },
        );
    }

    #[test]
    fn test_terminal_size_comparison() {
        let size1 = TerminalSize { cols: 80, rows: 24 };
        let size2 = TerminalSize { cols: 120, rows: 40 };

        assert_performance(
            "TerminalSize comparison 100000 times",
            Duration::from_millis(5),
            || {
                for _ in 0..100000 {
                    let _ = size1 == size2;
                    let _ = size1 != size2;
                }
            },
        );
    }
}

// ============================================================================
// Memory Allocation Performance Tests
// ============================================================================
mod memory_performance_tests {
    use super::*;

    #[test]
    fn test_grid_memory_allocation_pattern() {
        // Test that grid doesn't cause excessive allocations during normal use
        let mut grid = Grid::new(80, 24);

        assert_performance(
            "Write/scroll cycle (memory pattern)",
            Duration::from_millis(100),
            || {
                for _ in 0..1000 {
                    // Simulate typical terminal usage
                    for _ in 0..80 {
                        grid.write_char('X');
                    }
                    grid.scroll_up(1);
                }
            },
        );
    }

    #[test]
    fn test_cell_vector_growth() {
        assert_performance(
            "Cell vector growth to 100000",
            Duration::from_millis(50),
            || {
                let mut cells = Vec::new();
                for _ in 0..100000 {
                    cells.push(Cell::default());
                }
            },
        );
    }

    #[test]
    fn test_cell_vector_preallocated() {
        assert_performance(
            "Preallocated cell vector 100000",
            Duration::from_millis(20),
            || {
                let cells: Vec<Cell> = Vec::with_capacity(100000);
                let mut cells = cells;
                for _ in 0..100000 {
                    cells.push(Cell::default());
                }
            },
        );
    }

    #[test]
    fn test_string_buffer_growth() {
        let grid = Grid::new(200, 100);

        assert_performance(
            "Grid to_string_content with large grid",
            Duration::from_millis(20),
            || {
                let _ = grid.to_string_content();
            },
        );
    }
}

// ============================================================================
// Concurrent/Stress Tests
// ============================================================================
mod stress_tests {
    use super::*;

    #[test]
    fn test_rapid_grid_operations() {
        let mut grid = Grid::new(80, 24);

        assert_performance(
            "Rapid mixed operations 10000 cycles",
            Duration::from_millis(200),
            || {
                for i in 0..10000 {
                    match i % 5 {
                        0 => grid.write_char('A'),
                        1 => grid.set_cursor(i % 80, (i / 80) % 24),
                        2 => { if i % 10 == 0 { grid.scroll_up(1); } },
                        3 => { let _ = grid.get_cell(i % 80, (i / 80) % 24); },
                        4 => { if i % 100 == 0 { grid.clear(); } },
                        _ => {}
                    }
                }
            },
        );
    }

    #[test]
    fn test_parser_continuous_stream() {
        let mut parser = AnsiParser::new();
        let mut grid = Grid::new(80, 100);

        // Simulate continuous terminal output
        let chunks: Vec<Vec<u8>> = (0..100)
            .map(|i| format!("Line {} with some content\r\n\x1b[32mGreen\x1b[0m\r\n", i).into_bytes())
            .collect();

        assert_performance(
            "Process 100 stream chunks",
            Duration::from_millis(50),
            || {
                for chunk in &chunks {
                    parser.process(chunk, &mut grid);
                }
            },
        );
    }

    #[test]
    fn test_alternating_resize() {
        let mut grid = Grid::new(80, 24);

        assert_performance(
            "Alternating resize 1000 times",
            Duration::from_millis(200),
            || {
                for i in 0..1000 {
                    if i % 2 == 0 {
                        grid.resize(120, 40);
                    } else {
                        grid.resize(80, 24);
                    }
                }
            },
        );
    }

    #[test]
    fn test_full_screen_refresh() {
        let mut grid = Grid::new(200, 100);

        // Simulate full screen refresh (common in editors/TUIs)
        assert_performance(
            "Full screen refresh 100 times",
            Duration::from_millis(500),
            || {
                for _ in 0..100 {
                    grid.clear();
                    for row in 0..100 {
                        grid.set_cursor(0, row);
                        for _ in 0..200 {
                            grid.write_char('█');
                        }
                    }
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
    println!("Performance Test Summary");
    println!("========================================\n");

    // Grid throughput
    let mut grid = Grid::new(80, 24);
    let start = Instant::now();
    for _ in 0..100000 {
        grid.write_char('X');
    }
    let grid_throughput = 100000.0 / start.elapsed().as_secs_f64();
    println!("Grid write throughput: {:.0} chars/sec", grid_throughput);

    // Parser throughput
    let mut parser = AnsiParser::new();
    let mut grid = Grid::new(80, 100);
    let text = "Hello World with \x1b[31mcolor\x1b[0m!\r\n".repeat(10000);
    let bytes = text.as_bytes();
    let start = Instant::now();
    parser.process(bytes, &mut grid);
    let parser_throughput = bytes.len() as f64 / start.elapsed().as_secs_f64() / 1_000_000.0;
    println!("Parser throughput: {:.2} MB/s", parser_throughput);

    // Grid creation throughput
    let start = Instant::now();
    for _ in 0..1000 {
        let _ = Grid::new(80, 24);
    }
    let creation_rate = 1000.0 / start.elapsed().as_secs_f64();
    println!("Grid creation rate: {:.0} grids/sec", creation_rate);

    println!("\n========================================\n");
}
