//! Audit logging and compliance for Armature
//!
//! This crate provides comprehensive audit logging for security, compliance,
//! and operational tracking.
//!
//! # Features
//!
//! - **Audit Events** - Structured audit event logging
//! - **Request/Response Logging** - HTTP payload logging
//! - **Data Masking** - Automatic PII/sensitive data masking
//! - **Retention Policies** - Automatic log cleanup
//! - **Multiple Backends** - File, JSON, database storage
//! - **Filtering** - Configurable event filtering
//!
//! # Quick Start
//!
//! ```no_run
//! use armature_audit::*;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create audit logger
//! let audit = AuditLogger::builder()
//!     .backend(FileBackend::new("audit.log"))
//!     .build();
//!
//! // Log an event
//! audit.log(AuditEvent::new("user.login")
//!     .user("alice")
//!     .resource("system")
//!     .action("authenticate")
//!     .status(AuditStatus::Success)).await?;
//! # Ok(())
//! # }
//! ```

pub mod backend;
pub mod event;
pub mod logger;
pub mod masking;
pub mod middleware;
pub mod retention;

pub use backend::*;
pub use event::*;
pub use logger::*;
pub use masking::*;
pub use middleware::*;
pub use retention::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Just ensure all exports are accessible
        let _ = AuditEvent::new("test");
    }
}

