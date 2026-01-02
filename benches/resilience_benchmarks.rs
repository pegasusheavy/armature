#![allow(clippy::all)]
#![allow(deprecated)]
#![allow(clippy::needless_question_mark)]

//! Resilience Pattern Benchmarks
//!
//! Benchmarks for circuit breakers, retry strategies, bulkheads, timeouts, and fallbacks.

use armature_core::resilience::*;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;
use std::sync::Arc;
use std::time::Duration;

// =============================================================================
// Circuit Breaker Benchmarks
// =============================================================================

fn bench_circuit_breaker(c: &mut Criterion) {
    let mut group = c.benchmark_group("circuit_breaker");

    // Circuit breaker creation
    group.bench_function("create_default", |b| {
        b.iter(|| CircuitBreaker::new(CircuitBreakerConfig::default()))
    });

    // State checks
    let cb = Arc::new(CircuitBreaker::new(CircuitBreakerConfig::default()));

    group.bench_function("is_allowed", |b| {
        let cb = cb.clone();
        b.iter(|| black_box(cb.is_allowed()))
    });

    group.bench_function("state_check", |b| {
        let cb = cb.clone();
        b.iter(|| black_box(cb.state()))
    });

    // Recording results
    group.bench_function("record_success", |b| {
        let cb = CircuitBreaker::new(CircuitBreakerConfig::default());
        b.iter(|| cb.record_success())
    });

    group.bench_function("record_failure", |b| {
        b.iter_batched(
            || {
                CircuitBreaker::new(CircuitBreakerConfig {
                    failure_threshold: 1000,
                    ..Default::default()
                })
            },
            |cb| cb.record_failure(),
            criterion::BatchSize::SmallInput,
        )
    });

    group.finish();
}

// =============================================================================
// Retry Benchmarks
// =============================================================================

fn bench_retry(c: &mut Criterion) {
    let mut group = c.benchmark_group("retry");

    // Retry config creation - using default
    group.bench_function("config_default", |b| b.iter(RetryConfig::default));

    // Retry creation
    group.bench_function("retry_new", |b| {
        let config = RetryConfig::default();
        b.iter(|| Retry::new(config.clone()))
    });

    // Backoff calculation
    let config = RetryConfig::default();
    for attempt in [1, 3, 5, 10] {
        group.bench_with_input(
            BenchmarkId::new("delay_for_attempt", attempt),
            &attempt,
            |b, &attempt| b.iter(|| config.backoff.delay_for_attempt(black_box(attempt))),
        );
    }

    group.finish();
}

// =============================================================================
// Bulkhead Benchmarks
// =============================================================================

fn bench_bulkhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("bulkhead");
    let runtime = tokio::runtime::Runtime::new().unwrap();

    // Bulkhead creation
    group.bench_function("create", |b| {
        b.iter(|| {
            Bulkhead::new(BulkheadConfig {
                name: "test".to_string(),
                max_concurrent: 10,
                max_wait: Duration::from_secs(5),
                queue_size: None,
            })
        })
    });

    // Stats retrieval
    let bulkhead = Bulkhead::new(BulkheadConfig {
        name: "test".to_string(),
        max_concurrent: 100,
        max_wait: Duration::from_secs(5),
        queue_size: None,
    });

    group.bench_function("stats", |b| b.iter(|| black_box(bulkhead.stats())));

    // Permit acquisition (successful case)
    for concurrency in [10, 50, 100] {
        let bulkhead = Bulkhead::new(BulkheadConfig {
            name: "test".to_string(),
            max_concurrent: concurrency,
            max_wait: Duration::from_secs(5),
            queue_size: None,
        });

        group.bench_with_input(
            BenchmarkId::new("call_success", concurrency),
            &concurrency,
            |b, _| {
                b.to_async(&runtime).iter(|| async {
                    let _ = bulkhead
                        .call(|| async { Ok::<_, std::convert::Infallible>(42) })
                        .await;
                })
            },
        );
    }

    group.finish();
}

