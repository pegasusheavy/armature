//! Integration tests for armature-audit

use armature_audit::*;
use std::sync::Arc;

#[tokio::test]
async fn test_audit_event_creation() {
    let event = AuditEvent::new("test.event")
        .user("alice")
        .action("test")
        .status(AuditStatus::Success);

    assert_eq!(event.event_type, "test.event");
    assert_eq!(event.user_id, Some("alice".to_string()));
    assert_eq!(event.action, "test");
}

#[tokio::test]
async fn test_memory_backend() {
    let backend = MemoryBackend::new();
    let event = AuditEvent::new("test");

    backend.write(&event).await.unwrap();

    let events = backend.get_events().await;
    assert_eq!(events.len(), 1);
}

#[tokio::test]
async fn test_audit_logger() {
    let backend = MemoryBackend::new();
    let backend_clone = backend.clone();

    let logger = AuditLogger::builder().backend(backend).build();

    let event = AuditEvent::new("test")
        .user("alice")
        .status(AuditStatus::Success);

    logger.log(event).await.unwrap();

    let events = backend_clone.get_events().await;
    assert_eq!(events.len(), 1);
}

#[tokio::test]
async fn test_masking() {
    let _config = MaskingConfig::default();
    let masked = mask_value("secret123", '*', 3);
    assert_eq!(masked, "******123");
}

#[tokio::test]
async fn test_masking_json() {
    let config = MaskingConfig::default();
    let data = serde_json::json!({
        "username": "alice",
        "password": "secret123"
    });

    let masked = mask_json(&data, &config);
    assert_eq!(masked["username"], "alice");
    assert_ne!(masked["password"], "secret123");
}

#[tokio::test]
async fn test_retention_policy() {
    let policy = RetentionPolicy::days(90);
    assert_eq!(policy.max_age.num_days(), 90);
}

#[tokio::test]
async fn test_retention_manager_cleanup() {
    let backend = Arc::new(MemoryBackend::new());
    let old_event = AuditEvent::new("old");
    backend.write(&old_event).await.unwrap();

    let policy = RetentionPolicy::new(chrono::Duration::milliseconds(5));
    let manager = RetentionManager::new(backend.clone(), policy);

    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    let deleted = manager.cleanup().await.unwrap();
    assert!(deleted > 0);
}

#[tokio::test]
async fn test_multi_backend() {
    let backend1 = MemoryBackend::new();
    let backend2 = MemoryBackend::new();

    let backend1_clone = backend1.clone();
    let backend2_clone = backend2.clone();

    let multi = MultiBackend::new()
        .with_backend(Box::new(backend1))
        .with_backend(Box::new(backend2));

    let event = AuditEvent::new("test");
    multi.write(&event).await.unwrap();

    assert_eq!(backend1_clone.get_events().await.len(), 1);
    assert_eq!(backend2_clone.get_events().await.len(), 1);
}

#[test]
fn test_audit_status() {
    assert_eq!(AuditStatus::Success, AuditStatus::Success);
    assert_ne!(AuditStatus::Success, AuditStatus::Failure);
}

#[test]
fn test_audit_severity_ordering() {
    assert!(AuditSeverity::Info < AuditSeverity::Warning);
    assert!(AuditSeverity::Warning < AuditSeverity::Error);
    assert!(AuditSeverity::Error < AuditSeverity::Critical);
}
