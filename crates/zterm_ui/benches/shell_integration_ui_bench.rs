//! Benchmarks for shell integration UI module

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use gpui::Point;
use zterm_terminal::shell_integration::CommandState;
use zterm_ui::shell_integration::{
    build_context_menu, command_state_to_visual, GutterConfig, GutterMark, GutterVisual,
    HighlightConfig, HighlightRect, HighlightRegion, HighlightType, HoverState, MenuContext,
    MouseConfig, MouseHandler,
};

fn bench_screen_to_line(c: &mut Criterion) {
    let mut group = c.benchmark_group("screen_to_line");

    let handler = MouseHandler::new(MouseConfig::default());

    group.bench_function("basic", |b| {
        let mut y = 0.0f32;
        b.iter(|| {
            let pos = Point::new(100.0, y);
            let result = black_box(handler.screen_to_line(pos));
            y = (y + 1.0) % 1000.0;
            result
        });
    });

    group.bench_function("with_scroll", |b| {
        let config = MouseConfig {
            scroll_offset: 500.0,
            first_visible_line: 100,
            ..Default::default()
        };
        let handler = MouseHandler::new(config);

        let mut y = 0.0f32;
        b.iter(|| {
            let pos = Point::new(100.0, y);
            let result = black_box(handler.screen_to_line(pos));
            y = (y + 1.0) % 1000.0;
            result
        });
    });

    group.finish();
}

fn bench_screen_to_cell(c: &mut Criterion) {
    let handler = MouseHandler::new(MouseConfig::default());

    c.bench_function("screen_to_cell", |b| {
        let mut x = 20.0f32;
        let mut y = 0.0f32;
        b.iter(|| {
            let pos = Point::new(x, y);
            let result = black_box(handler.screen_to_cell(pos));
            x = 20.0 + ((x + 1.0) % 800.0);
            y = (y + 1.0) % 1000.0;
            result
        });
    });
}

fn bench_hover_state_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("hover_state_update");

    let handler = MouseHandler::new(MouseConfig::default());

    // No zones
    group.bench_function("no_zones", |b| {
        let mut y = 0.0f32;
        b.iter(|| {
            let pos = Point::new(100.0, y);
            let state = black_box(HoverState::from_position(&handler, pos, |_| None));
            y = (y + 1.0) % 1000.0;
            state
        });
    });

    // With zones
    group.bench_function("with_zones", |b| {
        let mut y = 0.0f32;
        b.iter(|| {
            let pos = Point::new(100.0, y);
            let state = black_box(HoverState::from_position(&handler, pos, |line| {
                // Simulate zone lookup
                let zone_start = (line / 10) * 10;
                let is_output = line > zone_start;
                Some((zone_start, is_output))
            }));
            y = (y + 1.0) % 1000.0;
            state
        });
    });

    // In gutter
    group.bench_function("in_gutter", |b| {
        let mut y = 0.0f32;
        b.iter(|| {
            let pos = Point::new(10.0, y);
            let state = black_box(HoverState::from_position(&handler, pos, |_| None));
            y = (y + 1.0) % 1000.0;
            state
        });
    });

    group.finish();
}

fn bench_gutter_state_to_visual(c: &mut Criterion) {
    let mut group = c.benchmark_group("gutter_state_to_visual");

    let test_cases = [
        ("prompt", true, false, None),
        ("running", true, true, None),
        ("success", true, false, Some(0)),
        ("failure", true, false, Some(1)),
        ("continuation", false, false, None),
    ];

    for (name, is_prompt, is_running, exit_code) in test_cases {
        group.bench_function(name, |b| {
            b.iter(|| {
                black_box(command_state_to_visual(is_prompt, is_running, exit_code))
            });
        });
    }

    group.finish();
}

fn bench_gutter_color_lookup(c: &mut Criterion) {
    let config = GutterConfig::default();

    let visuals = [
        GutterVisual::None,
        GutterVisual::Prompt,
        GutterVisual::Running,
        GutterVisual::Success,
        GutterVisual::Failure,
        GutterVisual::Continuation,
    ];

    c.bench_function("gutter_color_lookup", |b| {
        let mut i = 0;
        b.iter(|| {
            let visual = visuals[i % visuals.len()];
            let color = black_box(config.color_for_visual(visual));
            i += 1;
            color
        });
    });
}

