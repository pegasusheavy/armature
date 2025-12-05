//! Job queue and background processing for Armature framework.
//!
//! Provides a robust job queue system with:
//! - Redis-backed persistence
//! - Automatic retries with exponential backoff
//! - Job priorities
//! - Delayed/scheduled jobs
//! - Dead letter queue
//! - Job progress tracking
//! - Multiple queues
//! - Worker pools
//!
//! # Examples
//!
//! ```no_run
//! use armature_queue::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), QueueError> {
//!     // Create a queue
//!     let queue = Queue::new("redis://localhost:6379", "default").await?;
//!
//!     // Enqueue a job
//!     let job_id = queue.enqueue(
//!         "send_email",
//!         serde_json::json!({
//!             "to": "user@example.com",
//!             "subject": "Hello"
//!         })
//!     ).await?;
//!
//!     // Process jobs
//!     let mut worker = Worker::new(queue);
//!     worker.register_handler("send_email", |job| async move {
//!         // Send email logic
//!         Ok(())
//!     });
//!
//!     worker.start().await?;
//!
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod job;
pub mod queue;
pub mod worker;

pub use error::{QueueError, QueueResult};
pub use job::{Job, JobData, JobId, JobPriority, JobState, JobStatus};
pub use queue::{Queue, QueueConfig};
pub use worker::{JobHandler, Worker, WorkerConfig};

/// Re-export commonly used types
pub mod prelude {
    pub use crate::error::{QueueError, QueueResult};
    pub use crate::job::{Job, JobData, JobId, JobPriority, JobState, JobStatus};
    pub use crate::queue::{Queue, QueueConfig};
    pub use crate::worker::{JobHandler, Worker, WorkerConfig};
}
