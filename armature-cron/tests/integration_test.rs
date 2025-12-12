//! Integration tests for armature-cron

use armature_cron::expression::CronPresets;
use armature_cron::*;

#[test]
fn test_cron_expression_parsing() {
    // Valid expression
    let expr = CronExpression::parse("0 0 * * * *");
    assert!(expr.is_ok());

    // Invalid expression
    let expr = CronExpression::parse("invalid");
    assert!(expr.is_err());
}

#[test]
fn test_cron_presets() {
    assert!(CronExpression::parse(CronPresets::EVERY_MINUTE).is_ok());
    assert!(CronExpression::parse(CronPresets::EVERY_HOUR).is_ok());
    assert!(CronExpression::parse(CronPresets::DAILY).is_ok());
    assert!(CronExpression::parse(CronPresets::WEEKLY).is_ok());
    assert!(CronExpression::parse(CronPresets::MONTHLY).is_ok());
}

#[tokio::test]
async fn test_job_creation() {
    let job = Job::new(
        "test_job".to_string(),
        CronExpression::parse("0 0 * * * *").unwrap(),
        |_ctx| Box::pin(async { Ok(()) }),
    );

    assert_eq!(job.name, "test_job");
    assert!(matches!(job.status, JobStatus::Scheduled));
}

#[tokio::test]
async fn test_job_execution() {
    use std::sync::{Arc, Mutex};

    let executed = Arc::new(Mutex::new(false));
    let executed_clone = executed.clone();

    let mut job = Job::new(
        "test_job".to_string(),
        CronExpression::parse("0 0 * * * *").unwrap(),
        move |_ctx| {
            let executed = executed_clone.clone();
            Box::pin(async move {
                *executed.lock().unwrap() = true;
                Ok(())
            })
        },
    );

    // Execute job manually
    job.execute().await.unwrap();

    assert!(*executed.lock().unwrap());
}

#[tokio::test]
async fn test_job_enable_disable() {
    let mut job = Job::new(
        "test_job".to_string(),
        CronExpression::parse("0 0 * * * *").unwrap(),
        |_ctx| Box::pin(async { Ok(()) }),
    );

    assert!(job.enabled);

    job.disable();
    assert!(!job.enabled);

    job.enable();
    assert!(job.enabled);
}

#[tokio::test]
async fn test_job_status() {
    let job = Job::new(
        "test_job".to_string(),
        CronExpression::parse("0 0 * * * *").unwrap(),
        |_ctx| Box::pin(async { Ok(()) }),
    );

    assert!(matches!(job.status, JobStatus::Scheduled));
}

#[tokio::test]
async fn test_job_context() {
    let mut job = Job::new(
        "test_job".to_string(),
        CronExpression::parse("0 0 * * * *").unwrap(),
        |ctx| {
            Box::pin(async move {
                assert!(!ctx.name.is_empty());
                Ok(())
            })
        },
    );

    job.execute().await.unwrap();
}

#[test]
fn test_cron_error_display() {
    let err = CronError::InvalidExpression("bad cron".to_string());
    let display = format!("{}", err);
    assert!(display.contains("bad cron"));
}