fn bench_gutter_mark_creation(c: &mut Criterion) {
    c.bench_function("gutter_mark_creation", |b| {
        let mut y = 0.0f32;
        b.iter(|| {
            let mark = GutterMark::new(GutterVisual::Success, y, 20.0).with_hover(true);
            y = (y + 20.0) % 10000.0;
            black_box(mark)
        });
    });
}

fn bench_highlight_region_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("highlight_region");

    group.bench_function("contains_line", |b| {
        let region = HighlightRegion::new(100, 200, HighlightType::Hover);
        let mut line = 0usize;
        b.iter(|| {
            let contains = black_box(region.contains_line(line));
            line = (line + 1) % 300;
            contains
        });
    });

    group.bench_function("overlaps", |b| {
        let region1 = HighlightRegion::new(100, 200, HighlightType::Hover);
        let mut start = 0usize;
        b.iter(|| {
            let region2 = HighlightRegion::new(start, start + 50, HighlightType::Selected);
            let overlaps = black_box(region1.overlaps(&region2));
            start = (start + 10) % 300;
            overlaps
        });
    });

    group.bench_function("merge", |b| {
        let region1 = HighlightRegion::new(100, 150, HighlightType::Hover);
        b.iter(|| {
            let region2 = HighlightRegion::new(140, 200, HighlightType::Hover);
            black_box(region1.merge(&region2))
        });
    });

    group.finish();
}

fn bench_highlight_rect_computation(c: &mut Criterion) {
    let config = HighlightConfig::default();
    let region = HighlightRegion::new(50, 100, HighlightType::Hover);

    c.bench_function("highlight_rect_from_region", |b| {
        let mut first_visible = 0usize;
        b.iter(|| {
            let rect = black_box(HighlightRect::from_region(
                &region,
                &config,
                20.0,         // line_height
                first_visible,
                0.0,          // x
                800.0,        // width
                0.0,          // scroll_offset
            ));
            first_visible = (first_visible + 1) % 60;
            rect
        });
    });
}

fn bench_build_context_menu(c: &mut Criterion) {
    let mut group = c.benchmark_group("build_context_menu");

    // Minimal context
    group.bench_function("minimal", |b| {
        let context = MenuContext::new();
        b.iter(|| black_box(build_context_menu(&context)));
    });

    // Full context
    group.bench_function("full", |b| {
        let context = MenuContext::new()
            .with_command("ls -la /home/user/documents")
            .with_output(true)
            .with_state(CommandState::CommandFinished(0))
            .with_working_directory("/home/user")
            .with_ai(true);
        b.iter(|| black_box(build_context_menu(&context)));
    });

    // Error context (with debug option)
    group.bench_function("error_with_ai", |b| {
        let context = MenuContext::new()
            .with_command("make build")
            .with_output(true)
            .with_state(CommandState::CommandFinished(1))
            .with_ai(true);
        b.iter(|| black_box(build_context_menu(&context)));
    });

    group.finish();
}

fn bench_menu_context_builder(c: &mut Criterion) {
    c.bench_function("menu_context_builder", |b| {
        b.iter(|| {
            let context = MenuContext::new()
                .with_command("echo hello")
                .with_output(true)
                .with_state(CommandState::CommandFinished(0))
                .with_working_directory("/tmp")
                .with_ai(true);
            black_box(context)
        });
    });
}

criterion_group!(
    benches,
    bench_screen_to_line,
    bench_screen_to_cell,
    bench_hover_state_update,
    bench_gutter_state_to_visual,
    bench_gutter_color_lookup,
    bench_gutter_mark_creation,
    bench_highlight_region_operations,
    bench_highlight_rect_computation,
    bench_build_context_menu,
    bench_menu_context_builder,
);

criterion_main!(benches);
