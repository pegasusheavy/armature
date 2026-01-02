#![allow(clippy::all)]
#![allow(clippy::needless_question_mark)]
//! Load Testing Example
//!
//! Demonstrates performance testing and load generation.

use armature_testing::load::*;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Load Testing Example ===\n");

    // 1. Basic load test
    println!("1. Basic Load Test (10 concurrent, 100 requests):");
    println!("   Starting load test...");

    let basic_config = LoadTestConfig::new(10, 100).with_timeout(Duration::from_secs(5));

    let basic_runner = LoadTestRunner::new(basic_config, || async {
        // Simulate API call
        tokio::time::sleep(Duration::from_millis(50)).await;
        Ok(())
    });

    let stats = basic_runner.run().await?;
    stats.print();

    // 2. Duration-based load test
    println!("\n2. Duration-Based Load Test (5 concurrent, 3 seconds):");
    println!("   Starting duration-based test...");

    let duration_config = LoadTestConfig::new(5, u64::MAX)
        .with_duration(Duration::from_secs(3))
        .with_timeout(Duration::from_secs(5));

    let duration_runner = LoadTestRunner::new(duration_config, || async {
        tokio::time::sleep(Duration::from_millis(30)).await;
        Ok(())
    });

    let stats = duration_runner.run().await?;
    stats.print();

    // 3. Load test with failures
    println!("\n3. Load Test with Some Failures:");
    println!("   Starting load test with 20% failure rate...");

    let failure_count = Arc::new(AtomicU32::new(0));
    let failure_config = LoadTestConfig::new(5, 50).with_timeout(Duration::from_secs(5));

    let failure_count_clone = failure_count.clone();
    let failure_runner = LoadTestRunner::new(failure_config, move || {
        let count = failure_count_clone.clone();
        async move {
            let current = count.fetch_add(1, Ordering::SeqCst);
            tokio::time::sleep(Duration::from_millis(40)).await;

            // Fail 20% of requests
            if current % 5 == 0 {
                Err(LoadTestError::TestFailed("Simulated failure".to_string()))
            } else {
                Ok(())
            }
        }
    });

    let stats = failure_runner.run().await?;
    stats.print();

    // 4. Stress test (gradually increasing load)
    println!("\n4. Stress Test (1 → 20 concurrent, step by 5, 2 seconds per step):");
    println!("   Starting stress test...");

    let stress_runner = StressTestRunner::new(
        1,                      // Initial concurrency
        20,                     // Max concurrency
        5,                      // Step size
        Duration::from_secs(2), // Duration per step
        || async {
            tokio::time::sleep(Duration::from_millis(30)).await;
            Ok(())
        },
    );

    let stress_results = stress_runner.run().await?;

    // Print stress test summary
    println!("\nStress Test Summary:");
    println!("┌─────────────┬──────────┬────────────┬────────────┐");
    println!("│ Concurrency │ RPS      │ Avg (ms)   │ p95 (ms)   │");
    println!("├─────────────┼──────────┼────────────┼────────────┤");
    for (concurrency, stats) in stress_results {
        println!(
            "│ {:11} │ {:8.2} │ {:10.2} │ {:10.2} │",
            concurrency,
            stats.rps,
            stats.avg_response_time.as_millis(),
            stats.p95_response_time.as_millis()
        );
    }
    println!("└─────────────┴──────────┴────────────┴────────────┘");

    // 5. Real-world example: Testing an API endpoint
    println!("\n5. Real-World Example: API Endpoint Load Test");
    println!("   (Simulated - replace with actual HTTP client)");

    let api_config = LoadTestConfig::new(20, 200).with_timeout(Duration::from_secs(10));

    let api_runner = LoadTestRunner::new(api_config, || async {
        // In a real scenario, you would use reqwest or similar:
        // let response = reqwest::get("http://localhost:3000/api/users").await?;
        // if !response.status().is_success() {
        //     return Err(LoadTestError::TestFailed("Request failed".to_string()));
        // }

        // Simulated API call
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(())
    });

    let stats = api_runner.run().await?;
    stats.print();

    println!("=== Load Testing Complete ===\n");
    println!("💡 Tips:");
    println!("   - Use LoadTestRunner for fixed request counts");
    println!("   - Use duration-based tests for sustained load");
    println!("   - Use StressTestRunner to find breaking points");
    println!("   - Monitor p95/p99 latencies for SLA compliance");
    println!();

    Ok(())
}
