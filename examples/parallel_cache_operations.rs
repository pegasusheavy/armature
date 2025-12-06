//! Parallel Cache Operations Example
//!
//! Demonstrates the performance benefits of batch cache operations
//! compared to sequential operations.

use armature_cache::*;
use std::time::{Duration, Instant};
use tokio;

#[tokio::main]
async fn main() -> Result<(), CacheError> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                                                              ║");
    println!("║        Parallel Cache Operations Performance Demo           ║");
    println!("║                                                              ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Create Redis cache
    let config = CacheConfig::redis("redis://localhost:6379")?;
    let cache = RedisCache::new(config).await?;

    println!("✅ Connected to Redis\n");

    // ========================================================================
    // 1. SEQUENTIAL VS PARALLEL GET
    // ========================================================================

    println!("═══════════════════════════════════════════════════════════════");
    println!("          TEST 1: Sequential vs Parallel GET                   ");
    println!("═══════════════════════════════════════════════════════════════\n");

    // Prepare test data
    let num_keys = 100;
    println!("📝 Setting up {} test keys...", num_keys);

    for i in 1..=num_keys {
        let key = format!("test:user:{}", i);
        let value = serde_json::json!({
            "id": i,
            "name": format!("User {}", i),
            "email": format!("user{}@example.com", i)
        });
        cache.set_json(&key, value.to_string(), Some(Duration::from_secs(60))).await?;
    }
    println!("   ✅ Test data ready\n");

    // Sequential GET
    println!("🐌 Sequential GET (one at a time)...");
    let start = Instant::now();

    let mut results = Vec::new();
    for i in 1..=num_keys {
        let key = format!("test:user:{}", i);
        if let Some(value) = cache.get_json(&key).await? {
            results.push(value);
        }
    }

    let sequential_time = start.elapsed();
    println!("   Time taken: {:?}", sequential_time);
    println!("   Results: {} values fetched", results.len());

    // Parallel GET (using get_many)
    println!("\n⚡ Parallel GET (all at once)...");
    let keys: Vec<String> = (1..=num_keys).map(|i| format!("test:user:{}", i)).collect();
    let key_refs: Vec<&str> = keys.iter().map(|s| s.as_str()).collect();

    let start = Instant::now();
    let parallel_results = cache.get_many(&key_refs).await?;
    let parallel_time = start.elapsed();

    println!("   Time taken: {:?}", parallel_time);
    println!("   Results: {} values fetched", parallel_results.iter().filter(|r| r.is_some()).count());

    let speedup = sequential_time.as_millis() as f64 / parallel_time.as_millis().max(1) as f64;
    println!("\n   🚀 Speedup: {:.1}x faster!", speedup);

    // ========================================================================
    // 2. SEQUENTIAL VS PARALLEL SET
    // ========================================================================

    println!("\n═══════════════════════════════════════════════════════════════");
    println!("          TEST 2: Sequential vs Parallel SET                   ");
    println!("═══════════════════════════════════════════════════════════════\n");

    let num_writes = 50;

    // Sequential SET
    println!("🐌 Sequential SET...");
    let start = Instant::now();

    for i in 1..=num_writes {
        let key = format!("test:product:{}", i);
        let value = serde_json::json!({
            "id": i,
            "name": format!("Product {}", i),
            "price": i * 10
        });
        cache.set_json(&key, value.to_string(), Some(Duration::from_secs(60))).await?;
    }

    let sequential_set_time = start.elapsed();
    println!("   Time taken: {:?}", sequential_set_time);

    // Parallel SET (using set_many)
    println!("\n⚡ Parallel SET...");

    let items: Vec<(String, String)> = (1..=num_writes)
        .map(|i| {
            let key = format!("test:order:{}", i);
            let value = serde_json::json!({
                "id": i,
                "order_number": format!("ORD-{:05}", i),
                "total": i * 25
            }).to_string();
            (key, value)
        })
        .collect();

    let item_refs: Vec<(&str, String)> = items.iter()
        .map(|(k, v)| (k.as_str(), v.clone()))
        .collect();

    let start = Instant::now();
    cache.set_many(&item_refs, Some(Duration::from_secs(60))).await?;
    let parallel_set_time = start.elapsed();

    println!("   Time taken: {:?}", parallel_set_time);

    let speedup = sequential_set_time.as_millis() as f64 / parallel_set_time.as_millis().max(1) as f64;
    println!("\n   🚀 Speedup: {:.1}x faster!", speedup);

    // ========================================================================
    // 3. SEQUENTIAL VS PARALLEL DELETE
    // ========================================================================

    println!("\n═══════════════════════════════════════════════════════════════");
    println!("          TEST 3: Sequential vs Parallel DELETE                ");
    println!("═══════════════════════════════════════════════════════════════\n");

    let num_deletes = 50;

    // Sequential DELETE
    println!("🐌 Sequential DELETE...");
    let start = Instant::now();

    for i in 1..=num_deletes {
        let key = format!("test:user:{}", i);
        cache.delete(&key).await?;
    }

    let sequential_del_time = start.elapsed();
    println!("   Time taken: {:?}", sequential_del_time);

    // Parallel DELETE (using delete_many)
    println!("\n⚡ Parallel DELETE...");

    let delete_keys: Vec<String> = (1..=num_deletes)
        .map(|i| format!("test:product:{}", i))
        .collect();
    let delete_key_refs: Vec<&str> = delete_keys.iter().map(|s| s.as_str()).collect();

    let start = Instant::now();
    cache.delete_many(&delete_key_refs).await?;
    let parallel_del_time = start.elapsed();

    println!("   Time taken: {:?}", parallel_del_time);

    let speedup = sequential_del_time.as_millis() as f64 / parallel_del_time.as_millis().max(1) as f64;
    println!("\n   🚀 Speedup: {:.1}x faster!", speedup);

    // ========================================================================
    // 4. PARALLEL EXISTS CHECK
    // ========================================================================

    println!("\n═══════════════════════════════════════════════════════════════");
    println!("          TEST 4: Parallel EXISTS Check                        ");
    println!("═══════════════════════════════════════════════════════════════\n");

    let check_keys: Vec<String> = (1..=30)
        .map(|i| format!("test:order:{}", i))
        .collect();
    let check_key_refs: Vec<&str> = check_keys.iter().map(|s| s.as_str()).collect();

    println!("🔍 Checking existence of {} keys in parallel...", check_keys.len());
    let start = Instant::now();

    let exists_results = cache.exists_many(&check_key_refs).await?;
    let exists_time = start.elapsed();

    let existing_count = exists_results.iter().filter(|&&e| e).count();

    println!("   Time taken: {:?}", exists_time);
    println!("   Found: {}/{} keys exist", existing_count, check_keys.len());

    // ========================================================================
    // 5. REAL-WORLD USE CASE: CACHE WARMING
    // ========================================================================

    println!("\n═══════════════════════════════════════════════════════════════");
    println!("          USE CASE: Cache Warming                              ");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("📊 Simulating cache warming scenario...");
    println!("   Loading 100 user profiles at application startup\n");

    // Simulate fetching from database
    let user_data: Vec<(String, String)> = (1..=100)
        .map(|i| {
            let key = format!("warm:user:{}", i);
            let value = serde_json::json!({
                "id": i,
                "name": format!("User {}", i),
                "email": format!("user{}@example.com", i),
                "preferences": {
                    "theme": "dark",
                    "notifications": true
                }
            }).to_string();
            (key, value)
        })
        .collect();

    let user_refs: Vec<(&str, String)> = user_data.iter()
        .map(|(k, v)| (k.as_str(), v.clone()))
        .collect();

    println!("⚡ Warming cache with set_many...");
    let start = Instant::now();

    cache.set_many(&user_refs, Some(Duration::from_secs(3600))).await?;

    let warm_time = start.elapsed();
    println!("   ✅ Cached 100 profiles in {:?}", warm_time);
    println!("   Rate: {:.1} writes/sec", 100.0 / warm_time.as_secs_f64());

    // ========================================================================
    // 6. PERFORMANCE SUMMARY
    // ========================================================================

    println!("\n═══════════════════════════════════════════════════════════════");
    println!("                   PERFORMANCE SUMMARY                         ");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("┌──────────────────────┬─────────────┬─────────────┬──────────┐");
    println!("│ Operation            │ Sequential  │ Parallel    │ Speedup  │");
    println!("├──────────────────────┼─────────────┼─────────────┼──────────┤");
    println!("│ GET ({} keys)      │ {:>9.0}ms │ {:>9.0}ms │ {:>6.1}x │",
        num_keys,
        sequential_time.as_millis(),
        parallel_time.as_millis(),
        speedup
    );

    let set_speedup = sequential_set_time.as_millis() as f64 / parallel_set_time.as_millis().max(1) as f64;
    println!("│ SET ({} keys)      │ {:>9.0}ms │ {:>9.0}ms │ {:>6.1}x │",
        num_writes,
        sequential_set_time.as_millis(),
        parallel_set_time.as_millis(),
        set_speedup
    );

    let del_speedup = sequential_del_time.as_millis() as f64 / parallel_del_time.as_millis().max(1) as f64;
    println!("│ DELETE ({} keys)   │ {:>9.0}ms │ {:>9.0}ms │ {:>6.1}x │",
        num_deletes,
        sequential_del_time.as_millis(),
        parallel_del_time.as_millis(),
        del_speedup
    );
    println!("└──────────────────────┴─────────────┴─────────────┴──────────┘\n");

    println!("🎯 Key Takeaways:");
    println!("   • Parallel operations reduce total latency to ~max(latencies)");
    println!("   • Essential for cache warming and bulk operations");
    println!("   • 10-100x faster for network-bound operations");
    println!("   • No code complexity increase - simple API");

    // Cleanup
    println!("\n🧹 Cleaning up test data...");
    cache.clear().await?;
    println!("   ✅ Test data cleared\n");

    println!("═══════════════════════════════════════════════════════════════");
    println!("✅ Parallel cache operations demo complete!");
    println!("═══════════════════════════════════════════════════════════════\n");

    Ok(())
}

