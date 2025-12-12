//! Integration tests for armature-queue

use armature_queue::*;
use serde_json::json;
use std::time::Duration;

#[test]
fn test_queue_config_creation() {
    let config = QueueConfig::new("redis://localhost:6379", "default");
    assert_eq!(config.redis_url, "redis://localhost:6379");
    assert_eq!(config.queue_name, "default");
}

#[test]
fn test_queue_config_builder() {
    let config = QueueConfig::new("redis://localhost:6379", "default")
        .with_max_size(1000)
        .with_retention_time(Duration::from_secs(86400));

    assert_eq!(config.max_size, 1000);
    assert_eq!(config.retention_time, Duration::from_secs(86400));
}

#[test]
fn test_job_creation() {
    let job = Job::new("default", "send_email", json!({"to": "user@example.com"}));

    assert_eq!(job.job_type, "send_email");
    assert!(!job.data.is_null());
    assert_eq!(job.attempts, 0);
}

#[test]
fn test_job_with_priority() {
    let job = Job::new("default", "process_data", json!({"data": "value"}))
        .with_priority(JobPriority::High);

    assert_eq!(job.job_type, "process_data");
    assert_eq!(job.priority, JobPriority::High);
}

#[test]
fn test_job_with_max_attempts() {
    let job = Job::new("default", "process_data", json!({})).with_max_attempts(5);

    assert_eq!(job.max_attempts, 5);
}

#[test]
fn test_job_priority_ordering() {
    // Job priorities should be orderable
    assert!(JobPriority::Critical > JobPriority::High);
    assert!(JobPriority::High > JobPriority::Normal);
    assert!(JobPriority::Normal > JobPriority::Low);
}

#[test]
fn test_job_ready() {
    let job = Job::new("default", "test", json!({}));
    assert!(job.is_ready());

    // Job scheduled for future
    let future_job = Job::new("default", "test", json!({}))
        .schedule_at(chrono::Utc::now() + chrono::Duration::minutes(10));
    assert!(!future_job.is_ready());
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
    let queue = Queue::new("redis://localhost:6379", "test_queue")
        .await
        .unwrap();

    // Enqueue a job
    let job = Job::new(
        "test_queue",
        "send_email",
        json!({"to": "test@example.com"}),
    );
    let job_id = queue.enqueue_job(job).await.unwrap();

    assert!(!job_id.is_nil());
}
