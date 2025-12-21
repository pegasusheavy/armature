//! Cron job scheduler.

use crate::error::{CronError, CronResult};
use crate::expression::CronExpression;
use crate::job::{Job, JobContext};
use armature_log::{debug, info, warn};
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

/// Scheduler configuration.
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    /// Tick interval for checking scheduled jobs
    pub tick_interval: Duration,

    /// Whether to run missed jobs on startup
    pub run_missed_jobs: bool,

    /// Maximum concurrent jobs
    pub max_concurrent_jobs: usize,

    /// Whether to log job execution
    pub log_execution: bool,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            tick_interval: Duration::from_secs(1),
            run_missed_jobs: false,
            max_concurrent_jobs: 10,
            log_execution: true,
        }
    }
}

/// Cron job scheduler.
pub struct CronScheduler {
    jobs: Arc<RwLock<HashMap<String, Job>>>,
    config: SchedulerConfig,
    running: Arc<RwLock<bool>>,
    handle: Option<JoinHandle<()>>,
}

impl CronScheduler {
    /// Create a new scheduler with default configuration.
    pub fn new() -> Self {
        Self::with_config(SchedulerConfig::default())
    }

    /// Create a new scheduler with custom configuration.
    pub fn with_config(config: SchedulerConfig) -> Self {
        info!("Initializing cron scheduler");
        debug!(
            "Scheduler config - tick_interval: {:?}, max_concurrent: {}",
            config.tick_interval, config.max_concurrent_jobs
        );
        Self {
            jobs: Arc::new(RwLock::new(HashMap::new())),
            config,
            running: Arc::new(RwLock::new(false)),
            handle: None,
        }
    }

