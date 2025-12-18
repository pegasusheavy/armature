//! Audit logger

use crate::{AuditBackend, AuditEvent, MaskingConfig};
use std::sync::Arc;

/// Audit logger
///
/// Main interface for logging audit events.
pub struct AuditLogger {
    backend: Arc<dyn AuditBackend>,
    masking_config: MaskingConfig,
    enabled: bool,
}

impl AuditLogger {
    /// Create a new audit logger builder
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use armature_audit::*;
    ///
    /// let logger = AuditLogger::builder()
    ///     .backend(FileBackend::new("audit.log"))
    ///     .build();
    /// ```
    pub fn builder() -> AuditLoggerBuilder {
        AuditLoggerBuilder::new()
    }

    /// Log an audit event
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use armature_audit::*;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let logger = AuditLogger::builder()
    ///     .backend(MemoryBackend::new())
    ///     .build();
    ///
    /// logger.log(AuditEvent::new("user.login")
    ///     .user("alice")
    ///     .action("authenticate")
    ///     .status(AuditStatus::Success)).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn log(
        &self,
        mut event: AuditEvent,
    ) -> Result<(), crate::backend::AuditBackendError> {
        if !self.enabled {
            return Ok(());
        }

        // Apply masking
        if let Some(body) = &event.request_body {
            event.request_body = Some(crate::mask_body(body, &self.masking_config));
        }

        if let Some(body) = &event.response_body {
            event.response_body = Some(crate::mask_body(body, &self.masking_config));
        }

        self.backend.write(&event).await
    }

    /// Flush any pending writes
    pub async fn flush(&self) -> Result<(), crate::backend::AuditBackendError> {
        self.backend.flush().await
    }

    /// Check if logger is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Enable or disable the logger
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

/// Audit logger builder
pub struct AuditLoggerBuilder {
    backend: Option<Arc<dyn AuditBackend>>,
    masking_config: MaskingConfig,
    enabled: bool,
}

impl AuditLoggerBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            backend: None,
            masking_config: MaskingConfig::default(),
            enabled: true,
        }
    }

    /// Set the storage backend
    pub fn backend(mut self, backend: impl AuditBackend + 'static) -> Self {
        self.backend = Some(Arc::new(backend));
        self
    }

    /// Set masking configuration
    pub fn masking_config(mut self, config: MaskingConfig) -> Self {
        self.masking_config = config;
        self
    }

    /// Enable or disable the logger
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Build the audit logger
    pub fn build(self) -> AuditLogger {
        AuditLogger {
            backend: self.backend.expect("Backend must be set"),
            masking_config: self.masking_config,
            enabled: self.enabled,
        }
    }
}

impl Default for AuditLoggerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AuditStatus, MemoryBackend};

    #[tokio::test]
    async fn test_audit_logger() {
        let backend = MemoryBackend::new();
        let backend_clone = backend.clone();

        let logger = AuditLogger::builder().backend(backend).build();

        let event = AuditEvent::new("test.event")
            .user("alice")
            .action("test")
            .status(AuditStatus::Success);

        logger.log(event).await.unwrap();

        let events = backend_clone.get_events().await;
        assert_eq!(events.len(), 1);
    }

    #[tokio::test]
    async fn test_audit_logger_masking() {
        let backend = MemoryBackend::new();
        let backend_clone = backend.clone();

        let logger = AuditLogger::builder().backend(backend).build();

        let event = AuditEvent::new("test.event")
            .request_body(r#"{"username":"alice","password":"secret123"}"#);

        logger.log(event).await.unwrap();

        let events = backend_clone.get_events().await;
        assert_eq!(events.len(), 1);

        let body = events[0].request_body.as_ref().unwrap();
        assert!(body.contains("alice"));
        assert!(!body.contains("secret123"));
    }

    #[tokio::test]
    async fn test_audit_logger_disabled() {
        let backend = MemoryBackend::new();
        let backend_clone = backend.clone();

        let logger = AuditLogger::builder()
            .backend(backend)
            .enabled(false)
            .build();

        let event = AuditEvent::new("test.event");
        logger.log(event).await.unwrap();

        let events = backend_clone.get_events().await;
        assert_eq!(events.len(), 0);
    }
}
