//! Performance benchmarks for title_bar component
//!
//! Run with: cargo bench -p zterm_ui

#![allow(clippy::redundant_closure)]
#![allow(clippy::clone_on_copy)]

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use zterm_ui::TabInfo;

fn create_tab_info(id: usize) -> TabInfo {
    TabInfo::new(
        id,
        format!("Terminal {}", id),
        id == 0,
        "bash".to_string(),
        format!("/home/user/projects/project{}/src/deep/nested", id),
    )
}

fn benchmark_tab_info_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("TabInfo Creation");

    for count in [1, 10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("create_tabs", count),
            count,
            |b, &count| {
                b.iter(|| {
                    let tabs: Vec<TabInfo> = (0..count).map(|i| create_tab_info(i)).collect();
                    black_box(tabs)
                });
            },
        );
    }

    group.finish();
}

fn benchmark_display_directory(c: &mut Criterion) {
    let mut group = c.benchmark_group("display_directory");

    // Test with various path types
    let paths = vec![
        (
            "home",
            dirs::home_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "/home/user".to_string()),
        ),
        (
            "home_subdir",
            dirs::home_dir()
                .map(|p| p.join("projects").to_string_lossy().to_string())
                .unwrap_or_else(|| "/home/user/projects".to_string()),
        ),
        (
            "deep_path",
            "/home/user/projects/myapp/src/components/ui/buttons".to_string(),
        ),
        ("root", "/".to_string()),
        ("short", "/tmp".to_string()),
    ];

    for (name, path) in paths {
        let tab = TabInfo::new(0, "Test".to_string(), true, "bash".to_string(), path);

        group.bench_with_input(
            BenchmarkId::new("display_directory", name),
            &tab,
            |b, tab| {
                b.iter(|| black_box(tab.display_directory()));
            },
        );
    }

    group.finish();
}

fn benchmark_tabs_clone(c: &mut Criterion) {
    let mut group = c.benchmark_group("Tabs Clone");

    for count in [1, 10, 50, 100, 500].iter() {
        let tabs: Vec<TabInfo> = (0..*count).map(|i| create_tab_info(i)).collect();

        group.bench_with_input(BenchmarkId::new("clone_tabs", count), &tabs, |b, tabs| {
            b.iter(|| black_box(tabs.clone()));
        });
    }

    group.finish();
}

fn benchmark_element_id_creation(c: &mut Criterion) {
    use gpui::ElementId;

    let mut group = c.benchmark_group("ElementId Creation");

    // Current implementation: format string
    group.bench_function("format_string", |b| {
        b.iter(|| {
            for i in 0..100 {
                let id = ElementId::Name(format!("tab-{}", i).into());
                black_box(id);
            }
        });
    });

    // Optimized: integer id
    group.bench_function("integer", |b| {
        b.iter(|| {
            for i in 0..100u64 {
                let id = ElementId::Integer(i);
                black_box(id);
            }
        });
    });

    // Optimized: NamedInteger
    group.bench_function("named_integer", |b| {
        b.iter(|| {
            for i in 0..100u64 {
                let id = ElementId::NamedInteger("tab".into(), i);
                black_box(id);
            }
        });
    });

    group.finish();
}

fn benchmark_scroll_to_tab(c: &mut Criterion) {
    use zterm_ui::TitleBar;

    let mut group = c.benchmark_group("Scroll To Tab");

    for count in [10, 50, 100, 500].iter() {
        let tabs: Vec<TabInfo> = (0..*count).map(|i| create_tab_info(i)).collect();
        let title_bar = TitleBar::new().tabs(tabs);

        group.bench_with_input(
            BenchmarkId::new("scroll_to_middle", count),
            &(*count, &title_bar),
            |b, (count, title_bar)| {
                b.iter(|| {
                    title_bar.scroll_to_tab(count / 2);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_tab_info_creation,
    benchmark_display_directory,
    benchmark_tabs_clone,
    benchmark_element_id_creation,
    benchmark_scroll_to_tab,
);
criterion_main!(benches);
