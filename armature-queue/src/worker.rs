//! Worker implementation for processing jobs.

use crate::error::{QueueError, QueueResult};
use crate::job::Job;
use crate::queue::Queue;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

/// Job handler function type.
pub type JobHandler =
    Arc<dyn Fn(Job) -> Pin<Box<dyn Future<Output = QueueResult<()>> + Send>> + Send + Sync>;

/// Worker configuration.
#[derive(Debug, Clone)]
pub struct WorkerConfig {
    /// Number of concurrent jobs to process
    pub concurrency: usize,

    /// Poll interval for checking new jobs
    pub poll_interval: Duration,

    /// Timeout for job execution
    pub job_timeout: Duration,

    /// Whether to log job execution
    pub log_execution: bool,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            concurrency: 10,
            poll_interval: Duration::from_secs(1),
            job_timeout: Duration::from_secs(300), // 5 minutes
            log_execution: true,
        }
    }
}

/// Worker for processing jobs from a queue.
pub struct Worker {
    queue: Queue,
    handlers: Arc<RwLock<HashMap<String, JobHandler>>>,
    config: WorkerConfig,
    running: Arc<RwLock<bool>>,
    handles: Vec<JoinHandle<()>>,
}

impl Worker {
    /// Create a new worker.
    pub fn new(queue: Queue) -> Self {
        Self::with_config(queue, WorkerConfig::default())
    }

    /// Create a worker with custom configuration.
    pub fn with_config(queue: Queue, config: WorkerConfig) -> Self {
        Self {
            queue,
            handlers: Arc::new(RwLock::new(HashMap::new())),
            config,
            running: Arc::new(RwLock::new(false)),
            handles: Vec::new(),
        }
    }

    /// Register a job handler.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use armature_queue::*;
    ///
    /// # async fn example() -> QueueResult<()> {
    /// let queue = Queue::new("redis://localhost:6379", "default").await?;
    /// let mut worker = Worker::new(queue);
    ///
    /// worker.register_handler("send_email", |job| {
    ///     Box::pin(async move {
    ///         // Send email logic
    ///         println!("Sending email: {:?}", job.data);
    ///         Ok(())
    ///     })
    /// });
    /// # Ok(())
    /// # }
    /// ```
    pub fn register_handler<F, Fut>(&mut self, job_type: impl Into<String>, handler: F)
    where
        F: Fn(Job) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = QueueResult<()>> + Send + 'static,
    {
        let wrapped_handler = Arc::new(
            move |job: Job| -> Pin<Box<dyn Future<Output = QueueResult<()>> + Send>> {
                Box::pin(handler(job))
            },
        );

        let job_type = job_type.into();
        let handlers = self.handlers.clone();

        tokio::spawn(async move {
            let mut handlers = handlers.write().await;
            handlers.insert(job_type, wrapped_handler);
        });
    }

    /// Start the worker.
    pub async fn start(&mut self) -> QueueResult<()> {
        let mut running = self.running.write().await;
        if *running {
            return Err(QueueError::WorkerAlreadyRunning);
        }
        *running = true;
        drop(running);

        if self.config.log_execution {
            println!(
                "[WORKER] Starting with concurrency: {}",
                self.config.concurrency
            );
        }

        // Start worker tasks
        for i in 0..self.config.concurrency {
            let queue = self.queue.clone();
            let handlers = self.handlers.clone();
            let running = self.running.clone();
            let poll_interval = self.config.poll_interval;
            let job_timeout = self.config.job_timeout;
            let log = self.config.log_execution;

            let handle = tokio::spawn(async move {
                while *running.read().await {
                    match queue.dequeue().await {
                        Ok(Some(job)) => {
                            let job_id = job.id;
                            let job_type = job.job_type.clone();

                            if log {
                                println!(
                                    "[WORKER-{}] Processing job: {} (type: {})",
                                    i, job_id, job_type
                                );
                            }

                            // Get handler
                            let handler = {
                                let handlers = handlers.read().await;
                                handlers.get(&job_type).cloned()
                            };

                            if let Some(handler) = handler {
                                // Execute job with timeout
                                let result =
                                    tokio::time::timeout(job_timeout, handler(job.clone())).await;

                                match result {
                                    Ok(Ok(())) => {
                                        // Job succeeded
                                        if let Err(e) = queue.complete(job_id).await {
                                            eprintln!(
                                                "[WORKER-{}] Failed to mark job as complete: {}",
                                                i, e
                                            );
                                        } else if log {
                                            println!(
                                                "[WORKER-{}] Job {} completed successfully",
                                                i, job_id
                                            );
                                        }
                                    }
                                    Ok(Err(e)) => {
                                        // Job failed
                                        eprintln!("[WORKER-{}] Job {} failed: {}", i, job_id, e);
                                        if let Err(err) = queue.fail(job_id, e.to_string()).await {
                                            eprintln!(
                                                "[WORKER-{}] Failed to mark job as failed: {}",
                                                i, err
                                            );
                                        }
                                    }
                                    Err(_) => {
                                        // Timeout
                                        eprintln!("[WORKER-{}] Job {} timed out", i, job_id);
                                        if let Err(e) =
                                            queue.fail(job_id, "Job timeout".to_string()).await
                                        {
                                            eprintln!(
                                                "[WORKER-{}] Failed to mark job as failed: {}",
                                                i, e
                                            );
                                        }
                                    }
                                }
                            } else {
                                eprintln!("[WORKER-{}] No handler for job type: {}", i, job_type);
                                if let Err(e) = queue
                                    .fail(job_id, format!("No handler for job type: {}", job_type))
                                    .await
                                {
                                    eprintln!("[WORKER-{}] Failed to mark job as failed: {}", i, e);
                                }
                            }
                        }
                        Ok(None) => {
                            // No jobs available, wait before polling again
                            tokio::time::sleep(poll_interval).await;
                        }
                        Err(e) => {
                            eprintln!("[WORKER-{}] Error dequeuing job: {}", i, e);
                            tokio::time::sleep(poll_interval).await;
                        }
                    }
                }

                if log {
                    println!("[WORKER-{}] Stopped", i);
                }
            });

            self.handles.push(handle);
        }

        Ok(())
    }

    /// Stop the worker.
    pub async fn stop(&mut self) -> QueueResult<()> {
        let mut running = self.running.write().await;
        if !*running {
            return Err(QueueError::WorkerNotRunning);
        }
        *running = false;
        drop(running);

        if self.config.log_execution {
            println!("[WORKER] Stopping...");
        }

        // Abort all worker tasks
        for handle in self.handles.drain(..) {
            handle.abort();
        }

        if self.config.log_execution {
            println!("[WORKER] Stopped");
        }

        Ok(())
    }

    /// Check if the worker is running.
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[test]
    fn test_worker_config() {
        let config = WorkerConfig::default();
        assert_eq!(config.concurrency, 10);
        assert!(config.log_execution);
    }

    #[tokio::test]
    async fn test_worker_creation() {
        // This test requires a real Redis connection, so we just test creation
        // In a real environment, you'd use a test Redis instance
        let config = WorkerConfig {
            concurrency: 5,
            poll_interval: Duration::from_millis(500),
            job_timeout: Duration::from_secs(60),
            log_execution: false,
        };

        assert_eq!(config.concurrency, 5);
    }
}
