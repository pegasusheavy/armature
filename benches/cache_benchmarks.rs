#![allow(deprecated)]
#![allow(clippy::needless_question_mark)]

//! Cache benchmarks for armature-cache

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use std::sync::Arc;
use std::time::Duration;

// Import from armature-cache
use armature_cache::{CacheStore, InMemoryCache, TieredCache};

fn cache_set_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let cache = Arc::new(InMemoryCache::new());

    let mut group = c.benchmark_group("cache_set");
    group.throughput(Throughput::Elements(1));

    group.bench_function("set_json_no_ttl", |b| {
        b.to_async(&rt).iter(|| async {
            let cache = cache.clone();
            cache
                .set_json("benchmark_key", "benchmark_value".to_string(), None)
                .await
                .unwrap();
        });
    });

    group.bench_function("set_json_with_ttl", |b| {
        b.to_async(&rt).iter(|| async {
            let cache = cache.clone();
            cache
                .set_json(
                    "benchmark_key",
                    "benchmark_value".to_string(),
                    Some(Duration::from_secs(60)),
                )
                .await
                .unwrap();
        });
    });

    group.finish();
}

fn cache_get_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let cache = Arc::new(InMemoryCache::new());

    // Pre-populate cache
    rt.block_on(async {
        cache
            .set_json("existing_key", "existing_value".to_string(), None)
            .await
            .unwrap();
    });

    let mut group = c.benchmark_group("cache_get");
    group.throughput(Throughput::Elements(1));

    group.bench_function("get_json_hit", |b| {
        b.to_async(&rt).iter(|| async {
            let cache = cache.clone();
            let result = cache.get_json("existing_key").await.unwrap();
            black_box(result)
        });
    });

    group.bench_function("get_json_miss", |b| {
        b.to_async(&rt).iter(|| async {
            let cache = cache.clone();
            let result = cache.get_json("nonexistent_key").await.unwrap();
            black_box(result)
        });
    });

    group.finish();
}

fn cache_operations_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let cache = Arc::new(InMemoryCache::new());

    let mut group = c.benchmark_group("cache_operations");

    group.bench_function("exists_check", |b| {
        let cache = cache.clone();
        rt.block_on(async {
            cache
                .set_json("check_key", "value".to_string(), None)
                .await
                .unwrap();
        });
        b.to_async(&rt).iter(|| async {
            let cache = cache.clone();
            let result = cache.exists("check_key").await.unwrap();
            black_box(result)
        });
    });

    group.bench_function("delete", |b| {
        b.to_async(&rt).iter(|| async {
            let cache = cache.clone();
            cache
                .set_json("delete_key", "value".to_string(), None)
                .await
                .unwrap();
            cache.delete("delete_key").await.unwrap();
        });
    });

    group.bench_function("increment", |b| {
        b.to_async(&rt).iter(|| async {
            let cache = cache.clone();
            let result = cache.increment("counter", 1).await.unwrap();
            black_box(result)
        });
    });

    group.finish();
}

fn tiered_cache_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let l1 = Arc::new(InMemoryCache::new());
    let l2 = Arc::new(InMemoryCache::new());
    let tiered = TieredCache::new(l1.clone(), l2.clone());

    let mut group = c.benchmark_group("tiered_cache");

    group.bench_function("set", |b| {
        b.to_async(&rt).iter(|| async {
            tiered.set("key", "value".to_string(), None).await.unwrap();
        });
    });

    group.bench_function("get_l1_hit", |b| {
        rt.block_on(async {
            tiered
                .set("l1_key", "l1_value".to_string(), None)
                .await
                .unwrap();
        });
        b.to_async(&rt).iter(|| async {
            let result = tiered.get("l1_key").await.unwrap();
            black_box(result)
        });
    });

    group.bench_function("get_l2_promotion", |b| {
        rt.block_on(async {
            // Set directly in L2
            l2.set_json("l2_only_key", "l2_value".to_string(), None)
                .await
                .unwrap();
        });
        b.to_async(&rt).iter(|| async {
            let result = tiered.get("l2_only_key").await.unwrap();
            black_box(result)
        });
    });

    group.finish();
}

fn batch_operations_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let cache = Arc::new(InMemoryCache::new());

    let mut group = c.benchmark_group("batch_operations");

    for batch_size in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("get_many", batch_size),
            batch_size,
            |b, &size| {
                let cache = cache.clone();
                let keys: Vec<String> = (0..size).map(|i| format!("key_{}", i)).collect();

                // Pre-populate
                rt.block_on(async {
                    for key in &keys {
                        cache
                            .set_json(key, "value".to_string(), None)
                            .await
                            .unwrap();
                    }
                });

                let key_refs: Vec<&str> = keys.iter().map(|s| s.as_str()).collect();

                b.to_async(&rt).iter(|| async {
                    let cache = cache.clone();
                    let result = cache.get_many(&key_refs).await.unwrap();
                    black_box(result)
                });
            },
        );
    }

    for batch_size in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("set_many", batch_size),
            batch_size,
            |b, &size| {
                let cache = cache.clone();
                let items: Vec<(&str, String)> = (0..size)
                    .map(|i| {
                        let key = Box::leak(format!("batch_key_{}", i).into_boxed_str());
                        (key as &str, format!("value_{}", i))
                    })
                    .collect();

                b.to_async(&rt).iter(|| async {
                    let cache = cache.clone();
                    cache.set_many(&items, None).await.unwrap();
                });
            },
        );
    }

    group.finish();
}

fn value_size_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let cache = Arc::new(InMemoryCache::new());

    let mut group = c.benchmark_group("value_sizes");

    // Test different value sizes
    let sizes = [
        ("100B", 100usize),
        ("1KB", 1024),
        ("10KB", 10 * 1024),
        ("100KB", 100 * 1024),
    ];

    for (name, size) in sizes {
        let value: String = "x".repeat(size);
        let cache_clone = cache.clone();

        group.bench_function(format!("set_{}", name), |b| {
            let cache = cache_clone.clone();
            let value = value.clone();
            b.to_async(&rt).iter(|| {
                let cache = cache.clone();
                let value = value.clone();
                async move {
                    cache.set_json("sized_key", value, None).await.unwrap();
                }
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    cache_set_benchmark,
    cache_get_benchmark,
    cache_operations_benchmark,
    tiered_cache_benchmark,
    batch_operations_benchmark,
    value_size_benchmark,
);

criterion_main!(benches);
