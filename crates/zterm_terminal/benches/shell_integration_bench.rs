//! Benchmarks for shell integration module

#![allow(clippy::let_and_return)]

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use zterm_terminal::shell_integration::{
    CommandState, CommandZone, OscScanner, ShellIntegrationHandler, TextBuffer, TextExtractor,
    ZoneId, ZoneManager,
};

/// Mock buffer for benchmarks
struct MockBuffer {
    lines: Vec<String>,
}

impl MockBuffer {
    fn new(line_count: usize) -> Self {
        let lines: Vec<String> = (0..line_count)
            .map(|i| format!("Line {} content here with some text", i))
            .collect();
        Self { lines }
    }
}

impl TextBuffer for MockBuffer {
    fn line_text(&self, line: usize) -> Option<String> {
        self.lines.get(line).cloned()
    }

    fn total_lines(&self) -> usize {
        self.lines.len()
    }
}

fn setup_zone_manager(zone_count: usize) -> ZoneManager {
    let mut manager = ZoneManager::new();
    let lines_per_zone = 10;

    for i in 0..zone_count {
        let start = i * lines_per_zone;
        manager.start_zone(CommandState::PromptStart, start);
        manager.transition_state(CommandState::CommandStart, start + 1);
        manager.transition_state(CommandState::CommandExecuting, start + 1);
        manager.finish_zone(start + lines_per_zone, if i % 10 == 0 { 1 } else { 0 });
    }

    manager
}

fn bench_zone_at_line(c: &mut Criterion) {
    let mut group = c.benchmark_group("zone_at_line");

    for zone_count in [10, 100, 1000, 10000] {
        let manager = setup_zone_manager(zone_count);
        let total_lines = zone_count * 10;

        group.bench_with_input(
            BenchmarkId::from_parameter(zone_count),
            &(manager, total_lines),
            |b, (manager, total_lines)| {
                let mut line = 0usize;
                b.iter(|| {
                    let result = black_box(manager.zone_at_line(line % total_lines));
                    line = line.wrapping_add(1);
                    result
                });
            },
        );
    }

    group.finish();
}

fn bench_previous_prompt(c: &mut Criterion) {
    let mut group = c.benchmark_group("previous_prompt");

    for zone_count in [10, 100, 1000] {
        let manager = setup_zone_manager(zone_count);
        let total_lines = zone_count * 10;

        group.bench_with_input(
            BenchmarkId::from_parameter(zone_count),
            &(manager, total_lines),
            |b, (manager, total_lines)| {
                let mut line = total_lines - 1;
                b.iter(|| {
                    let result = black_box(manager.previous_prompt(line));
                    line = if line > 0 { line - 1 } else { *total_lines - 1 };
                    result
                });
            },
        );
    }

    group.finish();
}

fn bench_next_prompt(c: &mut Criterion) {
    let mut group = c.benchmark_group("next_prompt");

    for zone_count in [10, 100, 1000] {
        let manager = setup_zone_manager(zone_count);
        let total_lines = zone_count * 10;

        group.bench_with_input(
            BenchmarkId::from_parameter(zone_count),
            &(manager, total_lines),
            |b, (manager, total_lines)| {
                let mut line = 0usize;
                b.iter(|| {
                    let result = black_box(manager.next_prompt(line));
                    line = (line + 1) % total_lines;
                    result
                });
            },
        );
    }

    group.finish();
}

fn bench_osc_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("osc_parsing");

    // OSC 133 A
    group.bench_function("osc_133_a", |b| {
        let mut handler = ShellIntegrationHandler::new();
        let osc = b"133;A";
        b.iter(|| {
            black_box(handler.handle_osc(osc));
        });
    });

    // OSC 133 B
    group.bench_function("osc_133_b", |b| {
        let mut handler = ShellIntegrationHandler::new();
        handler.handle_osc(b"133;A");
        let osc = b"133;B";
        b.iter(|| {
            black_box(handler.handle_osc(osc));
        });
    });

    // OSC 133 C
    group.bench_function("osc_133_c", |b| {
        let mut handler = ShellIntegrationHandler::new();
        handler.handle_osc(b"133;A");
        handler.handle_osc(b"133;B");
        let osc = b"133;C";
        b.iter(|| {
            black_box(handler.handle_osc(osc));
        });
    });

    // OSC 133 D
    group.bench_function("osc_133_d", |b| {
        let mut handler = ShellIntegrationHandler::new();
        handler.handle_osc(b"133;A");
        handler.handle_osc(b"133;C");
        let osc = b"133;D;0";
        b.iter(|| {
            black_box(handler.handle_osc(osc));
        });
    });

    // OSC 633 E (command capture)
    group.bench_function("osc_633_e", |b| {
        let mut handler = ShellIntegrationHandler::new();
        handler.handle_osc(b"633;A");
        handler.handle_osc(b"633;B");
        let osc = b"633;E;ls%20-la%20/home/user/documents";
        b.iter(|| {
            black_box(handler.handle_osc(osc));
        });
    });

    // Full lifecycle
    group.bench_function("full_lifecycle", |b| {
        let mut handler = ShellIntegrationHandler::new();
        b.iter(|| {
            handler.handle_osc(b"133;A");
            handler.handle_osc(b"133;B");
            handler.handle_osc(b"633;E;echo%20hello");
            handler.handle_osc(b"133;C");
            handler.handle_osc(b"133;D;0");
            black_box(handler.take_events())
        });
    });

    group.finish();
}

