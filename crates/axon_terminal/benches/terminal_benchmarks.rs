//! Criterion-based benchmarks for axon_terminal
//!
//! These benchmarks provide statistically accurate performance measurements using Criterion.
//! Following Zed editor's benchmarking approach with:
//! - Bootstrap confidence intervals
//! - Noise detection
//! - Regression detection
//! - HTML reports
//!
//! Run with: cargo bench --package axon_terminal
//! View reports in: target/criterion/report/index.html

use criterion::{
    black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput,
};

use axon_terminal::buffer::{Cell, CellFlags, Color, Grid, Row};
use axon_terminal::parser::AnsiParser;
use axon_terminal::TerminalSize;

// ============================================================================
// Grid Benchmarks
// ============================================================================

fn bench_grid_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("grid_creation");

    for (cols, rows) in [(80, 24), (120, 40), (200, 100), (500, 500)] {
        group.bench_with_input(
            BenchmarkId::new("size", format!("{}x{}", cols, rows)),
            &(cols, rows),
            |b, &(cols, rows)| {
                b.iter(|| black_box(Grid::new(cols, rows)));
            },
        );
    }

    group.finish();
}

fn bench_grid_write_char(c: &mut Criterion) {
    let mut group = c.benchmark_group("grid_write_char");

    group.throughput(Throughput::Elements(1));
    group.bench_function("single_char", |b| {
        let mut grid = Grid::new(80, 100);
        b.iter(|| {
            grid.write_char(black_box('X'));
            if grid.cursor().1 >= 90 {
                grid.set_cursor(0, 0);
            }
        });
    });

    group.throughput(Throughput::Elements(1000));
    group.bench_function("1000_chars", |b| {
        let mut grid = Grid::new(80, 100);
        b.iter(|| {
            for _ in 0..1000 {
                grid.write_char(black_box('X'));
            }
            grid.set_cursor(0, 0);
        });
    });

    group.finish();
}

fn bench_grid_scroll(c: &mut Criterion) {
    let mut group = c.benchmark_group("grid_scroll");

    group.bench_function("scroll_up_1", |b| {
        let mut grid = Grid::new(80, 24);
        // Pre-fill grid
        for _ in 0..80 * 24 {
            grid.write_char('X');
        }
        b.iter(|| {
            grid.scroll_up(black_box(1));
        });
    });

    group.bench_function("scroll_up_10", |b| {
        let mut grid = Grid::new(80, 100);
        for _ in 0..80 * 100 {
            grid.write_char('X');
        }
        b.iter(|| {
            grid.scroll_up(black_box(10));
        });
    });

    group.bench_function("scroll_down_1", |b| {
        let mut grid = Grid::new(80, 24);
        b.iter(|| {
            grid.scroll_down(black_box(1));
        });
    });

    group.finish();
}

