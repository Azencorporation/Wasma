// benches/resource_allocation.rs
// WASMA Resource Allocation & WBackend Performance Benchmarks
// January 16, 2026

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use wasma_client::{
    WindowHandler, WindowGeometry, ResourceLimits, ResourceMode, ExecutionMode,
};
use wbackend::{Assignment, WBackend};
use std::time::Duration;

fn benchmark_assignment_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("assignment_creation");
    
    for exec_mode in [
        ExecutionMode::CpuOnly,
        ExecutionMode::GpuPreferred,
        ExecutionMode::GpuOnly,
        ExecutionMode::Hybrid,
    ] {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{:?}", exec_mode)),
            &exec_mode,
            |b, &mode| {
                b.iter(|| {
                    let mut assignment = Assignment::new(black_box(1));
                    assignment.execution_mode = mode;
                    black_box(assignment)
                });
            }
        );
    }
    
    group.finish();
}

fn benchmark_cpu_binding(c: &mut Criterion) {
    c.bench_function("cpu_bind", |b| {
        b.iter(|| {
            let mut assignment = Assignment::new(1);
            assignment.bind_cpu();
            black_box(assignment)
        });
    });
}

fn benchmark_gpu_binding(c: &mut Criterion) {
    c.bench_function("gpu_bind", |b| {
        b.iter(|| {
            let mut assignment = Assignment::new(1);
            assignment.bind_gpu();
            black_box(assignment)
        });
    });
}

fn benchmark_wbackend_add_assignment(c: &mut Criterion) {
    let mut group = c.benchmark_group("wbackend_add");
    
    for mode in [ResourceMode::Auto, ResourceMode::Manual] {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{:?}", mode)),
            &mode,
            |b, &mode| {
                let backend = WBackend::new(mode);
                let mut id = 1;
                
                b.iter(|| {
                    let mut assignment = Assignment::new(id);
                    assignment.execution_mode = ExecutionMode::GpuPreferred;
                    backend.add_assignment(black_box(assignment));
                    id += 1;
                });
            }
        );
    }
    
    group.finish();
}

fn benchmark_wbackend_run_cycle(c: &mut Criterion) {
    let mut group = c.benchmark_group("wbackend_cycle");
    
    for assignment_count in [10, 50, 100, 500] {
        group.bench_with_input(
            BenchmarkId::from_parameter(assignment_count),
            &assignment_count,
            |b, &count| {
                let backend = WBackend::new(ResourceMode::Auto);
                
                // Add assignments
                for i in 1..=count {
                    let mut assignment = Assignment::new(i);
                    assignment.execution_mode = ExecutionMode::GpuPreferred;
                    backend.add_assignment(assignment);
                }
                
                b.iter(|| {
                    backend.run_cycle();
                });
            }
        );
    }
    
    group.finish();
}

fn benchmark_resource_limits_creation(c: &mut Criterion) {
    c.bench_function("resource_limits_default", |b| {
        b.iter(|| {
            let limits = ResourceLimits::default();
            black_box(limits)
        });
    });
    
    c.bench_function("resource_limits_custom", |b| {
        b.iter(|| {
            let limits = ResourceLimits {
                max_memory_mb: 2048,
                max_gpu_memory_mb: 1024,
                cpu_cores: vec![0, 1, 2, 3],
                execution_mode: Some(ExecutionMode::Hybrid),
                lease_duration: Duration::from_secs(60),
                renderer: "glx_renderer".to_string(),
                pixel_load_limit: 75,
            };
            black_box(limits)
        });
    });
}

fn benchmark_adjust_window_resources(c: &mut Criterion) {
    let handler = WindowHandler::new(ResourceMode::Manual);
    let geometry = WindowGeometry { x: 0, y: 0, width: 800, height: 600 };
    
    let window_id = handler.create_window(
        "Resource Test".to_string(),
        "resource.test".to_string(),
        geometry,
        None,
        ResourceMode::Manual,
    ).unwrap();
    
    c.bench_function("adjust_resources", |b| {
        b.iter(|| {
            let new_limits = ResourceLimits {
                max_memory_mb: 1024,
                max_gpu_memory_mb: 512,
                cpu_cores: vec![0, 1],
                execution_mode: Some(ExecutionMode::GpuPreferred),
                lease_duration: Duration::from_secs(30),
                renderer: "cpu_renderer".to_string(),
                pixel_load_limit: 50,
            };
            handler.adjust_window_resources(black_box(window_id), black_box(new_limits)).ok();
        });
    });
}

