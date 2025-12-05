//! Cron job scheduling for Armature framework.
//!
//! Provides a robust cron job scheduler with support for:
//! - Standard cron expressions
//! - Named jobs with metadata
//! - Async job execution
//! - Job lifecycle hooks
//! - Error handling and retry logic
//! - Job overlap prevention
//!
//! # Examples
//!
//! ```no_run
//! use armature_cron::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), CronError> {
//!     let mut scheduler = CronScheduler::new();
//!
//!     // Schedule a job to run every minute
//!     scheduler.add_job(
//!         "cleanup",
//!         "0 * * * * *",
//!         |context| Box::pin(async move {
//!             println!("Running cleanup job");
//!             Ok(())
//!         })
//!     )?;
//!
//!     // Start the scheduler
//!     scheduler.start().await?;
//!
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod expression;
pub mod job;
pub mod scheduler;

pub use error::{CronError, CronResult};
pub use expression::CronExpression;
pub use job::{Job, JobContext, JobFn, JobStatus};
pub use scheduler::{CronScheduler, SchedulerConfig};

/// Re-export commonly used types
pub mod prelude {
    pub use crate::error::{CronError, CronResult};
    pub use crate::expression::CronExpression;
    pub use crate::job::{Job, JobContext, JobFn, JobStatus};
    pub use crate::scheduler::{CronScheduler, SchedulerConfig};
}
