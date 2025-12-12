//! Example: HTTP Response Compression
//!
//! This example demonstrates how to use the compression middleware
//! to automatically compress HTTP responses.
//!
//! Run with: `cargo run --example compression_example --features compression`

#![allow(dead_code, unused_imports)]

use armature::prelude::*;
use armature_compression::{CompressionAlgorithm, CompressionConfig, CompressionMiddleware};

// Sample large JSON response
fn large_json_response() -> HttpResponse {
    let items: Vec<serde_json::Value> = (0..100)
        .map(|i| {
            serde_json::json!({
                "id": i,
                "name": format!("Item {}", i),
                "description": "This is a sample item with a long description that will compress well due to repetitive text patterns.",
                "tags": ["rust", "web", "compression", "example"],
                "metadata": {
                    "created_at": "2024-01-01T00:00:00Z",
                    "updated_at": "2024-01-01T00:00:00Z",
                    "version": 1
                }
            })
        })
        .collect();

    let body = serde_json::to_string(&items).unwrap();

    HttpResponse::ok()
        .with_header("Content-Type".to_string(), "application/json".to_string())
        .with_body(body.into_bytes())
}

fn main() {
    println!("🗜️  Armature Compression Middleware Example");
    println!("==========================================\n");

    // Create compression middleware with different configurations
    demonstrate_default_compression();
    demonstrate_gzip_compression();
    demonstrate_brotli_compression();
    demonstrate_algorithm_selection();
}

fn demonstrate_default_compression() {
    println!("1️⃣  Default Compression (Auto-select)");
    println!("------------------------------------");

    // Default: auto-select best algorithm based on Accept-Encoding
    let middleware = CompressionMiddleware::new();
    println!("   Algorithm: {:?}", middleware.config().algorithm);
    println!("   Min size: {} bytes", middleware.config().min_size);
    println!();
}

fn demonstrate_gzip_compression() {
    println!("2️⃣  Gzip Compression");
    println!("--------------------");

    let config = CompressionConfig::builder()
        .gzip()
        .level(6) // Balanced compression
        .min_size(100) // Compress responses > 100 bytes
        .build();

    let _middleware = CompressionMiddleware::with_config(config);

    // Simulate compressing a response
    let response = large_json_response();
    let original_size = response.body.len();

    // Note: In real usage, the middleware handles this in the request pipeline
    let compressed = CompressionAlgorithm::Gzip
        .compress(&response.body, 6)
        .unwrap();
    let compressed_size = compressed.len();

    println!("   Original size: {} bytes", original_size);
    println!("   Compressed size: {} bytes", compressed_size);
    println!(
        "   Compression ratio: {:.1}%",
        (1.0 - compressed_size as f64 / original_size as f64) * 100.0
    );
    println!();
}

fn demonstrate_brotli_compression() {
    println!("3️⃣  Brotli Compression");
    println!("----------------------");

    let config = CompressionConfig::builder()
        .brotli()
        .level(4) // Good balance of speed and compression
        .build();

    let _middleware = CompressionMiddleware::with_config(config);

    let response = large_json_response();
    let original_size = response.body.len();

    let compressed = CompressionAlgorithm::Brotli
        .compress(&response.body, 4)
        .unwrap();
    let compressed_size = compressed.len();

    println!("   Original size: {} bytes", original_size);
    println!("   Compressed size: {} bytes", compressed_size);
    println!(
        "   Compression ratio: {:.1}%",
        (1.0 - compressed_size as f64 / original_size as f64) * 100.0
    );
    println!();
}

fn demonstrate_algorithm_selection() {
    println!("4️⃣  Algorithm Selection from Accept-Encoding");
    println!("--------------------------------------------");

    let test_cases = [
        "gzip, deflate",
        "br, gzip, deflate",
        "zstd, br, gzip",
        "gzip",
        "deflate, compress",
    ];

    for accept_encoding in test_cases {
        let algo = CompressionAlgorithm::select_from_accept_encoding(accept_encoding);
        println!("   Accept-Encoding: {:20} → {:?}", accept_encoding, algo);
    }
    println!();

    println!("✅ Compression middleware is ready for use!");
    println!();
    println!("Usage in your application:");
    println!("```rust");
    println!("let mut chain = MiddlewareChain::new();");
    println!("chain.use_middleware(CompressionMiddleware::new());");
    println!("```");
}
