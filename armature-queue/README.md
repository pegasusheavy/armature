# armature-queue

Background job queue for the Armature framework.

## Features

- **Redis-Backed** - Reliable job storage with Redis
- **Priorities** - Multiple priority levels
- **Retries** - Automatic retry with backoff
- **Scheduling** - Delayed job execution
- **Concurrency** - Multi-worker processing
- **Dead Letter Queue** - Failed job handling

## Installation

```toml
[dependencies]
armature-queue = "0.1"
```

## Quick Start

```rust
use armature_queue::{Queue, Job, Worker};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create queue
    let queue = Queue::new("redis://localhost:6379", "default").await?;

    // Enqueue a job
    queue.enqueue(
        "send_email",
        serde_json::json!({
            "to": "user@example.com",
            "subject": "Welcome!"
        })
    ).await?;

    // Process jobs
    let worker = Worker::new(queue)
        .register("send_email", |job| async move {
            println!("Sending email: {:?}", job.data);
            Ok(())
        })
        .concurrency(4);

    worker.start().await?;
    Ok(())
}
```

## Scheduling

```rust
// Delayed execution
queue.enqueue_in(
    Duration::from_secs(300),
    "reminder",
    serde_json::json!({"message": "Don't forget!"})
).await?;

// Scheduled time
queue.enqueue_at(
    Utc::now() + Duration::hours(1),
    "report",
    serde_json::json!({})
).await?;
```

## License

MIT OR Apache-2.0

