use armature_jwt::*;
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use jsonwebtoken::Algorithm;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
struct BenchClaims {
    sub: String,
    name: String,
    admin: bool,
}

fn bench_jwt_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("jwt");

    let config = JwtConfig::new("test_secret_key_32_bytes_long!!!".to_string())
        .with_expiration(Duration::from_secs(3600));
    let manager = JwtManager::new(config).unwrap();

    let claims = BenchClaims {
        sub: "user_123".to_string(),
        name: "John Doe".to_string(),
        admin: false,
    };

    group.bench_function("sign", |b| {
        b.iter(|| manager.sign(black_box(&claims)).unwrap())
    });

    let token = manager.sign(&claims).unwrap();

    group.bench_function("verify", |b| {
        b.iter(|| {
            let _: Claims<BenchClaims> = manager.verify(black_box(&token)).unwrap();
        })
    });

    // Test different algorithms
    for algo in [Algorithm::HS256, Algorithm::HS384, Algorithm::HS512].iter() {
        let config =
            JwtConfig::new("test_secret_key_32_bytes_long!!!".to_string()).with_algorithm(*algo);
        let manager = JwtManager::new(config).unwrap();

        group.bench_function(format!("sign_{:?}", algo), |b| {
            b.iter(|| manager.sign(black_box(&claims)).unwrap())
        });
    }

    group.finish();
}

criterion_group!(security_benches, bench_jwt_operations);

criterion_main!(security_benches);
