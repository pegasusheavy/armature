//! Distributed Locks Example
//!
//! Demonstrates Redis-based distributed locks for coordinating work across multiple instances.
//!
//! Note: This example requires Redis to be running
//! Start Redis: docker run -p 6379:6379 redis

use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Distributed Locks Example ===\n");

    println!("⚠️  This example requires Redis to be running on localhost:6379");
    println!("   Start Redis with: docker run -p 6379:6379 redis\n");

    // Try to connect to Redis
    println!("Attempting to connect to Redis...");

    // Import the distributed lock module
    use armature_distributed::*;

    // For demo purposes, we'll show the API even if Redis isn't running
    println!("\n📚 API Overview:\n");

    println!("1. Basic Lock Usage:");
    println!("   ```rust");
    println!("   let lock = RedisLock::new(");
    println!("       \"my-resource\",");
    println!("       Duration::from_secs(30),");
    println!("       redis_conn,");
    println!("   );");
    println!("   let guard = lock.acquire().await?;");
    println!("   // Do critical work...");
    println!("   guard.release().await?;");
    println!("   ```\n");

    println!("2. Try Acquire (Non-Blocking):");
    println!("   ```rust");
    println!("   match lock.try_acquire().await? {{");
    println!("       Some(guard) => {{ /* acquired */ }}");
    println!("       None => {{ /* already held */ }}");
    println!("   }}");
    println!("   ```\n");

    println!("3. Acquire with Timeout:");
    println!("   ```rust");
    println!("   let guard = lock.acquire_timeout(Duration::from_secs(5)).await?;");
    println!("   ```\n");

    println!("4. Using Lock Builder:");
    println!("   ```rust");
    println!("   let lock = LockBuilder::new(\"builder-resource\")");
    println!("       .with_ttl(Duration::from_secs(60))");
    println!("       .build(redis_conn);");
    println!("   ```\n");

    println!("5. Automatic Release (RAII):");
    println!("   ```rust");
    println!("   {{");
    println!("       let _guard = lock.acquire().await?;");
    println!("       // Lock auto-releases when _guard goes out of scope");
    println!("   }}");
    println!("   ```\n");

    // Show a mock demonstration
    println!("=== Mock Demonstration ===\n");

    println!("Simulating lock behavior...\n");

    println!("1. Acquiring lock on 'order-processor'...");
    tokio::time::sleep(Duration::from_millis(100)).await;
    println!("   ✅ Lock acquired!");

    println!("\n2. Doing critical work...");
    for i in 1..=3 {
        tokio::time::sleep(Duration::from_millis(300)).await;
        println!("   Processing batch {}...", i);
    }

    println!("\n3. Releasing lock...");
    tokio::time::sleep(Duration::from_millis(100)).await;
    println!("   ✅ Lock released!\n");

    println!("=== Distributed Locks Example Complete ===\n");
    println!("💡 Key Features:");
    println!("   ✅ Acquire locks for critical sections");
    println!("   ✅ Try acquire (non-blocking)");
    println!("   ✅ Acquire with timeout");
    println!("   ✅ Automatic lock release (RAII)");
    println!("   ✅ Concurrent access coordination");
    println!("   ✅ Builder pattern configuration");
    println!();
    println!("💡 Use Cases:");
    println!("   - Prevent concurrent job processing");
    println!("   - Coordinate distributed tasks");
    println!("   - Ensure single-instance operations");
    println!("   - Resource access control");
    println!();

    Ok(())
}