fn bench_grid_resize(c: &mut Criterion) {
    let mut group = c.benchmark_group("grid_resize");

    group.bench_function("grow", |b| {
        b.iter_batched(
            || Grid::new(80, 24),
            |mut grid| {
                grid.resize(black_box(120), black_box(40));
                grid
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("shrink", |b| {
        b.iter_batched(
            || Grid::new(120, 40),
            |mut grid| {
                grid.resize(black_box(80), black_box(24));
                grid
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("grow_shrink_cycle", |b| {
        let mut grid = Grid::new(80, 24);
        b.iter(|| {
            grid.resize(120, 40);
            grid.resize(80, 24);
        });
    });

    group.finish();
}

fn bench_grid_clear(c: &mut Criterion) {
    let mut group = c.benchmark_group("grid_clear");

    for (cols, rows) in [(80, 24), (200, 100)] {
        group.bench_with_input(
            BenchmarkId::new("size", format!("{}x{}", cols, rows)),
            &(cols, rows),
            |b, &(cols, rows)| {
                let mut grid = Grid::new(cols, rows);
                // Fill with content
                for _ in 0..cols * rows / 2 {
                    grid.write_char('X');
                }
                b.iter(|| {
                    grid.clear();
                });
            },
        );
    }

    group.finish();
}

fn bench_grid_cell_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("grid_cell_access");

    let grid = Grid::new(80, 24);

    group.bench_function("get_cell", |b| {
        b.iter(|| {
            black_box(grid.get_cell(black_box(40), black_box(12)));
        });
    });

    group.bench_function("get_cell_sequential", |b| {
        b.iter(|| {
            for row in 0..24 {
                for col in 0..80 {
                    black_box(grid.get_cell(col, row));
                }
            }
        });
    });

    group.finish();
}

fn bench_grid_cursor(c: &mut Criterion) {
    let mut group = c.benchmark_group("grid_cursor");

    let mut grid = Grid::new(80, 24);

    group.bench_function("set_cursor", |b| {
        let mut i = 0usize;
        b.iter(|| {
            grid.set_cursor(i % 80, (i / 80) % 24);
            i = i.wrapping_add(1);
        });
    });

    group.bench_function("get_cursor", |b| {
        b.iter(|| {
            black_box(grid.cursor());
        });
    });

    group.finish();
}

fn bench_grid_to_string(c: &mut Criterion) {
    let mut group = c.benchmark_group("grid_to_string");

    for (cols, rows) in [(80, 24), (200, 100)] {
        group.throughput(Throughput::Elements((cols * rows) as u64));
        group.bench_with_input(
            BenchmarkId::new("size", format!("{}x{}", cols, rows)),
            &(cols, rows),
            |b, &(cols, rows)| {
                let mut grid = Grid::new(cols, rows);
                for _ in 0..cols * rows {
                    grid.write_char('X');
                }
                b.iter(|| {
                    black_box(grid.to_string_content());
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Row Benchmarks
// ============================================================================

fn bench_row_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("row_operations");

    group.bench_function("new_80", |b| {
        b.iter(|| black_box(Row::new(black_box(80))));
    });

    group.bench_function("new_200", |b| {
        b.iter(|| black_box(Row::new(black_box(200))));
    });

    group.bench_function("resize", |b| {
        let mut row = Row::new(80);
        let mut size = 80;
        b.iter(|| {
            size = if size == 80 { 120 } else { 80 };
            row.resize(black_box(size));
        });
    });

    group.bench_function("clear", |b| {
        let mut row = Row::new(80);
        b.iter(|| {
            row.clear();
        });
    });

    group.bench_function("clone", |b| {
        let row = Row::new(80);
        b.iter(|| black_box(row.clone()));
    });

    group.bench_function("iterate", |b| {
        let row = Row::new(200);
        b.iter(|| {
            for cell in row.iter() {
                black_box(cell.c);
            }
        });
    });

    group.finish();
}

// ============================================================================
// Cell Benchmarks
// ============================================================================

fn bench_cell_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("cell_operations");

    group.bench_function("default", |b| {
        b.iter(|| black_box(Cell::default()));
    });

    group.bench_function("new", |b| {
        b.iter(|| black_box(Cell::new(black_box('X'))));
    });

    group.bench_function("reset", |b| {
        let mut cell = Cell {
            c: 'X',
            fg: Color::Rgb(255, 128, 64),
            bg: Color::Indexed(100),
            flags: CellFlags {
                bold: true,
                italic: true,
                ..Default::default()
            },
            width: 1,
        };
        b.iter(|| {
            cell.reset();
        });
    });

    group.bench_function("clone", |b| {
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
        b.iter(|| black_box(cell.clone()));
    });

    group.bench_function("is_empty", |b| {
        let cell = Cell::default();
        b.iter(|| black_box(cell.is_empty()));
    });

    group.finish();
}

fn bench_color_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("color_operations");

    group.bench_function("named", |b| {
        let mut i = 0u8;
        b.iter(|| {
            i = i.wrapping_add(1);
            black_box(Color::Named(i % 16));
        });
    });

    group.bench_function("indexed", |b| {
        let mut i = 0u8;
        b.iter(|| {
            i = i.wrapping_add(1);
            black_box(Color::Indexed(i));
        });
    });

    group.bench_function("rgb", |b| {
        let mut i = 0u8;
        b.iter(|| {
            i = i.wrapping_add(1);
            black_box(Color::Rgb(i, i.wrapping_add(50), i.wrapping_add(100)));
        });
    });

    group.finish();
}

// ============================================================================
// ANSI Parser Benchmarks
// ============================================================================

fn bench_ansi_parser_plain_text(c: &mut Criterion) {
    let mut group = c.benchmark_group("ansi_parser_plain_text");

    for size in [1_000, 10_000, 100_000] {
        let text = "Hello, World! ".repeat(size / 14);
        let bytes = text.as_bytes();

        group.throughput(Throughput::Bytes(bytes.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("bytes", bytes.len()),
            bytes,
            |b, bytes| {
                let mut parser = AnsiParser::new();
                let mut grid = Grid::new(80, 100);
                b.iter(|| {
                    parser.process(black_box(bytes), &mut grid);
                    grid.set_cursor(0, 0);
                });
            },
        );
    }

    group.finish();
}

fn bench_ansi_parser_colored_text(c: &mut Criterion) {
    let mut group = c.benchmark_group("ansi_parser_colored_text");

    // Generate colored text with ANSI sequences
    let mut colored_text = String::new();
    for i in 0..1000 {
        colored_text.push_str(&format!("\x1b[{}mText{}\x1b[0m ", 31 + (i % 7), i));
    }
    let bytes = colored_text.as_bytes();

    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("1000_colored_words", |b| {
        let mut parser = AnsiParser::new();
        let mut grid = Grid::new(80, 100);
        b.iter(|| {
            parser.process(black_box(bytes), &mut grid);
            grid.set_cursor(0, 0);
        });
    });

    group.finish();
}

fn bench_ansi_parser_sgr(c: &mut Criterion) {
    let mut group = c.benchmark_group("ansi_parser_sgr");

    // Generate SGR sequences
    let mut sgr_text = String::new();
    for _ in 0..500 {
        sgr_text.push_str("\x1b[1;4;31;42mBold underline red on green\x1b[0m ");
        sgr_text.push_str("\x1b[38;5;200;48;5;50m256 color\x1b[0m ");
        sgr_text.push_str("\x1b[38;2;255;128;64;48;2;0;64;128mTrue color\x1b[0m\n");
    }
    let bytes = sgr_text.as_bytes();

    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("complex_sgr", |b| {
        let mut parser = AnsiParser::new();
        let mut grid = Grid::new(80, 50);
        b.iter(|| {
            parser.process(black_box(bytes), &mut grid);
            grid.set_cursor(0, 0);
        });
    });

    group.finish();
}

fn bench_ansi_parser_cursor(c: &mut Criterion) {
    let mut group = c.benchmark_group("ansi_parser_cursor");

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

    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("cursor_movements", |b| {
        let mut parser = AnsiParser::new();
        let mut grid = Grid::new(80, 24);
        b.iter(|| {
            parser.process(black_box(bytes), &mut grid);
        });
    });

    group.finish();
}

fn bench_ansi_parser_unicode(c: &mut Criterion) {
    let mut group = c.benchmark_group("ansi_parser_unicode");

    // Generate Unicode content
    let mut unicode_text = String::new();
    for _ in 0..200 {
        unicode_text.push_str("Hello 你好 こんにちは 안녕하세요 🎉🚀💻\r\n");
    }
    let bytes = unicode_text.as_bytes();

    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("multilingual", |b| {
        let mut parser = AnsiParser::new();
        let mut grid = Grid::new(80, 50);
        b.iter(|| {
            parser.process(black_box(bytes), &mut grid);
            grid.set_cursor(0, 0);
        });
    });

    group.finish();
}

fn bench_ansi_parser_mixed(c: &mut Criterion) {
    let mut group = c.benchmark_group("ansi_parser_mixed");

    // Simulate real terminal output (like `ls -la --color`)
    let mut mixed_text = String::new();
    for i in 0..500 {
        mixed_text.push_str(&format!(
            "\x1b[34mdrwxr-xr-x\x1b[0m  \x1b[33m{}\x1b[0m user group {:>8} Jan {:>2} 12:00 \x1b[32mfile_{}.txt\x1b[0m\r\n",
            i % 10, i * 1024, i % 31 + 1, i
        ));
    }
    let bytes = mixed_text.as_bytes();

    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("ls_output_simulation", |b| {
        let mut parser = AnsiParser::new();
        let mut grid = Grid::new(120, 50);
        b.iter(|| {
            parser.process(black_box(bytes), &mut grid);
            grid.set_cursor(0, 0);
        });
    });

    group.finish();
}

// ============================================================================
// TerminalSize Benchmarks
// ============================================================================

fn bench_terminal_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("terminal_size");

    group.bench_function("new", |b| {
        let mut i = 0u16;
        b.iter(|| {
            i = i.wrapping_add(1);
            black_box(TerminalSize {
                cols: 80 + (i % 100),
                rows: 24 + (i % 50),
            });
        });
    });

    group.bench_function("default", |b| {
        b.iter(|| black_box(TerminalSize::default()));
    });

    group.bench_function("comparison", |b| {
        let size1 = TerminalSize { cols: 80, rows: 24 };
        let size2 = TerminalSize { cols: 120, rows: 40 };
        b.iter(|| {
            black_box(size1 == size2);
            black_box(size1 != size2);
        });
    });

    group.finish();
}

// ============================================================================
// Stress Test Benchmarks
// ============================================================================

fn bench_stress_tests(c: &mut Criterion) {
    let mut group = c.benchmark_group("stress_tests");
    group.sample_size(50); // Fewer samples for expensive tests

    group.bench_function("full_screen_refresh", |b| {
        let mut grid = Grid::new(200, 100);
        b.iter(|| {
            grid.clear();
            for row in 0..100 {
                grid.set_cursor(0, row);
                for _ in 0..200 {
                    grid.write_char('█');
                }
            }
        });
    });

    group.bench_function("rapid_resize", |b| {
        let mut grid = Grid::new(80, 24);
        b.iter(|| {
            for _ in 0..10 {
                grid.resize(120, 40);
                grid.resize(80, 24);
            }
        });
    });

    group.bench_function("mixed_operations", |b| {
        let mut grid = Grid::new(80, 24);
        b.iter(|| {
            for i in 0..100 {
                match i % 5 {
                    0 => grid.write_char('A'),
                    1 => grid.set_cursor(i % 80, (i / 80) % 24),
                    2 => grid.scroll_up(1),
                    3 => {
                        let _ = grid.get_cell(i % 80, (i / 80) % 24);
                    }
                    4 => grid.clear(),
                    _ => {}
                }
            }
        });
    });

    group.finish();
}

// ============================================================================
// Criterion Groups
// ============================================================================

criterion_group!(
    grid_benches,
    bench_grid_creation,
    bench_grid_write_char,
    bench_grid_scroll,
    bench_grid_resize,
    bench_grid_clear,
    bench_grid_cell_access,
    bench_grid_cursor,
    bench_grid_to_string,
);

criterion_group!(row_benches, bench_row_operations,);

criterion_group!(cell_benches, bench_cell_operations, bench_color_operations,);

criterion_group!(
    parser_benches,
    bench_ansi_parser_plain_text,
    bench_ansi_parser_colored_text,
    bench_ansi_parser_sgr,
    bench_ansi_parser_cursor,
    bench_ansi_parser_unicode,
    bench_ansi_parser_mixed,
);

criterion_group!(misc_benches, bench_terminal_size, bench_stress_tests,);

criterion_main!(
    grid_benches,
    row_benches,
    cell_benches,
    parser_benches,
    misc_benches
);