fn bench_mixed_output_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed_output_parsing");

    // Simulate parsing a mix of regular output and OSC sequences
    let create_mixed_data = |size: usize| -> Vec<Vec<u8>> {
        let mut data = Vec::new();

        // Start with OSC 133 A
        data.push(b"133;A".to_vec());

        // Add some "normal lines" (just dummy data to skip)
        for i in 0..size {
            if i % 100 == 50 {
                // Occasional OSC sequence
                data.push(format!("633;E;command{}", i).into_bytes());
            }
        }

        // End with OSC 133 D
        data.push(b"133;D;0".to_vec());

        data
    };

    for size in [100, 1000, 10000] {
        let data = create_mixed_data(size);

        group.bench_with_input(BenchmarkId::new("lines", size), &data, |b, data| {
            let mut handler = ShellIntegrationHandler::new();
            b.iter(|| {
                for osc in data {
                    handler.handle_osc(osc);
                }
                black_box(handler.take_events())
            });
        });
    }

    group.finish();
}

/// Helper to create OSC sequence with BEL terminator
fn make_osc_bel(content: &str) -> Vec<u8> {
    let mut data = vec![0x1b, b']'];
    data.extend_from_slice(content.as_bytes());
    data.push(0x07);
    data
}

/// Benchmark OscScanner (PTY-level scanning)
fn bench_osc_scanner(c: &mut Criterion) {
    let mut group = c.benchmark_group("osc_scanner");

    // Simple sequences
    group.bench_function("scan_133_a", |b| {
        let mut scanner = OscScanner::new();
        let data = make_osc_bel("133;A");
        b.iter(|| {
            let result = black_box(scanner.scan(&data));
            result
        });
    });

    group.bench_function("scan_133_d_with_code", |b| {
        let mut scanner = OscScanner::new();
        let data = make_osc_bel("133;D;127");
        b.iter(|| {
            let result = black_box(scanner.scan(&data));
            result
        });
    });

    group.bench_function("scan_633_e_command", |b| {
        let mut scanner = OscScanner::new();
        let data = make_osc_bel("633;E;ls%20-la%20/home/user/documents");
        b.iter(|| {
            let result = black_box(scanner.scan(&data));
            result
        });
    });

    group.bench_function("scan_osc7_path", |b| {
        let mut scanner = OscScanner::new();
        let data = make_osc_bel("7;file://localhost/home/user/project");
        b.iter(|| {
            let result = black_box(scanner.scan(&data));
            result
        });
    });

    // Full lifecycle in single scan
    group.bench_function("scan_full_lifecycle", |b| {
        let mut scanner = OscScanner::new();
        let mut data = Vec::new();
        data.extend(make_osc_bel("133;A"));
        data.extend(make_osc_bel("133;B"));
        data.extend(make_osc_bel("633;E;echo%20hello"));
        data.extend(make_osc_bel("133;C"));
        data.extend(make_osc_bel("133;D;0"));

        b.iter(|| {
            let result = black_box(scanner.scan(&data));
            result
        });
    });

    group.finish();
}

