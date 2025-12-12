#![allow(dead_code)]
//! Rate Limiting Example
//!
//! This example demonstrates how to use the armature-ratelimit crate
//! to add rate limiting to your application.
//!
//! Run with: cargo run --example rate_limiting --features ratelimit

use armature_ratelimit::{
    Algorithm, KeyExtractor, RateLimitMiddleware, RateLimiter, extractor::RequestInfo,
    middleware::RateLimitCheckResponse,
};
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Rate Limiting Examples ===\n");

    // Example 1: Token Bucket Algorithm
    token_bucket_example().await?;

    // Example 2: Sliding Window Algorithm
    sliding_window_example().await?;

    // Example 3: Fixed Window Algorithm
    fixed_window_example().await?;

    // Example 4: Middleware with Key Extraction
    middleware_example().await?;

    // Example 5: Different Keys for Different Clients
    multi_client_example().await?;

    // Example 6: Bypass Keys
    bypass_example().await?;

    println!("\n=== All Examples Complete ===");
    Ok(())
}

/// Example 1: Token Bucket Algorithm
async fn token_bucket_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Token Bucket Example ---");
    println!("Capacity: 5 tokens, Refill rate: 2 tokens/second\n");

    let limiter = RateLimiter::builder()
        .algorithm(Algorithm::TokenBucket {
            capacity: 5,
            refill_rate: 2.0,
        })
        .build()
        .await?;

    // Make 7 requests
    for i in 1..=7 {
        let result = limiter.check("user_1").await?;
        println!(
            "Request {}: {} (remaining: {})",
            i,
            if result.allowed { "ALLOWED" } else { "DENIED" },
            result.remaining
        );
    }

    println!("\nWaiting 2 seconds for token refill...");
    tokio::time::sleep(Duration::from_secs(2)).await;

    // After refill
    let result = limiter.check("user_1").await?;
    println!(
        "After refill: {} (remaining: {})\n",
        if result.allowed { "ALLOWED" } else { "DENIED" },
        result.remaining
    );

    Ok(())
}

/// Example 2: Sliding Window Algorithm
async fn sliding_window_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Sliding Window Example ---");
    println!("Max requests: 3 per 5 seconds\n");

    let limiter = RateLimiter::builder()
        .sliding_window(3, Duration::from_secs(5))
        .build()
        .await?;

    // Make 5 requests
    for i in 1..=5 {
        let result = limiter.check("user_2").await?;
        println!(
            "Request {}: {} (remaining: {})",
            i,
            if result.allowed { "ALLOWED" } else { "DENIED" },
            result.remaining
        );
    }

    println!("\nWaiting 5 seconds for window to expire...");
    tokio::time::sleep(Duration::from_secs(5)).await;

    let result = limiter.check("user_2").await?;
    println!(
        "After window expires: {} (remaining: {})\n",
        if result.allowed { "ALLOWED" } else { "DENIED" },
        result.remaining
    );

    Ok(())
}

/// Example 3: Fixed Window Algorithm
async fn fixed_window_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Fixed Window Example ---");
    println!("Max requests: 3 per window\n");

    let limiter = RateLimiter::builder()
        .fixed_window(3, Duration::from_secs(60))
        .build()
        .await?;

    // Make 5 requests
    for i in 1..=5 {
        let result = limiter.check("user_3").await?;
        println!(
            "Request {}: {} (remaining: {})",
            i,
            if result.allowed { "ALLOWED" } else { "DENIED" },
            result.remaining
        );
    }

    println!();
    Ok(())
}

/// Example 4: Middleware with Key Extraction
async fn middleware_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Middleware Example ---");
    println!("Using IP-based key extraction\n");

    let limiter = Arc::new(RateLimiter::builder().token_bucket(3, 1.0).build().await?);

    let middleware = RateLimitMiddleware::new(limiter)
        .with_extractor(KeyExtractor::Ip)
        .with_headers(true)
        .with_error_message("Too many requests! Please slow down.");

    // Simulate requests from the same IP
    let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100));

    for i in 1..=5 {
        let info = RequestInfo::new("/api/users", "GET").with_ip(ip);

        let response = middleware.check(&info).await;

        match &response {
            RateLimitCheckResponse::Allowed { headers } => {
                if let Some(h) = headers {
                    let pairs = h.to_header_pairs();
                    let limit = pairs
                        .iter()
                        .find(|(k, _)| *k == "X-RateLimit-Limit")
                        .map(|(_, v)| v.as_str())
                        .unwrap_or("?");
                    let remaining = pairs
                        .iter()
                        .find(|(k, _)| *k == "X-RateLimit-Remaining")
                        .map(|(_, v)| v.as_str())
                        .unwrap_or("?");
                    let reset = pairs
                        .iter()
                        .find(|(k, _)| *k == "X-RateLimit-Reset")
                        .map(|(_, v)| v.as_str())
                        .unwrap_or("?");
                    println!(
                        "Request {}: ALLOWED - Limit: {}, Remaining: {}, Reset: {}",
                        i, limit, remaining, reset
                    );
                } else {
                    println!("Request {}: ALLOWED", i);
                }
            }
            RateLimitCheckResponse::Limited {
                message,
                retry_after,
                ..
            } => {
                println!(
                    "Request {}: DENIED - {} (retry after: {:?}s)",
                    i, message, retry_after
                );
            }
        }
    }

    println!();
    Ok(())
}

/// Example 5: Different Keys for Different Clients
async fn multi_client_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Multi-Client Example ---");
    println!("Each client has their own rate limit\n");

    let limiter = RateLimiter::builder().token_bucket(2, 0.5).build().await?;

    let clients = ["client_a", "client_b", "client_c"];

    for client in &clients {
        println!("{}:", client);
        for i in 1..=3 {
            let result = limiter.check(client).await?;
            println!(
                "  Request {}: {}",
                i,
                if result.allowed { "ALLOWED" } else { "DENIED" }
            );
        }
    }

    println!();
    Ok(())
}

/// Example 6: Bypass Keys
async fn bypass_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Bypass Keys Example ---");
    println!("Admin users bypass rate limiting\n");

    let limiter = Arc::new(
        RateLimiter::builder()
            .token_bucket(2, 0.1)
            .bypass_key("admin_key")
            .build()
            .await?,
    );

    let middleware = RateLimitMiddleware::new(limiter)
        .with_extractor(KeyExtractor::ApiKey {
            header_name: "X-API-Key".to_string(),
        })
        .with_bypass_keys(["admin_key"]);

    // Regular user gets rate limited
    println!("Regular user (user_key):");
    for i in 1..=4 {
        let info = RequestInfo::new("/api/data", "GET").with_header("X-API-Key", "user_key");

        let response = middleware.check(&info).await;
        println!(
            "  Request {}: {}",
            i,
            if response.is_allowed() {
                "ALLOWED"
            } else {
                "DENIED"
            }
        );
    }

    // Admin user bypasses rate limit
    println!("\nAdmin user (admin_key):");
    for i in 1..=4 {
        let info = RequestInfo::new("/api/data", "GET").with_header("X-API-Key", "admin_key");

        let response = middleware.check(&info).await;
        println!(
            "  Request {}: {}",
            i,
            if response.is_allowed() {
                "ALLOWED"
            } else {
                "DENIED"
            }
        );
    }

    println!();
    Ok(())
}
