//! Job definition and execution.

use crate::error::CronResult;
use crate::expression::CronExpression;
use chrono::{DateTime, Utc};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Job execution function type.
pub type JobFn =
    Arc<dyn Fn(JobContext) -> Pin<Box<dyn Future<Output = CronResult<()>> + Send>> + Send + Sync>;

/// Job execution context.
#[derive(Debug, Clone)]
pub struct JobContext {
    /// Job name
    pub name: String,

    /// Scheduled execution time
    pub scheduled_time: DateTime<Utc>,

    /// Actual execution time
    pub execution_time: DateTime<Utc>,

    /// Execution count (0-based)
    pub execution_count: u64,
}

impl JobContext {
    /// Create a new job context.
    pub fn new(name: String, scheduled_time: DateTime<Utc>, execution_count: u64) -> Self {
        Self {
            name,
            scheduled_time,
            execution_time: Utc::now(),
            execution_count,
        }
    }

    /// Get the delay between scheduled and actual execution time.
    pub fn delay(&self) -> chrono::Duration {
        self.execution_time - self.scheduled_time
    }
}

/// Job status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobStatus {
    /// Job is scheduled and waiting
    Scheduled,

    /// Job is currently running
    Running,

    /// Job completed successfully
    Completed,

    /// Job failed
    Failed(String),
}

/// Scheduled job.
pub struct Job {
    /// Job name
    pub name: String,

    /// Cron expression
    pub expression: CronExpression,

    /// Job function
    pub function: JobFn,

    /// Job status
    pub status: JobStatus,

    /// Next execution time
    pub next_run: Option<DateTime<Utc>>,

    /// Last execution time
    pub last_run: Option<DateTime<Utc>>,

    /// Total execution count
    pub execution_count: u64,

    /// Whether the job is enabled
    pub enabled: bool,

    /// Whether to prevent overlapping executions
    pub prevent_overlap: bool,

    /// Job metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl Job {
    /// Create a new job.
    pub fn new<F, Fut>(name: impl Into<String>, expression: CronExpression, function: F) -> Self
    where
        F: Fn(JobContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = CronResult<()>> + Send + 'static,
    {
        let name = name.into();
        let next_run = expression.next();

        let wrapped_fn = Arc::new(
            move |ctx: JobContext| -> Pin<Box<dyn Future<Output = CronResult<()>> + Send>> {
                Box::pin(function(ctx))
            },
        );

        Self {
            name,
            expression,
            function: wrapped_fn,
            status: JobStatus::Scheduled,
            next_run,
            last_run: None,
            execution_count: 0,
            enabled: true,
            prevent_overlap: true,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Check if the job should run now.
    pub fn should_run(&self) -> bool {
        if !self.enabled {
            return false;
        }

        if self.prevent_overlap && self.status == JobStatus::Running {
            return false;
        }

        if let Some(next_run) = self.next_run {
            Utc::now() >= next_run
        } else {
            false
        }
    }

    /// Execute the job.
    pub async fn execute(&mut self) -> CronResult<()> {
        if !self.enabled {
            return Ok(());
        }

        if self.prevent_overlap && self.status == JobStatus::Running {
            return Ok(());
        }

        self.status = JobStatus::Running;

        let context = JobContext::new(
            self.name.clone(),
            self.next_run.unwrap_or_else(Utc::now),
            self.execution_count,
        );

        let result = (self.function)(context).await;

        self.last_run = Some(Utc::now());
        self.execution_count += 1;

        match result {
            Ok(()) => {
                self.status = JobStatus::Completed;
                self.next_run = self.expression.next();
                Ok(())
            }
            Err(e) => {
                self.status = JobStatus::Failed(e.to_string());
                self.next_run = self.expression.next();
                Err(e)
            }
        }
    }

    /// Enable the job.
    pub fn enable(&mut self) {
        self.enabled = true;
        if self.next_run.is_none() {
            self.next_run = self.expression.next();
        }
    }

    /// Disable the job.
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Set metadata.
    pub fn set_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }

    /// Get metadata.
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CronExpression;

    #[tokio::test]
    async fn test_job_creation() {
        let expr = CronExpression::parse("0 * * * * *").unwrap();
        let job = Job::new("test", expr, |_ctx| async { Ok(()) });

        assert_eq!(job.name, "test");
        assert_eq!(job.execution_count, 0);
        assert!(job.enabled);
    }

    #[tokio::test]
    async fn test_job_execution() {
        let expr = CronExpression::parse("0 * * * * *").unwrap();
        let mut job = Job::new("test", expr, |_ctx| async { Ok(()) });

        job.next_run = Some(Utc::now()); // Force it to run now

        let result = job.execute().await;
        assert!(result.is_ok());
        assert_eq!(job.execution_count, 1);
        assert_eq!(job.status, JobStatus::Completed);
    }

    #[tokio::test]
    async fn test_job_failure() {
        let expr = CronExpression::parse("0 * * * * *").unwrap();
        let mut job = Job::new("test", expr, |_ctx| async {
            Err(crate::CronError::ExecutionFailed("test error".to_string()))
        });

        job.next_run = Some(Utc::now());

        let result = job.execute().await;
        assert!(result.is_err());
        assert!(matches!(job.status, JobStatus::Failed(_)));
    }

    #[test]
    fn test_job_enable_disable() {
        let expr = CronExpression::parse("0 * * * * *").unwrap();
        let mut job = Job::new("test", expr, |_ctx| async { Ok(()) });

        assert!(job.enabled);

        job.disable();
        assert!(!job.enabled);

        job.enable();
        assert!(job.enabled);
    }
}
