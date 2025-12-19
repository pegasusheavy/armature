//! Audit event structures and types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Audit event status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuditStatus {
    /// Operation succeeded
    Success,
    /// Operation failed
    Failure,
    /// Operation denied
    Denied,
    /// Operation resulted in error
    Error,
}

/// Severity level for audit events
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuditSeverity {
    /// Informational events
    Info,
    /// Warning events
    Warning,
    /// Error events
    Error,
    /// Critical events
    Critical,
}

/// Audit event structure
///
/// Contains all information about an auditable event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event ID
    pub id: String,

    /// Timestamp when event occurred
    pub timestamp: DateTime<Utc>,

    /// Event type (e.g., "user.login", "resource.delete")
    pub event_type: String,

    /// User who performed the action
    pub user_id: Option<String>,

    /// User's IP address
    pub ip_address: Option<String>,

    /// User agent string
    pub user_agent: Option<String>,

    /// Resource being acted upon
    pub resource_type: Option<String>,

    /// Resource identifier
    pub resource_id: Option<String>,

    /// Action performed
    pub action: String,

    /// Status of the operation
    pub status: AuditStatus,

    /// Severity level
    pub severity: AuditSeverity,

    /// HTTP method (if applicable)
    pub method: Option<String>,

    /// Request path (if applicable)
    pub path: Option<String>,

    /// HTTP status code (if applicable)
    pub status_code: Option<u16>,

    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,

    /// Error message (if applicable)
    pub error: Option<String>,

    /// Request body (masked)
    pub request_body: Option<String>,

    /// Response body (masked)
    pub response_body: Option<String>,

    /// Duration in milliseconds
    pub duration_ms: Option<u64>,
}

impl AuditEvent {
    /// Create a new audit event
    ///
    /// # Examples
    ///
    /// ```
    /// use armature_audit::*;
    ///
    /// let event = AuditEvent::new("user.login")
    ///     .user("alice")
    ///     .action("authenticate")
    ///     .status(AuditStatus::Success);
    /// ```
    pub fn new(event_type: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type: event_type.into(),
            user_id: None,
            ip_address: None,
            user_agent: None,
            resource_type: None,
            resource_id: None,
            action: "unknown".to_string(),
            status: AuditStatus::Success,
            severity: AuditSeverity::Info,
            method: None,
            path: None,
            status_code: None,
            metadata: HashMap::new(),
            error: None,
            request_body: None,
            response_body: None,
            duration_ms: None,
        }
    }

    /// Set user ID
    pub fn user(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Set IP address
    pub fn ip(mut self, ip: impl Into<String>) -> Self {
        self.ip_address = Some(ip.into());
        self
    }

    /// Set user agent
    pub fn user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = Some(ua.into());
        self
    }

    /// Set resource type
    pub fn resource(mut self, resource_type: impl Into<String>) -> Self {
        self.resource_type = Some(resource_type.into());
        self
    }

    /// Set resource ID
    pub fn resource_id(mut self, id: impl Into<String>) -> Self {
        self.resource_id = Some(id.into());
        self
    }

    /// Set action
    pub fn action(mut self, action: impl Into<String>) -> Self {
        self.action = action.into();
        self
    }

    /// Set status
    pub fn status(mut self, status: AuditStatus) -> Self {
        self.status = status;
        self
    }

    /// Set severity
    pub fn severity(mut self, severity: AuditSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Set HTTP method
    pub fn method(mut self, method: impl Into<String>) -> Self {
        self.method = Some(method.into());
        self
    }

    /// Set request path
    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Set HTTP status code
    pub fn status_code(mut self, code: u16) -> Self {
        self.status_code = Some(code);
        self
    }

    /// Add metadata
    pub fn metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Set error message
    pub fn error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }

    /// Set request body
    pub fn request_body(mut self, body: impl Into<String>) -> Self {
        self.request_body = Some(body.into());
        self
    }

    /// Set response body
    pub fn response_body(mut self, body: impl Into<String>) -> Self {
        self.response_body = Some(body.into());
        self
    }

    /// Set duration
    pub fn duration_ms(mut self, duration: u64) -> Self {
        self.duration_ms = Some(duration);
        self
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Convert to pretty JSON string
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_event_creation() {
        let event = AuditEvent::new("test.event")
            .user("alice")
            .action("test")
            .status(AuditStatus::Success);

        assert_eq!(event.event_type, "test.event");
        assert_eq!(event.user_id, Some("alice".to_string()));
        assert_eq!(event.action, "test");
        assert_eq!(event.status, AuditStatus::Success);
    }

    #[test]
    fn test_audit_event_to_json() {
        let event = AuditEvent::new("test.event");
        let json = event.to_json();
        assert!(json.is_ok());
    }

    #[test]
    fn test_audit_severity_ordering() {
        assert!(AuditSeverity::Info < AuditSeverity::Warning);
        assert!(AuditSeverity::Warning < AuditSeverity::Error);
        assert!(AuditSeverity::Error < AuditSeverity::Critical);
    }
}
