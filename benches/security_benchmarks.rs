use criterion::{black_box, criterion_group, criterion_main, Criterion};
use armature_csrf::*;
use armature_xss::*;
use armature_jwt::*;
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
        b.iter(|| {
            manager.sign(black_box(&claims)).unwrap()
        })
    });

    let token = manager.sign(&claims).unwrap();

    group.bench_function("verify", |b| {
        b.iter(|| {
            let _: Claims<BenchClaims> = manager.verify(black_box(&token)).unwrap();
        })
    });

    // Test different algorithms
    for algo in [Algorithm::HS256, Algorithm::HS384, Algorithm::HS512].iter() {
        let config = JwtConfig::new("test_secret_key_32_bytes_long!!!".to_string())
            .with_algorithm(*algo);
        let manager = JwtManager::new(config).unwrap();

        group.bench_function(&format!("sign_{:?}", algo), |b| {
            b.iter(|| manager.sign(black_box(&claims)).unwrap())
        });
    }

    group.finish();
}

fn bench_csrf_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("csrf");

    group.bench_function("token_generate", |b| {
        b.iter(|| CsrfToken::generate(black_box(3600)))
    });

    let secret = CsrfConfig::generate_secret();
    let token = CsrfToken::generate(3600);

    group.bench_function("token_encode", |b| {
        b.iter(|| token.encode(black_box(&secret)).unwrap())
    });

    let encoded = token.encode(&secret).unwrap();

    group.bench_function("token_decode", |b| {
        b.iter(|| CsrfToken::decode(black_box(&encoded), black_box(&secret)).unwrap())
    });

    group.bench_function("config_generate_secret", |b| {
        b.iter(|| CsrfConfig::generate_secret())
    });

    group.finish();
}

fn bench_xss_sanitization(c: &mut Criterion) {
    let mut group = c.benchmark_group("xss_sanitization");

    let sanitizer_default = XssSanitizer::new();
    let sanitizer_strict = XssSanitizer::strict();
    let sanitizer_permissive = XssSanitizer::permissive();

    let html_simple = r#"<p>Hello <strong>World</strong>!</p>"#;
    let html_with_script = r#"<p>Hello</p><script>alert('XSS')</script><p>World</p>"#;
    let html_complex = r#"
        <div class="container">
            <h1>Title</h1>
            <p>Paragraph with <a href="http://example.com">link</a></p>
            <img src="image.jpg" alt="Image" />
            <script>alert('XSS')</script>
            <ul>
                <li>Item 1</li>
                <li>Item 2</li>
            </ul>
        </div>
    "#;

    group.bench_function("sanitize_simple", |b| {
        b.iter(|| sanitizer_default.sanitize(black_box(html_simple)).unwrap())
    });

    group.bench_function("sanitize_with_script", |b| {
        b.iter(|| sanitizer_default.sanitize(black_box(html_with_script)).unwrap())
    });

    group.bench_function("sanitize_complex", |b| {
        b.iter(|| sanitizer_default.sanitize(black_box(html_complex)).unwrap())
    });

    group.bench_function("sanitize_strict", |b| {
        b.iter(|| sanitizer_strict.sanitize(black_box(html_complex)).unwrap())
    });

    group.bench_function("sanitize_permissive", |b| {
        b.iter(|| sanitizer_permissive.sanitize(black_box(html_complex)).unwrap())
    });

    group.finish();
}

fn bench_xss_encoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("xss_encoding");

    let text = r#"<script>alert("XSS")</script>"#;
    let url = "hello world&test=value";
    let js = r#"'; alert('XSS'); //'"#;

    group.bench_function("encode_html", |b| {
        b.iter(|| XssEncoder::encode_html(black_box(text)))
    });

    group.bench_function("encode_url", |b| {
        b.iter(|| XssEncoder::encode_url(black_box(url)))
    });

    group.bench_function("encode_javascript", |b| {
        b.iter(|| XssEncoder::encode_javascript(black_box(js)))
    });

    let encoded = XssEncoder::encode_html(text);

    group.bench_function("decode_html", |b| {
        b.iter(|| XssEncoder::decode_html(black_box(&encoded)))
    });

    group.finish();
}

fn bench_xss_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("xss_validation");

    let safe = r#"<p>Hello <strong>world</strong>!</p>"#;
    let xss_script = r#"<script>alert('XSS')</script>"#;
    let xss_onerror = r#"<img src="x" onerror="alert('XSS')">"#;
    let xss_javascript = r#"<a href="javascript:alert('XSS')">Click</a>"#;

    group.bench_function("validate_safe", |b| {
        b.iter(|| XssValidator::contains_xss(black_box(safe)))
    });

    group.bench_function("validate_script", |b| {
        b.iter(|| XssValidator::contains_xss(black_box(xss_script)))
    });

    group.bench_function("validate_onerror", |b| {
        b.iter(|| XssValidator::contains_xss(black_box(xss_onerror)))
    });

    group.bench_function("detect_attack_type", |b| {
        b.iter(|| XssValidator::detect_attack_type(black_box(xss_javascript)))
    });

    group.finish();
}

criterion_group!(
    security_benches,
    bench_jwt_operations,
    bench_csrf_operations,
    bench_xss_sanitization,
    bench_xss_encoding,
    bench_xss_validation,
);

criterion_main!(security_benches);

