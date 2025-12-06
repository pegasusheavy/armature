//! Parallel Batch Queue Processing Example
//!
//! Demonstrates the throughput improvements from processing multiple jobs
//! of the same type in parallel using batch processing.
//!
//! # Performance
//!
//! **Sequential:**  10 jobs × 200ms = 2 seconds
//! **Parallel:**    max(10 × 200ms) = ~200ms
//! **Speedup:**     ~10x higher throughput

use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use tokio::sync::Mutex;

/// Simulated job for demonstration
#[derive(Debug, Clone)]
struct Job {
    id: usize,
    job_type: String,
    data: serde_json::Value,
}

/// Simulated queue result
type Result<T> = std::result::Result<T, String>;

/// Mock queue worker for demonstration
struct MockQueueWorker {
    jobs: Arc<Mutex<Vec<Job>>>,
    processed: Arc<AtomicUsize>,
}

impl MockQueueWorker {
    fn new() -> Self {
        Self {
            jobs: Arc::new(Mutex::new(Vec::new())),
            processed: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Enqueue a job
    async fn enqueue(&self, job_type: String, data: serde_json::Value) -> Result<usize> {
        let mut jobs = self.jobs.lock().await;
        let id = jobs.len();
        jobs.push(Job { id, job_type, data });
        Ok(id)
    }

    /// Process jobs sequentially
    async fn process_sequential(&self, count: usize) -> Result<Vec<usize>> {
        println!("📝 Processing {} jobs sequentially...", count);
        let start = Instant::now();

        let mut processed_ids = Vec::new();
        let mut jobs = self.jobs.lock().await;

        for _ in 0..count {
            if let Some(job) = jobs.pop() {
                // Simulate job processing (image resize, video encode, etc.)
                self.process_job(&job).await?;
                processed_ids.push(job.id);
                self.processed.fetch_add(1, Ordering::SeqCst);
            }
        }

        let duration = start.elapsed();
        println!(
            "✅ Sequential complete: {} jobs in {:?} ({:.1} jobs/sec)",
            processed_ids.len(),
            duration,
            processed_ids.len() as f64 / duration.as_secs_f64()
        );

        Ok(processed_ids)
    }

    /// Process batch of jobs in parallel
    async fn process_batch(&self, batch_size: usize) -> Result<Vec<usize>> {
        use tokio::task::JoinSet;

        println!("🚀 Processing batch of {} jobs in parallel...", batch_size);
        let start = Instant::now();

        // Dequeue jobs
        let mut jobs_to_process = Vec::new();
        {
            let mut jobs = self.jobs.lock().await;
            for _ in 0..batch_size {
                if let Some(job) = jobs.pop() {
                    jobs_to_process.push(job);
                }
            }
        }

        if jobs_to_process.is_empty() {
            return Ok(Vec::new());
        }

        // Process all jobs in parallel
        let mut set = JoinSet::new();

        for job in jobs_to_process {
            let worker = self.clone();
            set.spawn(async move {
                worker.process_job(&job).await?;
                worker.processed.fetch_add(1, Ordering::SeqCst);
                Ok::<_, String>(job.id)
            });
        }

        // Collect results
        let mut processed_ids = Vec::new();
        while let Some(result) = set.join_next().await {
            match result {
                Ok(Ok(id)) => processed_ids.push(id),
                Ok(Err(e)) => return Err(e),
                Err(e) => return Err(format!("Task failed: {}", e)),
            }
        }

        let duration = start.elapsed();
        println!(
            "✅ Batch complete: {} jobs in {:?} ({:.1} jobs/sec)",
            processed_ids.len(),
            duration,
            processed_ids.len() as f64 / duration.as_secs_f64()
        );

        Ok(processed_ids)
    }

    /// Simulate CPU-intensive job processing
    async fn process_job(&self, job: &Job) -> Result<()> {
        // Simulate processing time (e.g., image resize, video encode)
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Simulate success
        Ok(())
    }

    /// Get number of processed jobs
    fn get_processed_count(&self) -> usize {
        self.processed.load(Ordering::SeqCst)
    }

    /// Reset worker
    async fn reset(&self) {
        self.jobs.lock().await.clear();
        self.processed.store(0, Ordering::SeqCst);
    }
}

impl Clone for MockQueueWorker {
    fn clone(&self) -> Self {
        Self {
            jobs: Arc::clone(&self.jobs),
            processed: Arc::clone(&self.processed),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                                                              ║");
    println!("║      Parallel Batch Queue Processing Performance Demo       ║");
    println!("║                                                              ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let worker = MockQueueWorker::new();

    // ========================================================================
    // BENCHMARK 1: Sequential Job Processing
    // ========================================================================

    println!("═══════════════════════════════════════════════════════════════");
    println!("  BENCHMARK 1: Sequential Job Processing                      ");
    println!("═══════════════════════════════════════════════════════════════\n");

    // Enqueue jobs
    println!("📦 Enqueuing 20 image processing jobs...\n");
    for i in 0..20 {
        worker
            .enqueue(
                "process_image".to_string(),
                json!({
                    "image_path": format!("/uploads/image{}.jpg", i),
                    "width": 800,
                    "height": 600
                }),
            )
            .await?;
    }

    let seq_start = Instant::now();
    let sequential = worker.process_sequential(20).await?;
    let seq_duration = seq_start.elapsed();

    println!("\nResults:");
    println!("  • Jobs processed: {}", sequential.len());
    println!("  • Total time: {:?}", seq_duration);
    println!("  • Average: {:?} per job", seq_duration / sequential.len() as u32);
    println!(
        "  • Throughput: {:.1} jobs/sec\n",
        sequential.len() as f64 / seq_duration.as_secs_f64()
    );

    worker.reset().await;

    // ========================================================================
    // BENCHMARK 2: Parallel Batch Processing
    // ========================================================================

    println!("═══════════════════════════════════════════════════════════════");
    println!("  BENCHMARK 2: Parallel Batch Processing                      ");
    println!("═══════════════════════════════════════════════════════════════\n");

    // Enqueue jobs again
    println!("📦 Enqueuing 20 image processing jobs...\n");
    for i in 0..20 {
        worker
            .enqueue(
                "process_image".to_string(),
                json!({
                    "image_path": format!("/uploads/image{}.jpg", i),
                    "width": 800,
                    "height": 600
                }),
            )
            .await?;
    }

    let par_start = Instant::now();
    let parallel = worker.process_batch(20).await?;
    let par_duration = par_start.elapsed();

    println!("\nResults:");
    println!("  • Jobs processed: {}", parallel.len());
    println!("  • Total time: {:?}", par_duration);
    println!("  • Average: {:?} per job", par_duration / parallel.len() as u32);
    println!(
        "  • Throughput: {:.1} jobs/sec\n",
        parallel.len() as f64 / par_duration.as_secs_f64()
    );

    // ========================================================================
    // COMPARISON
    // ========================================================================

    println!("═══════════════════════════════════════════════════════════════");
    println!("  PERFORMANCE COMPARISON                                       ");
    println!("═══════════════════════════════════════════════════════════════\n");

    let speedup = seq_duration.as_secs_f64() / par_duration.as_secs_f64();
    let time_saved = seq_duration - par_duration;
    let percentage_faster = ((speedup - 1.0) * 100.0).round() as i32;

    println!("Sequential:");
    println!("  ⏱️  Time: {:?}", seq_duration);
    println!("  📊 Throughput: {:.1} jobs/sec",
        sequential.len() as f64 / seq_duration.as_secs_f64());
    println!();

    println!("Parallel Batch:");
    println!("  ⏱️  Time: {:?}", par_duration);
    println!("  📊 Throughput: {:.1} jobs/sec",
        parallel.len() as f64 / par_duration.as_secs_f64());
    println!();

    println!("Performance Gain:");
    println!("  🚀 Speedup: {:.2}x faster", speedup);
    println!("  ⏰ Time saved: {:?}", time_saved);
    println!("  📈 Throughput increase: {}%\n", percentage_faster);

    // ========================================================================
    // DEMO: Mixed Job Types
    // ========================================================================

    println!("═══════════════════════════════════════════════════════════════");
    println!("  DEMO: Mixed Job Types with Batching                         ");
    println!("═══════════════════════════════════════════════════════════════\n");

    worker.reset().await;

    // Enqueue different job types
    println!("📦 Enqueuing mixed job types...");
    for i in 0..10 {
        worker.enqueue("process_image".to_string(), json!({"id": i})).await?;
    }
    for i in 0..10 {
        worker.enqueue("send_email".to_string(), json!({"id": i})).await?;
    }
    for i in 0..10 {
        worker.enqueue("generate_report".to_string(), json!({"id": i})).await?;
    }

    println!("  • process_image: 10 jobs");
    println!("  • send_email: 10 jobs");
    println!("  • generate_report: 10 jobs");
    println!("  • Total: 30 jobs\n");

    println!("Processing in batches of 10...");
    let batch_start = Instant::now();

    // Process in batches (would be filtered by job type in real implementation)
    let batch1 = worker.process_batch(10).await?;
    let batch2 = worker.process_batch(10).await?;
    let batch3 = worker.process_batch(10).await?;

    let batch_duration = batch_start.elapsed();
    let total_processed = batch1.len() + batch2.len() + batch3.len();

    println!(
        "\n✅ Processed {} jobs in {} batches ({:?})",
        total_processed,
        3,
        batch_duration
    );
    println!(
        "   Throughput: {:.1} jobs/sec\n",
        total_processed as f64 / batch_duration.as_secs_f64()
    );

    // ========================================================================
    // SUMMARY
    // ========================================================================

    println!("═══════════════════════════════════════════════════════════════");
    println!("                    KEY TAKEAWAYS                              ");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("✅ Batch Queue Processing Benefits:");
    println!("   • 3-5x higher job throughput");
    println!("   • Better resource utilization");
    println!("   • Reduced total processing time");
    println!("   • Scales with CPU cores\n");

    println!("📦 Use Cases:");
    println!("   • Image processing/resizing");
    println!("   • Video encoding");
    println!("   • PDF generation");
    println!("   • Email batching");
    println!("   • Report generation");
    println!("   • Data exports\n");

    println!("💡 Implementation:");
    println!("   • Dequeue multiple jobs of same type");
    println!("   • Use tokio::task::JoinSet for parallel execution");
    println!("   • Mark jobs as complete/failed individually");
    println!("   • Monitor batch statistics\n");

    println!("⚠️  Considerations:");
    println!("   • CPU-intensive jobs: Use spawn_blocking");
    println!("   • Memory limits: Batch size should fit in RAM");
    println!("   • Error handling: Individual job failures");
    println!("   • Job ordering: May not preserve order\n");

    println!("═══════════════════════════════════════════════════════════════");
    println!("✅ Batch queue processing demo complete!");
    println!("═══════════════════════════════════════════════════════════════\n");

    Ok(())
}

