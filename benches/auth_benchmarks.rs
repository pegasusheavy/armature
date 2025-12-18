//! Authentication benchmarks for armature-auth

#![allow(deprecated)]

use criterion::{Criterion, black_box, criterion_group, criterion_main};

use armature_auth::{AuthService, PasswordHasher};
use armature_jwt::{JwtConfig, JwtManager, StandardClaims};

fn password_hashing_benchmark(c: &mut Criterion) {
    let auth_service = AuthService::new();

    let mut group = c.benchmark_group("password_hashing");

    group.bench_function("hash_password", |b| {
        b.iter(|| {
            let hash = auth_service
                .hash_password(black_box("my_secure_password_123"))
                .unwrap();
            black_box(hash)
        });
    });

    // Pre-hash a password for verification benchmarks
    let pre_hashed = auth_service
        .hash_password("my_secure_password_123")
        .unwrap();
    let pre_hashed_clone = pre_hashed.clone();

    group.bench_function("verify_password_success", |b| {
        b.iter(|| {
            let result = auth_service
                .verify_password(
                    black_box("my_secure_password_123"),
                    black_box(&pre_hashed_clone),
                )
                .unwrap();
            black_box(result)
        });
    });

    let pre_hashed_clone2 = pre_hashed.clone();
    group.bench_function("verify_password_failure", |b| {
        b.iter(|| {
            let result = auth_service
                .verify_password(black_box("wrong_password"), black_box(&pre_hashed_clone2))
                .unwrap();
            black_box(result)
        });
    });

    group.finish();
}

fn jwt_benchmark(c: &mut Criterion) {
    let jwt_config =
        JwtConfig::new("super_secret_key_for_benchmarking_purposes_only_12345".to_string());
    let jwt_manager = JwtManager::new(jwt_config).unwrap();

    let mut group = c.benchmark_group("jwt_operations");

    // Use StandardClaims which is the proper type
    let claims = StandardClaims::new()
        .with_subject("user123".to_string())
        .with_expiration(3600);

    group.bench_function("sign_token", |b| {
        let claims = claims.clone();
        b.iter(|| {
            let token = jwt_manager.sign(black_box(&claims)).unwrap();
            black_box(token)
        });
    });

    // Pre-create a token for verification benchmarks
    let pre_created_token = jwt_manager.sign(&claims).unwrap();

    group.bench_function("verify_token", |b| {
        b.iter(|| {
            let result: StandardClaims = jwt_manager.verify(black_box(&pre_created_token)).unwrap();
            black_box(result)
        });
    });

    group.bench_function("sign_and_verify", |b| {
        let claims = claims.clone();
        b.iter(|| {
            let token = jwt_manager.sign(black_box(&claims)).unwrap();
            let result: StandardClaims = jwt_manager.verify(&token).unwrap();
            black_box(result)
        });
    });

    group.finish();
}

fn password_hasher_direct_benchmark(c: &mut Criterion) {
    let hasher = PasswordHasher::default();

    let mut group = c.benchmark_group("password_hasher_direct");

    // Different password lengths
    let passwords = [
        ("short", "pass123"),
        ("medium", "my_medium_password_123"),
        (
            "long",
            "this_is_a_very_long_password_that_someone_might_use_for_security_reasons_123456",
        ),
    ];

    for (name, password) in passwords {
        group.bench_function(format!("hash_{}", name), |b| {
            b.iter(|| {
                let hash = hasher.hash(black_box(password)).unwrap();
                black_box(hash)
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    password_hashing_benchmark,
    jwt_benchmark,
    password_hasher_direct_benchmark,
);

criterion_main!(benches);
