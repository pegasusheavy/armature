//! Rate limiting benchmarks for armature-ratelimit

#![allow(deprecated)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use std::time::Duration;

use armature_ratelimit::{Algorithm, RateLimiter};

fn token_bucket_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("token_bucket");
    group.throughput(Throughput::Elements(1));

    // High capacity for sustained benchmark
    let limiter = rt.block_on(async {
        RateLimiter::builder()
            .algorithm(Algorithm::TokenBucket {
                capacity: 1_000_000,
                refill_rate: 100_000.0,
            })
            .build()
            .await
            .unwrap()
    });

    group.bench_function("check_allowed", |b| {
        b.to_async(&rt).iter(|| async {
            let result = limiter.check(black_box("benchmark_key")).await.unwrap();
            black_box(result)
        });
    });

    group.finish();
}

fn fixed_window_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("fixed_window");
    group.throughput(Throughput::Elements(1));

    let limiter = rt.block_on(async {
        RateLimiter::builder()
            .algorithm(Algorithm::FixedWindow {
                max_requests: 1_000_000,
                window: Duration::from_secs(3600),
            })
            .build()
            .await
            .unwrap()
    });

    group.bench_function("check_allowed", |b| {
        b.to_async(&rt).iter(|| async {
            let result = limiter.check(black_box("benchmark_key")).await.unwrap();
            black_box(result)
        });
    });

    group.finish();
}

fn sliding_window_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("sliding_window");
    group.throughput(Throughput::Elements(1));

    let limiter = rt.block_on(async {
        RateLimiter::builder()
            .algorithm(Algorithm::SlidingWindowLog {
                max_requests: 1_000_000,
                window: Duration::from_secs(3600),
            })
            .build()
            .await
            .unwrap()
    });

    group.bench_function("check_allowed", |b| {
        b.to_async(&rt).iter(|| async {
            let result = limiter.check(black_box("benchmark_key")).await.unwrap();
            black_box(result)
        });
    });

    group.finish();
}

fn algorithm_comparison(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("algorithm_comparison");
    group.throughput(Throughput::Elements(1));

    let algorithms = [
        (
            "token_bucket",
            Algorithm::TokenBucket {
                capacity: 1_000_000,
                refill_rate: 100_000.0,
            },
        ),
        (
            "fixed_window",
            Algorithm::FixedWindow {
                max_requests: 1_000_000,
                window: Duration::from_secs(3600),
            },
        ),
        (
            "sliding_window",
            Algorithm::SlidingWindowLog {
                max_requests: 1_000_000,
                window: Duration::from_secs(3600),
            },
        ),
    ];

    for (name, algorithm) in algorithms {
        let limiter = rt.block_on(async {
            RateLimiter::builder()
                .algorithm(algorithm)
                .build()
                .await
                .unwrap()
        });

        group.bench_function(name, |b| {
            b.to_async(&rt).iter(|| async {
                let result = limiter.check(black_box("benchmark_key")).await.unwrap();
                black_box(result)
            });
        });
    }

    group.finish();
}

fn multi_key_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("multi_key");

    let limiter = rt.block_on(async {
        RateLimiter::builder()
            .algorithm(Algorithm::TokenBucket {
                capacity: 1_000_000,
                refill_rate: 100_000.0,
            })
            .build()
            .await
            .unwrap()
    });

    for num_keys in [10, 100, 1000].iter() {
        let keys: Vec<String> = (0..*num_keys).map(|i| format!("user_{}", i)).collect();
        let idx = std::sync::atomic::AtomicUsize::new(0);

        group.bench_with_input(
            BenchmarkId::new("unique_keys", num_keys),
            num_keys,
            |b, _| {
                b.to_async(&rt).iter(|| async {
                    let current = idx.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let key = &keys[current % keys.len()];
                    let result = limiter.check(black_box(key)).await.unwrap();
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

fn reset_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("reset_operations");

    let limiter = rt.block_on(async {
        RateLimiter::builder()
            .algorithm(Algorithm::TokenBucket {
                capacity: 100,
                refill_rate: 10.0,
            })
            .build()
            .await
            .unwrap()
    });

    group.bench_function("reset_key", |b| {
        b.to_async(&rt).iter(|| async {
            limiter.reset(black_box("reset_key")).await.unwrap();
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    token_bucket_benchmark,
    fixed_window_benchmark,
    sliding_window_benchmark,
    algorithm_comparison,
    multi_key_benchmark,
    reset_benchmark,
);

criterion_main!(benches);
