// benches/window_performance.rs
// WASMA Window Manager Performance Benchmarks
// January 16, 2026

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use wasma_client::{
    WindowHandler, WindowGeometry, WindowState,
    ResourceMode,
};

fn benchmark_window_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("window_creation");
    
    for mode in [ResourceMode::Auto, ResourceMode::Manual] {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{:?}", mode)),
            &mode,
            |b, &mode| {
                let handler = WindowHandler::new(mode);
                let geometry = WindowGeometry {
                    x: 100,
                    y: 100,
                    width: 800,
                    height: 600,
                };
                
                b.iter(|| {
                    let result = handler.create_window(
                        black_box("Benchmark Window".to_string()),
                        black_box("bench.app".to_string()),
                        black_box(geometry),
                        None,
                        mode,
                    );
                    black_box(result)
                });
            }
        );
    }
    
    group.finish();
}

fn benchmark_window_focus(c: &mut Criterion) {
    let handler = WindowHandler::new(ResourceMode::Auto);
    let geometry = WindowGeometry { x: 0, y: 0, width: 800, height: 600 };
    
    // Create 10 windows
    let window_ids: Vec<u64> = (0..10)
        .map(|i| {
            handler.create_window(
                format!("Window {}", i),
                format!("app{}.test", i),
                geometry,
                None,
                ResourceMode::Auto,
            ).unwrap()
        })
        .collect();
    
    c.bench_function("window_focus", |b| {
        let mut idx = 0;
        b.iter(|| {
            let id = window_ids[idx % window_ids.len()];
            handler.focus_window(black_box(id)).ok();
            idx += 1;
        });
    });
}

fn benchmark_window_state_changes(c: &mut Criterion) {
    let handler = WindowHandler::new(ResourceMode::Auto);
    let geometry = WindowGeometry { x: 0, y: 0, width: 800, height: 600 };
    
    let window_id = handler.create_window(
        "State Test".to_string(),
        "state.test".to_string(),
        geometry,
        None,
        ResourceMode::Auto,
    ).unwrap();
    
    let states = [
        WindowState::Normal,
        WindowState::Minimized,
        WindowState::Maximized,
        WindowState::Fullscreen,
    ];
    
    c.bench_function("window_state_changes", |b| {
        let mut idx = 0;
        b.iter(|| {
            let state = states[idx % states.len()].clone();
            handler.set_window_state(black_box(window_id), black_box(state)).ok();
            idx += 1;
        });
    });
}

fn benchmark_list_windows(c: &mut Criterion) {
    let mut group = c.benchmark_group("list_windows");
    
    for window_count in [10, 50, 100, 500] {
        group.bench_with_input(
            BenchmarkId::from_parameter(window_count),
            &window_count,
            |b, &count| {
                let handler = WindowHandler::new(ResourceMode::Auto);
                let geometry = WindowGeometry { x: 0, y: 0, width: 800, height: 600 };
                
                // Create windows
                for i in 0..count {
                    handler.create_window(
                        format!("Window {}", i),
                        format!("app{}.test", i),
                        geometry,
                        None,
                        ResourceMode::Auto,
                    ).ok();
                }
                
                b.iter(|| {
                    let windows = handler.list_windows();
                    black_box(windows)
                });
            }
        );
    }
    
    group.finish();
}

fn benchmark_resource_cycle(c: &mut Criterion) {
    let handler = WindowHandler::new(ResourceMode::Auto);
    let geometry = WindowGeometry { x: 0, y: 0, width: 800, height: 600 };
    
    // Create 50 windows
    for i in 0..50 {
        handler.create_window(
            format!("Window {}", i),
            format!("app{}.test", i),
            geometry,
            None,
            ResourceMode::Auto,
        ).ok();
    }
    
    c.bench_function("resource_cycle_50_windows", |b| {
        b.iter(|| {
            handler.run_resource_cycle();
        });
    });
}

fn benchmark_window_close(c: &mut Criterion) {
    c.bench_function("window_close", |b| {
        b.iter_batched(
            || {
                // Setup: Create window
                let handler = WindowHandler::new(ResourceMode::Auto);
                let geometry = WindowGeometry { x: 0, y: 0, width: 800, height: 600 };
                let id = handler.create_window(
                    "Close Test".to_string(),
                    "close.test".to_string(),
                    geometry,
                    None,
                    ResourceMode::Auto,
                ).unwrap();
                (handler, id)
            },
            |(handler, id)| {
                // Benchmark: Close window
                handler.close_window(black_box(id)).ok();
            },
            criterion::BatchSize::SmallInput
        );
    });
}

fn benchmark_geometry_updates(c: &mut Criterion) {
    let handler = WindowHandler::new(ResourceMode::Auto);
    let geometry = WindowGeometry { x: 0, y: 0, width: 800, height: 600 };
    
    let window_id = handler.create_window(
        "Geometry Test".to_string(),
        "geo.test".to_string(),
        geometry,
        None,
        ResourceMode::Auto,
    ).unwrap();
    
    c.bench_function("geometry_update", |b| {
        let mut x = 0;
        b.iter(|| {
            let new_geo = WindowGeometry {
                x: x % 1920,
                y: x % 1080,
                width: 800,
                height: 600,
            };
            handler.set_geometry(black_box(window_id), black_box(new_geo)).ok();
            x += 10;
        });
    });
}

fn benchmark_parent_child_operations(c: &mut Criterion) {
    let handler = WindowHandler::new(ResourceMode::Auto);
    let geometry = WindowGeometry { x: 0, y: 0, width: 800, height: 600 };
    
    let parent_id = handler.create_window(
        "Parent".to_string(),
        "parent.test".to_string(),
        geometry,
        None,
        ResourceMode::Auto,
    ).unwrap();
    
    c.bench_function("set_parent", |b| {
        b.iter_batched(
            || {
                // Setup: Create child window
                handler.create_window(
                    "Child".to_string(),
                    "child.test".to_string(),
                    geometry,
                    None,
                    ResourceMode::Auto,
                ).unwrap()
            },
            |child_id| {
                // Benchmark: Set parent
                handler.set_parent(black_box(child_id), black_box(parent_id)).ok();
            },
            criterion::BatchSize::SmallInput
        );
    });
}

criterion_group!(
    benches,
    benchmark_window_creation,
    benchmark_window_focus,
    benchmark_window_state_changes,
    benchmark_list_windows,
    benchmark_resource_cycle,
    benchmark_window_close,
    benchmark_geometry_updates,
    benchmark_parent_child_operations,
);
criterion_main!(benches);
