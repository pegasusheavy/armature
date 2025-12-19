//! Audit log storage backends

use crate::AuditEvent;
use async_trait::async_trait;
use std::path::PathBuf;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

/// Audit log storage backend trait
#[async_trait]
pub trait AuditBackend: Send + Sync {
    /// Write an audit event
    async fn write(&self, event: &AuditEvent) -> Result<(), AuditBackendError>;

    /// Flush any pending writes
    async fn flush(&self) -> Result<(), AuditBackendError>;

    /// Read events (if supported)
    async fn read(&self, _limit: usize) -> Result<Vec<AuditEvent>, AuditBackendError> {
        Err(AuditBackendError::NotSupported)
    }

    /// Delete old events (for retention)
    async fn delete_before(
        &self,
        _timestamp: chrono::DateTime<chrono::Utc>,
    ) -> Result<usize, AuditBackendError> {
        Err(AuditBackendError::NotSupported)
    }
}

/// Audit backend errors
#[derive(Debug, thiserror::Error)]
pub enum AuditBackendError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Operation not supported")]
    NotSupported,

    #[error("Backend error: {0}")]
    Other(String),
}

/// File-based audit backend
///
/// Writes audit events to a file, one JSON object per line.
pub struct FileBackend {
    path: PathBuf,
}

impl FileBackend {
    /// Create a new file backend
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use armature_audit::*;
    ///
    /// let backend = FileBackend::new("audit.log");
    /// ```
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

#[async_trait]
impl AuditBackend for FileBackend {
    async fn write(&self, event: &AuditEvent) -> Result<(), AuditBackendError> {
        let json = event.to_json()?;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .await?;

        file.write_all(json.as_bytes()).await?;
        file.write_all(b"\n").await?;
        file.flush().await?;

        Ok(())
    }

    async fn flush(&self) -> Result<(), AuditBackendError> {
        // File backend auto-flushes on each write
        Ok(())
    }
}

/// Memory backend for testing
///
/// Stores audit events in memory.
#[derive(Clone)]
pub struct MemoryBackend {
    events: std::sync::Arc<tokio::sync::Mutex<Vec<AuditEvent>>>,
}

impl MemoryBackend {
    /// Create a new memory backend
    pub fn new() -> Self {
        Self {
            events: std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new())),
        }
    }

    /// Get all events
    pub async fn get_events(&self) -> Vec<AuditEvent> {
        self.events.lock().await.clone()
    }

    /// Clear all events
    pub async fn clear(&self) {
        self.events.lock().await.clear();
    }
}

impl Default for MemoryBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AuditBackend for MemoryBackend {
    async fn write(&self, event: &AuditEvent) -> Result<(), AuditBackendError> {
        self.events.lock().await.push(event.clone());
        Ok(())
    }

    async fn flush(&self) -> Result<(), AuditBackendError> {
        Ok(())
    }

    async fn read(&self, limit: usize) -> Result<Vec<AuditEvent>, AuditBackendError> {
        let events = self.events.lock().await;
        let count = events.len().min(limit);
        Ok(events.iter().rev().take(count).cloned().collect())
    }

    async fn delete_before(
        &self,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> Result<usize, AuditBackendError> {
        let mut events = self.events.lock().await;
        let original_len = events.len();
        events.retain(|e| e.timestamp >= timestamp);
        Ok(original_len - events.len())
    }
}

/// Stdout backend for development
///
/// Prints audit events to stdout.
pub struct StdoutBackend;

impl StdoutBackend {
    pub fn new() -> Self {
        Self
    }
}

impl Default for StdoutBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AuditBackend for StdoutBackend {
    async fn write(&self, event: &AuditEvent) -> Result<(), AuditBackendError> {
        let json = event.to_json_pretty()?;
        println!("{}", json);
        Ok(())
    }

    async fn flush(&self) -> Result<(), AuditBackendError> {
        Ok(())
    }
}

/// Multiple backend wrapper
///
/// Writes to multiple backends simultaneously.
pub struct MultiBackend {
    backends: Vec<Box<dyn AuditBackend>>,
}

impl MultiBackend {
    /// Create a new multi-backend
    pub fn new() -> Self {
        Self {
            backends: Vec::new(),
        }
    }

    /// Add a backend
    pub fn with_backend(mut self, backend: Box<dyn AuditBackend>) -> Self {
        self.backends.push(backend);
        self
    }
}

impl Default for MultiBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AuditBackend for MultiBackend {
    async fn write(&self, event: &AuditEvent) -> Result<(), AuditBackendError> {
        for backend in &self.backends {
            backend.write(event).await?;
        }
        Ok(())
    }

    async fn flush(&self) -> Result<(), AuditBackendError> {
        for backend in &self.backends {
            backend.flush().await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AuditSeverity, AuditStatus};

    #[tokio::test]
    async fn test_memory_backend() {
        let backend = MemoryBackend::new();
        let event = AuditEvent::new("test.event").user("alice").action("test");

        backend.write(&event).await.unwrap();

        let events = backend.get_events().await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "test.event");
    }

    #[tokio::test]
    async fn test_memory_backend_read() {
        let backend = MemoryBackend::new();

        for i in 0..5 {
            let event = AuditEvent::new(format!("test.{}", i));
            backend.write(&event).await.unwrap();
        }

        let events = backend.read(3).await.unwrap();
        assert_eq!(events.len(), 3);
    }

    #[tokio::test]
    async fn test_memory_backend_delete_before() {
        let backend = MemoryBackend::new();

        let old_event = AuditEvent::new("old");
        backend.write(&old_event).await.unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let cutoff = chrono::Utc::now();

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let new_event = AuditEvent::new("new");
        backend.write(&new_event).await.unwrap();

        let deleted = backend.delete_before(cutoff).await.unwrap();
        assert_eq!(deleted, 1);

        let events = backend.get_events().await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "new");
    }
}
