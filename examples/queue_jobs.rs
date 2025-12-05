//! Job queue example.
//!
//! This example demonstrates how to use the job queue for background processing.

use armature_queue::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::time::Duration;

#[tokio::main]
async fn main() -> Result<(), QueueError> {
    println!("📦 Armature Job Queue Example\n");

    // Create a queue
    let queue = Queue::new("redis://localhost:6379", "default").await?;
    println!("✅ Connected to Redis queue\n");

    // Shared counter for demonstration
    let email_count = Arc::new(AtomicU32::new(0));
    let report_count = Arc::new(AtomicU32::new(0));

    // Create and configure worker
    let config = WorkerConfig {
        concurrency: 3,
        poll_interval: Duration::from_secs(1),
        job_timeout: Duration::from_secs(30),
        log_execution: true,
    };

    let mut worker = Worker::with_config(queue.clone(), config);

    // Register job handlers
    println!("📝 Registering job handlers...\n");

    // Email handler
    let email_counter = email_count.clone();
    worker.register_handler("send_email", move |job| {
        let counter = email_counter.clone();
        Box::pin(async move {
            let to = job.data["to"].as_str().unwrap_or("unknown");
            let subject = job.data["subject"].as_str().unwrap_or("No subject");

            println!("📧 Sending email to {} - {}", to, subject);

            // Simulate email sending
            tokio::time::sleep(Duration::from_millis(500)).await;

            counter.fetch_add(1, Ordering::SeqCst);
            println!("✅ Email sent successfully");

            Ok(())
        })
    });

    // Report generation handler
    let report_counter = report_count.clone();
    worker.register_handler("generate_report", move |job| {
        let counter = report_counter.clone();
        Box::pin(async move {
            let report_type = job.data["type"].as_str().unwrap_or("unknown");

            println!("📊 Generating {} report...", report_type);

            // Simulate report generation
            tokio::time::sleep(Duration::from_secs(2)).await;

            counter.fetch_add(1, Ordering::SeqCst);
            println!("✅ Report generated successfully");

            Ok(())
        })
    });

    // Image processing handler
    worker.register_handler("process_image", |job| {
        Box::pin(async move {
            let filename = job.data["filename"].as_str().unwrap_or("unknown.jpg");

            println!("🖼️  Processing image: {}", filename);

            // Simulate image processing
            tokio::time::sleep(Duration::from_secs(1)).await;

            println!("✅ Image processed: {}", filename);

            Ok(())
        })
    });

    // Flaky job handler (simulates failures)
    worker.register_handler("flaky_task", |job| {
        Box::pin(async move {
            let should_fail = job.data["should_fail"].as_bool().unwrap_or(false);

            if should_fail && job.attempts < 2 {
                println!("❌ Flaky task failed (attempt {})", job.attempts);
                return Err(QueueError::ExecutionFailed("Simulated failure".to_string()));
            }

            println!("✅ Flaky task succeeded (attempt {})", job.attempts);
            Ok(())
        })
    });

    // Wait for handlers to be registered
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Start the worker
    println!("▶️  Starting worker...\n");
    worker.start().await?;

    // Enqueue jobs
    println!("📤 Enqueuing jobs...\n");

    // 1. Regular priority emails
    for i in 1..=3 {
        let job_id = queue
            .enqueue(
                "send_email",
                serde_json::json!({
                    "to": format!("user{}@example.com", i),
                    "subject": format!("Welcome Email #{}", i)
                }),
            )
            .await?;
        println!("  ✓ Enqueued email job: {}", job_id);
    }

    // 2. High priority email
    let urgent_job = Job::new(
        "default",
        "send_email",
        serde_json::json!({
            "to": "admin@example.com",
            "subject": "URGENT: System Alert"
        }),
    )
    .with_priority(JobPriority::High);

    let job_id = queue.enqueue_job(urgent_job).await?;
    println!("  ✓ Enqueued HIGH priority email: {}", job_id);

    // 3. Report generation job
    let job_id = queue
        .enqueue(
            "generate_report",
            serde_json::json!({
                "type": "monthly_sales"
            }),
        )
        .await?;
    println!("  ✓ Enqueued report job: {}", job_id);

    // 4. Image processing jobs
    for i in 1..=2 {
        let job_id = queue
            .enqueue(
                "process_image",
                serde_json::json!({
                    "filename": format!("photo_{}.jpg", i)
                }),
            )
            .await?;
        println!("  ✓ Enqueued image job: {}", job_id);
    }

    // 5. Delayed job (scheduled for 5 seconds from now)
    let delayed_job = Job::new(
        "default",
        "send_email",
        serde_json::json!({
            "to": "delayed@example.com",
            "subject": "Delayed Email"
        }),
    )
    .schedule_after(chrono::Duration::seconds(5));

    let job_id = queue.enqueue_job(delayed_job).await?;
    println!("  ✓ Enqueued DELAYED job (5 seconds): {}", job_id);

    // 6. Flaky job that will retry
    let flaky_job = Job::new(
        "default",
        "flaky_task",
        serde_json::json!({
            "should_fail": true
        }),
    )
    .with_max_attempts(3);

    let job_id = queue.enqueue_job(flaky_job).await?;
    println!("  ✓ Enqueued flaky job (will retry): {}", job_id);

    println!("\n⏱️  Processing jobs for 15 seconds...\n");

    // Check queue size
    let size = queue.size().await?;
    println!("📊 Queue size: {} jobs\n", size);

    // Let jobs process
    tokio::time::sleep(Duration::from_secs(15)).await;

    // Show statistics
    println!("\n📈 Statistics:");
    println!("  📧 Emails sent: {}", email_count.load(Ordering::SeqCst));
    println!(
        "  📊 Reports generated: {}",
        report_count.load(Ordering::SeqCst)
    );

    // Check final queue size
    let final_size = queue.size().await?;
    println!("  📦 Remaining jobs: {}", final_size);

    // Enqueue one more job to demonstrate delayed processing
    println!("\n📤 Enqueuing one more delayed job...");
    let final_job = Job::new(
        "default",
        "send_email",
        serde_json::json!({
            "to": "final@example.com",
            "subject": "Final Email"
        }),
    )
    .schedule_after(chrono::Duration::seconds(3));

    queue.enqueue_job(final_job).await?;

    println!("⏱️  Waiting for delayed job (8 more seconds)...\n");
    tokio::time::sleep(Duration::from_secs(8)).await;

    // Stop the worker
    println!("\n⏹️  Stopping worker...");
    worker.stop().await?;

    println!("\n✅ Example completed!");
    println!("\n💡 Tips:");
    println!("  • Jobs are persisted in Redis");
    println!("  • Failed jobs are automatically retried with exponential backoff");
    println!("  • Use priorities for important jobs");
    println!("  • Schedule jobs for future execution");
    println!("  • Multiple workers can process the same queue");

    Ok(())
}