// =============================================================================
// Timeout Benchmarks
// =============================================================================

fn bench_timeout(c: &mut Criterion) {
    let mut group = c.benchmark_group("timeout");
    let runtime = tokio::runtime::Runtime::new().unwrap();

    // Timeout creation
    group.bench_function("create", |b| {
        b.iter(|| Timeout::with_duration(Duration::from_secs(5)))
    });

    // Successful timeout (operation completes in time)
    let timeout = Timeout::with_duration(Duration::from_secs(10));
    group.bench_function("call_fast_operation", |b| {
        b.to_async(&runtime).iter(|| async {
            let _: Result<i32, TimeoutError<()>> = timeout.call(|| async { Ok(42) }).await;
        })
    });

    group.finish();
}

// =============================================================================
// Fallback Benchmarks
// =============================================================================

fn bench_fallback(c: &mut Criterion) {
    let mut group = c.benchmark_group("fallback");
    let runtime = tokio::runtime::Runtime::new().unwrap();

    // Single fallback - primary succeeds
    group.bench_function("fallback_not_needed", |b| {
        let fallback: Fallback<String, String> =
            Fallback::new(|| async { Ok("fallback".to_string()) });

        b.to_async(&runtime).iter(|| async {
            let result = fallback
                .call(|| async { Ok::<String, String>("primary".to_string()) })
                .await;
            black_box(result)
        })
    });

    // Single fallback - primary fails
    group.bench_function("fallback_needed", |b| {
        let fallback: Fallback<String, String> =
            Fallback::new(|| async { Ok("fallback".to_string()) });

        b.to_async(&runtime).iter(|| async {
            let result = fallback
                .call(|| async { Err::<String, String>("error".to_string()) })
                .await;
            black_box(result)
        })
    });

    // Fallback chain creation
    group.bench_function("chain_create", |b| {
        b.iter(|| FallbackChain::<String, String>::new())
    });

    group.finish();
}

// =============================================================================
// Combined Resilience Pattern Benchmarks
// =============================================================================

fn bench_combined_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("combined_patterns");
    let runtime = tokio::runtime::Runtime::new().unwrap();

    // Circuit breaker + retry check
    group.bench_function("cb_with_retry_check", |b| {
        let cb = CircuitBreaker::new(CircuitBreakerConfig::default());
        let retry_config = RetryConfig::default();

        b.iter(|| {
            let allowed = cb.is_allowed();
            let should_retry = retry_config.max_attempts > 0;
            black_box((allowed, should_retry))
        })
    });

    // Full resilience stack simulation
    group.bench_function("full_stack_overhead", |b| {
        let cb = CircuitBreaker::new(CircuitBreakerConfig::default());
        let bulkhead = Bulkhead::new(BulkheadConfig {
            name: "test".to_string(),
            max_concurrent: 100,
            max_wait: Duration::from_secs(5),
            queue_size: None,
        });
        let timeout = Timeout::with_duration(Duration::from_secs(30));

        b.to_async(&runtime).iter(|| async {
            if !cb.is_allowed() {
                return black_box(Err::<i32, &str>("circuit_open"));
            }

            // Bulkhead wraps a simple operation for the benchmark
            let result = bulkhead.call(|| async { Ok::<_, &str>(42) }).await;

            match result {
                Ok(v) => {
                    cb.record_success();
                    // Also simulate timeout check
                    let _ = timeout.call(|| async { Ok::<_, &str>(v) }).await;
                    black_box(Ok(v))
                }
                Err(_) => {
                    cb.record_failure();
                    black_box(Err("failed"))
                }
            }
        })
    });

    group.finish();
}

criterion_group!(
    resilience_benches,
    bench_circuit_breaker,
    bench_retry,
    bench_bulkhead,
    bench_timeout,
    bench_fallback,
    bench_combined_patterns,
);

criterion_main!(resilience_benches);
