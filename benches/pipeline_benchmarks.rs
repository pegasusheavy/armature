//! Pipeline Configuration Benchmarks
//!
//! Benchmarks for HTTP/1.1 pipelining configuration and statistics tracking.
//!
//! Run with: cargo bench --bench pipeline_benchmarks

use armature_core::pipeline::{ConnectionStats, PipelineConfig, PipelineMode, PipelineStats};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::sync::Arc;

// ============================================================================
// Configuration Benchmarks
// ============================================================================

fn bench_config_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("pipeline_config");

    group.bench_function("default", |b| {
        b.iter(|| {
            let config = PipelineConfig::default();
            black_box(config.max_concurrent)
        })
    });

    group.bench_function("high_performance", |b| {
        b.iter(|| {
            let config = PipelineConfig::high_performance();
            black_box(config.max_concurrent)
        })
    });

    group.bench_function("low_latency", |b| {
        b.iter(|| {
            let config = PipelineConfig::low_latency();
            black_box(config.max_concurrent)
        })
    });

    group.bench_function("builder", |b| {
        b.iter(|| {
            let config = PipelineConfig::builder()
                .mode(PipelineMode::Concurrent)
                .max_concurrent(32)
                .pipeline_flush(true)
                .build();
            black_box(config.max_concurrent)
        })
    });

    group.finish();
}

// ============================================================================
// Statistics Benchmarks
// ============================================================================

fn bench_connection_stats(c: &mut Criterion) {
    let mut group = c.benchmark_group("connection_stats");

    group.bench_function("request_received", |b| {
        let stats = ConnectionStats::new();
        b.iter(|| {
            stats.request_received(black_box(1024));
        })
    });

    group.bench_function("response_sent", |b| {
        let stats = ConnectionStats::new();
        // Pre-populate
        for _ in 0..100 {
            stats.request_received(1024);
        }
        b.iter(|| {
            stats.response_sent(black_box(2048));
        })
    });

    group.bench_function("read_stats", |b| {
        let stats = ConnectionStats::new();
        stats.request_received(1024);
        stats.response_sent(2048);
        b.iter(|| {
            let _ = black_box(stats.requests_processed());
            let _ = black_box(stats.pending_requests());
            let _ = black_box(stats.pipeline_depth());
            let _ = black_box(stats.bytes_received());
            let _ = black_box(stats.bytes_sent());
        })
    });

    group.finish();
}

fn bench_global_stats(c: &mut Criterion) {
    let mut group = c.benchmark_group("pipeline_stats");

    group.bench_function("connection_opened", |b| {
        let stats = Arc::new(PipelineStats::new());
        b.iter(|| {
            stats.connection_opened();
        })
    });

    group.bench_function("connection_closed", |b| {
        let stats = Arc::new(PipelineStats::new());
        // Pre-open connections
        for _ in 0..100 {
            stats.connection_opened();
        }
        b.iter(|| {
            stats.connection_closed();
            stats.connection_opened(); // Re-open to maintain count
        })
    });

    group.bench_function("request_processed", |b| {
        let stats = Arc::new(PipelineStats::new());
        b.iter(|| {
            stats.request_processed();
        })
    });

    group.bench_function("update_pipeline_depth", |b| {
        let stats = Arc::new(PipelineStats::new());
        b.iter(|| {
            stats.update_pipeline_depth(black_box(5));
        })
    });

    group.bench_function("read_all_stats", |b| {
        let stats = Arc::new(PipelineStats::new());
        stats.connection_opened();
        stats.request_processed();
        stats.update_pipeline_depth(5);
        b.iter(|| {
            let _ = black_box(stats.active_connections());
            let _ = black_box(stats.total_connections());
            let _ = black_box(stats.total_requests());
            let _ = black_box(stats.avg_pipeline_depth());
            let _ = black_box(stats.max_pipeline_depth());
        })
    });

    group.finish();
}

// ============================================================================
// Concurrent Access Benchmarks
// ============================================================================

fn bench_concurrent_stats(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_stats");

    group.bench_function("multi_thread_request_processed", |b| {
        let stats = Arc::new(PipelineStats::new());

        b.iter(|| {
            let handles: Vec<_> = (0..4)
                .map(|_| {
                    let stats = Arc::clone(&stats);
                    std::thread::spawn(move || {
                        for _ in 0..100 {
                            stats.request_processed();
                        }
                    })
                })
                .collect();

            for h in handles {
                h.join().unwrap();
            }

            black_box(stats.total_requests())
        })
    });

    group.finish();
}

criterion_group!(
    pipeline_benches,
    bench_config_creation,
    bench_connection_stats,
    bench_global_stats,
    bench_concurrent_stats,
);

criterion_main!(pipeline_benches);

