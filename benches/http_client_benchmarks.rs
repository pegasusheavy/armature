#![allow(deprecated)]
#![allow(clippy::needless_question_mark)]

//! HTTP Client benchmarks for armature-http-client

use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use std::time::Duration;

use armature_http_client::{
    BackoffStrategy, CircuitBreaker, CircuitBreakerConfig, HttpClientConfig, RetryConfig,
};

fn retry_config_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("retry_config");

    group.bench_function("create_exponential", |b| {
        b.iter(|| {
            let config = RetryConfig::exponential(3, Duration::from_millis(100));
            black_box(config)
        });
    });

    group.bench_function("create_linear", |b| {
        b.iter(|| {
            let config = RetryConfig::linear(3, Duration::from_millis(100));
            black_box(config)
        });
    });

    group.bench_function("create_constant", |b| {
        b.iter(|| {
            let config = RetryConfig::constant(3, Duration::from_millis(100));
            black_box(config)
        });
    });

    group.bench_function("create_immediate", |b| {
        b.iter(|| {
            let config = RetryConfig::immediate(3);
            black_box(config)
        });
    });

    group.finish();
}

fn backoff_calculation_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("backoff_calculation");

    let exponential = BackoffStrategy::Exponential {
        initial: Duration::from_millis(100),
        max: Duration::from_secs(30),
        multiplier: 2.0,
    };

    let linear = BackoffStrategy::Linear {
        delay: Duration::from_millis(100),
        max: Duration::from_secs(30),
    };

    let constant = BackoffStrategy::Constant(Duration::from_millis(500));

    group.bench_function("exponential_attempt_0", |b| {
        b.iter(|| {
            let delay = exponential.delay_for_attempt(black_box(0));
            black_box(delay)
        });
    });

    group.bench_function("exponential_attempt_5", |b| {
        b.iter(|| {
            let delay = exponential.delay_for_attempt(black_box(5));
            black_box(delay)
        });
    });

    group.bench_function("exponential_attempt_10", |b| {
        b.iter(|| {
            let delay = exponential.delay_for_attempt(black_box(10));
            black_box(delay)
        });
    });

    group.bench_function("linear_attempt_0", |b| {
        b.iter(|| {
            let delay = linear.delay_for_attempt(black_box(0));
            black_box(delay)
        });
    });

    group.bench_function("linear_attempt_5", |b| {
        b.iter(|| {
            let delay = linear.delay_for_attempt(black_box(5));
            black_box(delay)
        });
    });

    group.bench_function("constant_any_attempt", |b| {
        b.iter(|| {
            let delay = constant.delay_for_attempt(black_box(5));
            black_box(delay)
        });
    });

    group.finish();
}

fn circuit_breaker_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("circuit_breaker");

    group.bench_function("create", |b| {
        b.iter(|| {
            let cb = CircuitBreaker::new(CircuitBreakerConfig::default());
            black_box(cb)
        });
    });

    group.bench_function("is_allowed_closed", |b| {
        let cb = CircuitBreaker::new(CircuitBreakerConfig::default());
        b.iter(|| {
            let allowed = cb.is_allowed();
            black_box(allowed)
        });
    });

    group.bench_function("record_success", |b| {
        let cb = CircuitBreaker::new(CircuitBreakerConfig::default());
        b.iter(|| {
            cb.record_success();
        });
    });

    group.bench_function("state_check", |b| {
        let cb = CircuitBreaker::new(CircuitBreakerConfig::default());
        b.iter(|| {
            let state = cb.state();
            black_box(state)
        });
    });

    group.bench_function("failure_count", |b| {
        let cb = CircuitBreaker::new(CircuitBreakerConfig::default());
        b.iter(|| {
            let count = cb.failure_count();
            black_box(count)
        });
    });

    group.finish();
}

fn circuit_breaker_state_transitions(c: &mut Criterion) {
    let mut group = c.benchmark_group("circuit_breaker_transitions");

    group.bench_function("multiple_successes", |b| {
        let cb = CircuitBreaker::new(CircuitBreakerConfig::default());
        b.iter(|| {
            for _ in 0..10 {
                cb.record_success();
            }
            black_box(cb.state())
        });
    });

    group.bench_function("reset", |b| {
        let cb = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 3,
            ..Default::default()
        });

        // Trip the circuit
        for _ in 0..5 {
            cb.record_failure();
        }

        b.iter(|| {
            cb.reset();
        });
    });

    group.finish();
}

fn http_client_config_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("http_client_config");

    group.bench_function("builder_default", |b| {
        b.iter(|| {
            let config = HttpClientConfig::default();
            black_box(config)
        });
    });

    group.bench_function("builder_with_retry", |b| {
        b.iter(|| {
            let config = HttpClientConfig::builder()
                .timeout(Duration::from_secs(30))
                .retry(RetryConfig::exponential(3, Duration::from_millis(100)))
                .build();
            black_box(config)
        });
    });

    group.bench_function("builder_with_circuit_breaker", |b| {
        b.iter(|| {
            let config = HttpClientConfig::builder()
                .timeout(Duration::from_secs(30))
                .circuit_breaker(CircuitBreakerConfig::default())
                .build();
            black_box(config)
        });
    });

    group.bench_function("builder_full", |b| {
        b.iter(|| {
            let config = HttpClientConfig::builder()
                .timeout(Duration::from_secs(30))
                .retry(RetryConfig::exponential(3, Duration::from_millis(100)))
                .circuit_breaker(CircuitBreakerConfig::new(5, Duration::from_secs(30)))
                .build();
            black_box(config)
        });
    });

    group.finish();
}

fn retry_should_retry_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("retry_should_retry");

    let config = RetryConfig::default();

    group.bench_function("check_status_code_match", |b| {
        b.iter(|| {
            let should_retry = config.should_retry_status(black_box(503));
            black_box(should_retry)
        });
    });

    group.bench_function("check_status_code_no_match", |b| {
        b.iter(|| {
            let should_retry = config.should_retry_status(black_box(200));
            black_box(should_retry)
        });
    });

    group.bench_function("delay_for_attempt", |b| {
        b.iter(|| {
            let delay = config.delay_for_attempt(black_box(2));
            black_box(delay)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    retry_config_benchmark,
    backoff_calculation_benchmark,
    circuit_breaker_benchmark,
    circuit_breaker_state_transitions,
    http_client_config_benchmark,
    retry_should_retry_benchmark,
);

criterion_main!(benches);
