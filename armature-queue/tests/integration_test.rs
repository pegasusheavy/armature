//! Integration tests for armature-queue

use armature_queue::*;
use serde_json::json;

#[test]
fn test_queue_config_creation() {
    let config = QueueConfig::new("redis://localhost:6379", "default");
    assert_eq!(config.redis_url, "redis://localhost:6379");
    assert_eq!(config.queue_name, "default");
}

#[test]
fn test_queue_config_builder() {
    let config = QueueConfig::new("redis://localhost:6379", "default")
        .with_max_retries(5)
        .with_retry_delay(10)
        .with_retention_time(86400);

    assert_eq!(config.max_retries, 5);
    assert_eq!(config.retry_delay, 10);
    assert_eq!(config.retention_time, 86400);
}

#[test]
fn test_job_creation() {
    let job = Job::new("send_email", json!({"to": "user@example.com"}));

    assert_eq!(job.job_type, "send_email");
    assert!(!job.payload.is_null());
    assert_eq!(job.retry_count, 0);
}

#[test]
fn test_job_builder() {
    let job = Job::builder()
        .job_type("process_data")
        .payload(json!({"data": "value"}))
        .priority(JobPriority::High)
        .max_retries(3)
        .build();

    assert_eq!(job.job_type, "process_data");
    assert_eq!(job.priority, JobPriority::High);
    assert_eq!(job.max_retries, Some(3));
}

#[test]
fn test_job_priority() {
    assert_eq!(JobPriority::Critical.score(), 4);
    assert_eq!(JobPriority::High.score(), 3);
    assert_eq!(JobPriority::Normal.score(), 2);
    assert_eq!(JobPriority::Low.score(), 1);
}

#[test]
fn test_job_ready() {
    let job = Job::new("test", json!({}));
    assert!(job.is_ready());

    // Job scheduled for future
    let future_job = Job::builder()
        .job_type("test")
        .payload(json!({}))
        .schedule_at(chrono::Utc::now() + chrono::Duration::minutes(10))
        .build();
    assert!(!future_job.is_ready());
}

#[test]
fn test_job_retry_logic() {
    let mut job = Job::new("test", json!({}));

    assert!(job.can_retry());
    job.retry_count = job.max_retries.unwrap_or(3);
    assert!(!job.can_retry());
}

#[test]
fn test_job_backoff_delay() {
    let job = Job::new("test", json!({}));
    let delay = job.backoff_delay();
    assert!(delay > 0);
}

#[test]
fn test_queue_error_display() {
    let err = QueueError::JobNotFound("job123".to_string());
    let display = format!("{}", err);
    assert!(display.contains("job123"));
}

// Note: These tests would require Redis running
// They are disabled by default but can be run with: cargo test -- --ignored

#[tokio::test]
#[ignore]
async fn test_queue_enqueue_and_process() {
    let config = QueueConfig::new("redis://localhost:6379", "test_queue");
    let queue = Queue::new(config).await.unwrap();

    // Enqueue a job
    let job_id = queue.enqueue("send_email", json!({"to": "test@example.com"}))
        .await
        .unwrap();

    assert!(!job_id.is_empty());

    // Process the job
    let job = queue.process_next().await.unwrap();
    assert!(job.is_some());

    let job = job.unwrap();
    assert_eq!(job.job_type, "send_email");
}

#[tokio::test]
#[ignore]
async fn test_queue_with_priority() {
    let config = QueueConfig::new("redis://localhost:6379", "test_queue");
    let queue = Queue::new(config).await.unwrap();

    // Enqueue jobs with different priorities
    queue.enqueue_with_priority("low", json!({}), JobPriority::Low)
        .await
        .unwrap();

    queue.enqueue_with_priority("high", json!({}), JobPriority::High)
        .await
        .unwrap();

    // High priority should be processed first
    let job = queue.process_next().await.unwrap().unwrap();
    assert_eq!(job.job_type, "high");
}

#[tokio::test]
#[ignore]
async fn test_worker_creation() {
    let config = QueueConfig::new("redis://localhost:6379", "test_queue");
    let queue = Queue::new(config).await.unwrap();

    let worker = Worker::new(queue);
    assert!(format!("{:?}", worker).contains("Worker"));
}