fn benchmark_get_window_resources(c: &mut Criterion) {
    let handler = WindowHandler::new(ResourceMode::Auto);
    let geometry = WindowGeometry { x: 0, y: 0, width: 800, height: 600 };
    
    let window_id = handler.create_window(
        "Query Test".to_string(),
        "query.test".to_string(),
        geometry,
        None,
        ResourceMode::Auto,
    ).unwrap();
    
    c.bench_function("get_window_resources", |b| {
        b.iter(|| {
            let usage = handler.get_window_resource_usage(black_box(window_id));
            black_box(usage)
        });
    });
}

fn benchmark_lease_management(c: &mut Criterion) {
    c.bench_function("lease_start", |b| {
        b.iter(|| {
            let mut assignment = Assignment::new(1);
            assignment.start_lease(black_box(Duration::from_secs(30)));
            black_box(assignment)
        });
    });
    
    c.bench_function("lease_check_expired", |b| {
        let mut assignment = Assignment::new(1);
        assignment.start_lease(Duration::from_millis(1));
        std::thread::sleep(Duration::from_millis(2));
        
        b.iter(|| {
            let expired = assignment.lease_expired();
            black_box(expired)
        });
    });
}

fn benchmark_execution_mode_switching(c: &mut Criterion) {
    let handler = WindowHandler::new(ResourceMode::Manual);
    let geometry = WindowGeometry { x: 0, y: 0, width: 800, height: 600 };
    
    let window_id = handler.create_window(
        "Mode Test".to_string(),
        "mode.test".to_string(),
        geometry,
        None,
        ResourceMode::Manual,
    ).unwrap();
    
    let modes = [
        ExecutionMode::CpuOnly,
        ExecutionMode::GpuPreferred,
        ExecutionMode::GpuOnly,
        ExecutionMode::Hybrid,
    ];
    
    c.bench_function("execution_mode_switch", |b| {
        let mut idx = 0;
        b.iter(|| {
            let window = handler.get_window(window_id).unwrap();
            let mut new_limits = window.resource_limits.clone();
            new_limits.execution_mode = Some(modes[idx % modes.len()]);
            handler.adjust_window_resources(black_box(window_id), black_box(new_limits)).ok();
            idx += 1;
        });
    });
}

fn benchmark_task_lifecycle(c: &mut Criterion) {
    c.bench_function("task_start_stop", |b| {
        b.iter_batched(
            || {
                // Setup
                let mut assignment = Assignment::new(1);
                assignment.execution_mode = ExecutionMode::CpuOnly;
                assignment
            },
            |mut assignment| {
                // Benchmark
                assignment.start_task();
                std::thread::sleep(Duration::from_millis(1));
                assignment.stop_task();
                black_box(assignment)
            },
            criterion::BatchSize::SmallInput
        );
    });
}

fn benchmark_concurrent_assignments(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_assignments");
    
    for count in [10, 50, 100] {
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            &count,
            |b, &count| {
                b.iter(|| {
                    let backend = WBackend::new(ResourceMode::Auto);
                    
                    // Add assignments concurrently
                    let handles: Vec<_> = (1..=count)
                        .map(|i| {
                            std::thread::spawn(move || {
                                let mut assignment = Assignment::new(i);
                                assignment.execution_mode = ExecutionMode::GpuPreferred;
                                assignment
                            })
                        })
                        .collect();
                    
                    for handle in handles {
                        let assignment = handle.join().unwrap();
                        backend.add_assignment(assignment);
                    }
                    
                    backend.run_cycle();
                    black_box(backend)
                });
            }
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_assignment_creation,
    benchmark_cpu_binding,
    benchmark_gpu_binding,
    benchmark_wbackend_add_assignment,
    benchmark_wbackend_run_cycle,
    benchmark_resource_limits_creation,
    benchmark_adjust_window_resources,
    benchmark_get_window_resources,
    benchmark_lease_management,
    benchmark_execution_mode_switching,
    benchmark_task_lifecycle,
    benchmark_concurrent_assignments,
);
criterion_main!(benches);