/// Benchmark scanning mixed PTY output (realistic scenario)
fn bench_osc_scanner_mixed_output(c: &mut Criterion) {
    let mut group = c.benchmark_group("osc_scanner_mixed");

    let create_pty_output = |line_count: usize| -> Vec<u8> {
        let mut data = Vec::new();

        // Add prompt with OSC sequences
        data.extend(make_osc_bel("133;A"));
        data.extend(b"user@host:~$ ");
        data.extend(make_osc_bel("133;B"));

        // Add command
        data.extend(make_osc_bel("633;E;ls%20-la"));
        data.extend(b"ls -la\r\n");
        data.extend(make_osc_bel("133;C"));

        // Add output lines
        for i in 0..line_count {
            data.extend(
                format!("drwxr-xr-x  2 user user 4096 Jan  1 00:00 dir{}\r\n", i).as_bytes(),
            );
        }

        // Command finished
        data.extend(make_osc_bel("133;D;0"));

        data
    };

    for line_count in [10, 100, 1000] {
        let data = create_pty_output(line_count);
        let data_size = data.len();

        group.bench_with_input(
            BenchmarkId::new("lines", format!("{}_{}kb", line_count, data_size / 1024)),
            &data,
            |b, data| {
                let mut scanner = OscScanner::new();
                b.iter(|| {
                    let result = black_box(scanner.scan(data));
                    result
                });
            },
        );
    }

    // Large output (64KB chunk like in real PTY)
    let large_data = create_pty_output(1000);
    group.bench_function("64kb_chunk", |b| {
        let mut scanner = OscScanner::new();
        let chunk: Vec<u8> = large_data.iter().cycle().take(65536).cloned().collect();
        b.iter(|| {
            let result = black_box(scanner.scan(&chunk));
            result
        });
    });

    group.finish();
}

/// Benchmark no-OSC scanning (baseline performance)
fn bench_osc_scanner_no_osc(c: &mut Criterion) {
    let mut group = c.benchmark_group("osc_scanner_no_osc");

    // Create data with no OSC sequences (worst case for scanner - must check every byte)
    for size_kb in [1, 16, 64] {
        let data: Vec<u8> = (0..size_kb * 1024)
            .map(|i| {
                if i % 80 == 79 {
                    b'\n'
                } else {
                    b'a' + (i % 26) as u8
                }
            })
            .collect();

        group.bench_with_input(BenchmarkId::new("kb", size_kb), &data, |b, data| {
            let mut scanner = OscScanner::new();
            b.iter(|| {
                let result = black_box(scanner.scan(data));
                result
            });
        });
    }

    group.finish();
}

fn bench_extract_lines(c: &mut Criterion) {
    let mut group = c.benchmark_group("extract_lines");

    for line_count in [10, 100, 500] {
        let buffer = MockBuffer::new(line_count);

        group.bench_with_input(
            BenchmarkId::from_parameter(line_count),
            &buffer,
            |b, buffer| {
                b.iter(|| black_box(TextExtractor::extract_lines(buffer, 0, Some(line_count))));
            },
        );
    }

    group.finish();
}

fn bench_extract_zone_text(c: &mut Criterion) {
    let mut group = c.benchmark_group("extract_zone_text");

    for line_count in [10, 100, 500] {
        let buffer = MockBuffer::new(line_count);
        let mut zone = CommandZone::new(ZoneId::new(0), CommandState::PromptStart, 0);
        zone.end_line = Some(line_count);

        group.bench_with_input(
            BenchmarkId::from_parameter(line_count),
            &(buffer, zone),
            |b, (buffer, zone)| {
                b.iter(|| black_box(TextExtractor::extract_zone_text(buffer, zone)));
            },
        );
    }

    group.finish();
}

fn bench_context_summary(c: &mut Criterion) {
    let mut group = c.benchmark_group("context_summary");

    for line_count in [10, 100, 500] {
        let buffer = MockBuffer::new(line_count);
        let mut zone = CommandZone::new(ZoneId::new(0), CommandState::PromptStart, 0);
        zone.end_line = Some(line_count);
        zone.command = Some("ls -la".to_string());
        zone.working_directory = Some("/home/user".to_string());
        zone.state = CommandState::CommandFinished(0);

        group.bench_with_input(
            BenchmarkId::from_parameter(line_count),
            &(buffer, zone),
            |b, (buffer, zone)| {
                b.iter(|| black_box(TextExtractor::get_context_summary(buffer, zone, 50)));
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_zone_at_line,
    bench_previous_prompt,
    bench_next_prompt,
    bench_osc_parsing,
    bench_mixed_output_parsing,
    bench_osc_scanner,
    bench_osc_scanner_mixed_output,
    bench_osc_scanner_no_osc,
    bench_extract_lines,
    bench_extract_zone_text,
    bench_context_summary,
);

criterion_main!(benches);
