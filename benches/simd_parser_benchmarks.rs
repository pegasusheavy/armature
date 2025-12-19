//! Benchmarks for SIMD-optimized HTTP parsing
//!
//! Run with: cargo bench --bench simd_parser_benchmarks

use armature_core::simd_parser::*;
use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use std::hint::black_box;

fn bench_query_string_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_string");

    // Small query string (typical)
    let small_query = "page=1&limit=10&sort=asc";
    group.throughput(Throughput::Bytes(small_query.len() as u64));
    group.bench_function("small/simd", |b| {
        b.iter(|| parse_query_string_fast(black_box(small_query)))
    });

    // Naive implementation for comparison
    group.bench_function("small/naive", |b| {
        b.iter(|| {
            black_box(small_query)
                .split('&')
                .filter_map(|part| {
                    let mut split = part.splitn(2, '=');
                    let key = split.next()?;
                    let value = split.next().unwrap_or("");
                    Some((key.to_string(), value.to_string()))
                })
                .collect::<std::collections::HashMap<_, _>>()
        })
    });

    // Medium query string
    let medium_query = "user_id=12345&session_token=abc123xyz789&timestamp=1704067200&signature=sha256_hash_value&action=view&page=dashboard&theme=dark&language=en-US";
    group.throughput(Throughput::Bytes(medium_query.len() as u64));
    group.bench_function("medium/simd", |b| {
        b.iter(|| parse_query_string_fast(black_box(medium_query)))
    });

    group.bench_function("medium/naive", |b| {
        b.iter(|| {
            black_box(medium_query)
                .split('&')
                .filter_map(|part| {
                    let mut split = part.splitn(2, '=');
                    let key = split.next()?;
                    let value = split.next().unwrap_or("");
                    Some((key.to_string(), value.to_string()))
                })
                .collect::<std::collections::HashMap<_, _>>()
        })
    });

    // Large query string (API with many parameters)
    let large_query = "api_key=sk_live_12345678901234567890&user_id=usr_abc123&organization_id=org_xyz789&project_id=prj_qrs456&environment=production&version=2.0&format=json&include_metadata=true&expand=user,project,organization&fields=id,name,email,created_at,updated_at&sort_by=created_at&sort_order=desc&page=1&per_page=50&filter_status=active&filter_type=standard&filter_region=us-east-1&timestamp=1704067200&nonce=random_string_here&signature=sha256_hmac_signature_value_here_1234567890";
    group.throughput(Throughput::Bytes(large_query.len() as u64));
    group.bench_function("large/simd", |b| {
        b.iter(|| parse_query_string_fast(black_box(large_query)))
    });

    group.bench_function("large/naive", |b| {
        b.iter(|| {
            black_box(large_query)
                .split('&')
                .filter_map(|part| {
                    let mut split = part.splitn(2, '=');
                    let key = split.next()?;
                    let value = split.next().unwrap_or("");
                    Some((key.to_string(), value.to_string()))
                })
                .collect::<std::collections::HashMap<_, _>>()
        })
    });

    group.finish();
}

fn bench_url_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("url_decode");

    // No encoding (fast path)
    let plain = "hello_world_no_special_chars";
    group.bench_function("plain/simd", |b| {
        b.iter(|| url_decode(black_box(plain)))
    });

    // With percent encoding
    let encoded = "hello%20world%21%40%23%24%25%5E%26%2A%28%29";
    group.bench_function("encoded/simd", |b| {
        b.iter(|| url_decode(black_box(encoded)))
    });

    // With plus signs (form encoding)
    let plus_encoded = "hello+world+this+is+a+test+with+spaces";
    group.bench_function("plus_encoded/simd", |b| {
        b.iter(|| url_decode(black_box(plus_encoded)))
    });

    group.finish();
}

fn bench_uri_splitting(c: &mut Criterion) {
    let mut group = c.benchmark_group("uri_split");

    let with_query = "/api/v1/users/123?page=1&limit=10";
    group.bench_function("with_query/simd", |b| {
        b.iter(|| split_uri(black_box(with_query)))
    });

    group.bench_function("with_query/naive", |b| {
        b.iter(|| {
            let s = black_box(with_query);
            s.split_once('?')
                .map(|(p, q)| (p, Some(q)))
                .unwrap_or((s, None))
        })
    });

    let without_query = "/api/v1/users/123";
    group.bench_function("without_query/simd", |b| {
        b.iter(|| split_uri(black_box(without_query)))
    });

    group.bench_function("without_query/naive", |b| {
        b.iter(|| {
            let s = black_box(without_query);
            s.split_once('?')
                .map(|(p, q)| (p, Some(q)))
                .unwrap_or((s, None))
        })
    });

    group.finish();
}

fn bench_header_interning(c: &mut Criterion) {
    let mut group = c.benchmark_group("header_intern");

    // Common headers (fast path - interned)
    group.bench_function("common/content-type", |b| {
        b.iter(|| intern_header_name(black_box("content-type")))
    });

    group.bench_function("common/authorization", |b| {
        b.iter(|| intern_header_name(black_box("authorization")))
    });

    // Custom headers (slow path - allocation)
    group.bench_function("custom/x-custom-header", |b| {
        b.iter(|| intern_header_name(black_box("X-Custom-Header")))
    });

    group.finish();
}

fn bench_path_splitting(c: &mut Criterion) {
    let mut group = c.benchmark_group("path_split");

    let short_path = "/api/users";
    group.bench_function("short/simd", |b| {
        b.iter(|| split_path(black_box(short_path)).collect::<Vec<_>>())
    });

    let medium_path = "/api/v1/users/123/posts/456/comments";
    group.bench_function("medium/simd", |b| {
        b.iter(|| split_path(black_box(medium_path)).collect::<Vec<_>>())
    });

    let long_path = "/api/v2/organizations/org123/projects/proj456/environments/env789/deployments/deploy012/logs/log345/entries";
    group.bench_function("long/simd", |b| {
        b.iter(|| split_path(black_box(long_path)).collect::<Vec<_>>())
    });

    group.finish();
}

fn bench_path_param_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("path_params");

    let pattern = "/users/:id";
    let path = "/users/123";
    group.bench_function("single_param", |b| {
        b.iter(|| extract_path_params(black_box(pattern), black_box(path)))
    });

    let pattern2 = "/users/:user_id/posts/:post_id/comments/:comment_id";
    let path2 = "/users/123/posts/456/comments/789";
    group.bench_function("multiple_params", |b| {
        b.iter(|| extract_path_params(black_box(pattern2), black_box(path2)))
    });

    group.finish();
}

criterion_group!(
    simd_benches,
    bench_query_string_parsing,
    bench_url_decode,
    bench_uri_splitting,
    bench_header_interning,
    bench_path_splitting,
    bench_path_param_extraction,
);

criterion_main!(simd_benches);

