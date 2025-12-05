//! Cron job scheduling example.
//!
//! This example demonstrates how to use the cron scheduler to run periodic tasks.

use armature_cron::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::time::Duration;

#[tokio::main]
async fn main() -> Result<(), CronError> {
    println!("🕐 Armature Cron Scheduler Example\n");

    // Create a scheduler with custom configuration
    let config = SchedulerConfig {
        tick_interval: Duration::from_secs(1),
        run_missed_jobs: false,
        max_concurrent_jobs: 5,
        log_execution: true,
    };

    let mut scheduler = CronScheduler::with_config(config);

    // Example 1: Simple job that runs every 5 seconds
    scheduler.add_job("heartbeat", "*/5 * * * * *", |ctx| {
        Box::pin(async move {
            println!("💓 Heartbeat - Execution #{}", ctx.execution_count + 1);
            Ok(())
        })
    })?;

    // Example 2: Job with shared state
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    scheduler.add_job("counter", "*/3 * * * * *", move |ctx| {
        let counter = counter_clone.clone();
        Box::pin(async move {
            let value = counter.fetch_add(1, Ordering::SeqCst);
            println!("🔢 Counter job - Count: {}", value + 1);
            Ok(())
        })
    })?;

    // Example 3: Job that simulates data cleanup
    scheduler.add_job("cleanup", "*/10 * * * * *", |ctx| {
        Box::pin(async move {
            println!("🧹 Cleanup job starting...");
            tokio::time::sleep(Duration::from_millis(500)).await;
            println!("✨ Cleanup completed!");
            Ok(())
        })
    })?;

    // Example 4: Job that can fail
    let mut fail_count = 0;
    scheduler.add_job("flaky_job", "*/7 * * * * *", move |ctx| {
        fail_count += 1;
        Box::pin(async move {
            if fail_count % 2 == 0 {
                println!("❌ Flaky job failed!");
                Err(CronError::ExecutionFailed("Simulated failure".to_string()))
            } else {
                println!("✅ Flaky job succeeded!");
                Ok(())
            }
        })
    })?;

    // Example 5: Using cron presets
    scheduler.add_job("report", CronPresets::EVERY_MINUTE, |ctx| {
        Box::pin(async move {
            println!("📊 Generating minute report...");
            println!("   Scheduled: {}", ctx.scheduled_time.format("%H:%M:%S"));
            println!("   Executed:  {}", ctx.execution_time.format("%H:%M:%S"));
            println!("   Delay:     {:?}", ctx.delay());
            Ok(())
        })
    })?;

    // Wait a moment for jobs to be added
    tokio::time::sleep(Duration::from_millis(100)).await;

    // List all jobs
    println!("\n📋 Registered Jobs:");
    for job_name in scheduler.list_jobs().await {
        println!("   - {}", job_name);
    }

    println!("\n▶️  Starting scheduler...\n");

    // Start the scheduler
    scheduler.start().await?;

    // Let it run for 30 seconds
    println!("⏱️  Running for 30 seconds...\n");
    tokio::time::sleep(Duration::from_secs(30)).await;

    // Get statistics for a job
    println!("\n📈 Job Statistics:");
    if let Ok(stats) = scheduler.get_stats("heartbeat").await {
        println!("   Job: {}", stats.name);
        println!("   Enabled: {}", stats.enabled);
        println!("   Executions: {}", stats.execution_count);
        if let Some(last_run) = stats.last_run {
            println!("   Last run: {}", last_run.format("%H:%M:%S"));
        }
        if let Some(next_run) = stats.next_run {
            println!("   Next run: {}", next_run.format("%H:%M:%S"));
        }
    }

    // Demonstrate disabling a job
    println!("\n⏸️  Disabling 'heartbeat' job...");
    scheduler.disable_job("heartbeat").await?;

    // Run for another 10 seconds
    println!("⏱️  Running for 10 more seconds (heartbeat disabled)...\n");
    tokio::time::sleep(Duration::from_secs(10)).await;

    // Re-enable the job
    println!("\n▶️  Re-enabling 'heartbeat' job...");
    scheduler.enable_job("heartbeat").await?;

    // Run for another 10 seconds
    println!("⏱️  Running for 10 more seconds (heartbeat enabled)...\n");
    tokio::time::sleep(Duration::from_secs(10)).await;

    // Stop the scheduler
    println!("\n⏹️  Stopping scheduler...");
    scheduler.stop().await?;

    // Final counter value
    let final_count = counter.load(Ordering::SeqCst);
    println!("\n🏁 Final counter value: {}", final_count);

    println!("\n✅ Example completed!");

    Ok(())
}