    /// Add a job to the scheduler.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use armature_cron::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), CronError> {
    /// let mut scheduler = CronScheduler::new();
    ///
    /// scheduler.add_job(
    ///     "cleanup",
    ///     "0 0 0 * * *", // Every day at midnight
    ///     |ctx| Box::pin(async move {
    ///         println!("Running cleanup job");
    ///         Ok(())
    ///     })
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn add_job<F, Fut>(
        &mut self,
        name: impl Into<String>,
        expression: &str,
        function: F,
    ) -> CronResult<()>
    where
        F: Fn(JobContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = CronResult<()>> + Send + 'static,
    {
        let name = name.into();
        let expr = CronExpression::parse(expression)?;
        info!("Adding cron job '{}' with schedule '{}'", name, expression);

        let jobs = self.jobs.clone();
        let name_clone = name.clone();

        tokio::spawn(async move {
            let mut jobs = jobs.write().await;
            if jobs.contains_key(&name_clone) {
                warn!("Job '{}' already exists, skipping", name_clone);
                return;
            }

            let job = Job::new(name_clone.clone(), expr, function);
            jobs.insert(name_clone.clone(), job);
            debug!("Job '{}' registered successfully", name_clone);
        });

        Ok(())
    }

    /// Remove a job from the scheduler.
    pub async fn remove_job(&mut self, name: &str) -> CronResult<()> {
        let mut jobs = self.jobs.write().await;
        jobs.remove(name)
            .ok_or_else(|| CronError::JobNotFound(name.to_string()))?;
        Ok(())
    }

    /// Get a list of all job names.
    pub async fn list_jobs(&self) -> Vec<String> {
        let jobs = self.jobs.read().await;
        jobs.keys().cloned().collect()
    }

    /// Enable a job.
    pub async fn enable_job(&self, name: &str) -> CronResult<()> {
        let mut jobs = self.jobs.write().await;
        let job = jobs
            .get_mut(name)
            .ok_or_else(|| CronError::JobNotFound(name.to_string()))?;
        job.enable();
        Ok(())
    }

    /// Disable a job.
    pub async fn disable_job(&self, name: &str) -> CronResult<()> {
        let mut jobs = self.jobs.write().await;
        let job = jobs
            .get_mut(name)
            .ok_or_else(|| CronError::JobNotFound(name.to_string()))?;
        job.disable();
        Ok(())
    }

    /// Start the scheduler.
    pub async fn start(&mut self) -> CronResult<()> {
        let mut running = self.running.write().await;
        if *running {
            warn!("Cron scheduler already running");
            return Err(CronError::SchedulerAlreadyRunning);
        }
        *running = true;
        drop(running);

        info!("Cron scheduler started");

        let jobs = self.jobs.clone();
        let running = self.running.clone();
        let tick_interval = self.config.tick_interval;
        let log_execution = self.config.log_execution;

        let handle = tokio::spawn(async move {
            while *running.read().await {
                let job_names: Vec<String> = {
                    let jobs = jobs.read().await;
                    jobs.keys().cloned().collect()
                };

                for name in job_names {
                    let jobs_clone = jobs.clone();
                    let log = log_execution;

                    tokio::spawn(async move {
                        let should_run = {
                            let jobs = jobs_clone.read().await;
                            jobs.get(&name).map(|j| j.should_run()).unwrap_or(false)
                        };

                        if should_run {
                            let mut jobs = jobs_clone.write().await;
                            if let Some(job) = jobs.get_mut(&name) {
                                if log {
                                    println!("[CRON] Executing job: {}", name);
                                }

                                if let Err(e) = job.execute().await {
                                    eprintln!("[CRON] Job {} failed: {}", name, e);
                                } else if log {
                                    println!("[CRON] Job {} completed successfully", name);
                                }
                            }
                        }
                    });
                }

                tokio::time::sleep(tick_interval).await;
            }
        });

        self.handle = Some(handle);
        Ok(())
    }

    /// Stop the scheduler.
    pub async fn stop(&mut self) -> CronResult<()> {
        let mut running = self.running.write().await;
        if !*running {
            return Err(CronError::SchedulerNotRunning);
        }
        *running = false;
        drop(running);

        if let Some(handle) = self.handle.take() {
            handle.abort();
        }

        Ok(())
    }

    /// Check if the scheduler is running.
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// Get job statistics.
    pub async fn get_stats(&self, name: &str) -> CronResult<JobStats> {
        let jobs = self.jobs.read().await;
        let job = jobs
            .get(name)
            .ok_or_else(|| CronError::JobNotFound(name.to_string()))?;

        Ok(JobStats {
            name: job.name.clone(),
            enabled: job.enabled,
            execution_count: job.execution_count,
            last_run: job.last_run,
            next_run: job.next_run,
            status: job.status.clone(),
        })
    }
}

impl Default for CronScheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// Job statistics.
#[derive(Debug, Clone)]
pub struct JobStats {
    pub name: String,
    pub enabled: bool,
    pub execution_count: u64,
    pub last_run: Option<chrono::DateTime<chrono::Utc>>,
    pub next_run: Option<chrono::DateTime<chrono::Utc>>,
    pub status: crate::job::JobStatus,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scheduler_creation() {
        let scheduler = CronScheduler::new();
        assert!(!scheduler.is_running().await);
    }

    #[tokio::test]
    async fn test_add_job() {
        let mut scheduler = CronScheduler::new();
        let result = scheduler.add_job("test", "0 * * * * *", |_| async { Ok(()) });
        assert!(result.is_ok());

        tokio::time::sleep(Duration::from_millis(100)).await;
        let jobs = scheduler.list_jobs().await;
        assert!(jobs.contains(&"test".to_string()));
    }

    #[tokio::test]
    async fn test_remove_job() {
        let mut scheduler = CronScheduler::new();
        scheduler
            .add_job("test", "0 * * * * *", |_| async { Ok(()) })
            .unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await;

        let result = scheduler.remove_job("test").await;
        assert!(result.is_ok());

        let jobs = scheduler.list_jobs().await;
        assert!(!jobs.contains(&"test".to_string()));
    }

    #[tokio::test]
    async fn test_start_stop() {
        let mut scheduler = CronScheduler::new();

        assert!(!scheduler.is_running().await);

        scheduler.start().await.unwrap();
        assert!(scheduler.is_running().await);

        scheduler.stop().await.unwrap();
        assert!(!scheduler.is_running().await);
    }
}
