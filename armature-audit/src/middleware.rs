//! Request/Response logging middleware

use crate::{AuditEvent, AuditLogger, AuditSeverity, AuditStatus};
use armature_core::{Error, HttpRequest, HttpResponse, Middleware};
use std::sync::Arc;
use std::time::Instant;

/// Request/Response audit logging middleware
///
/// Automatically logs HTTP requests and responses.
pub struct AuditMiddleware {
    logger: Arc<AuditLogger>,
    log_request_body: bool,
    log_response_body: bool,
    max_body_size: usize,
}

impl AuditMiddleware {
    /// Create a new audit middleware
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use armature_audit::*;
    /// use std::sync::Arc;
    ///
    /// let logger = Arc::new(AuditLogger::builder()
    ///     .backend(FileBackend::new("audit.log"))
    ///     .build());
    ///
    /// let middleware = AuditMiddleware::new(logger);
    /// ```
    pub fn new(logger: Arc<AuditLogger>) -> Self {
        Self {
            logger,
            log_request_body: true,
            log_response_body: true,
            max_body_size: 10_000, // 10KB default
        }
    }

    /// Set whether to log request bodies
    pub fn log_request_body(mut self, log: bool) -> Self {
        self.log_request_body = log;
        self
    }

    /// Set whether to log response bodies
    pub fn log_response_body(mut self, log: bool) -> Self {
        self.log_response_body = log;
        self
    }

    /// Set maximum body size to log (in bytes)
    pub fn max_body_size(mut self, size: usize) -> Self {
        self.max_body_size = size;
        self
    }

    /// Extract user ID from request (can be customized)
    fn extract_user_id(&self, request: &HttpRequest) -> Option<String> {
        // Try to get from Authorization header
        request.headers.get("authorization").and_then(|auth| {
            if auth.starts_with("Bearer ") {
                Some("authenticated_user".to_string())
            } else {
                None
            }
        })
    }

    /// Extract IP address from request
    fn extract_ip(&self, request: &HttpRequest) -> Option<String> {
        // Try X-Forwarded-For first
        if let Some(forwarded) = request.headers.get("x-forwarded-for") {
            return Some(
                forwarded
                    .split(',')
                    .next()
                    .unwrap_or(forwarded)
                    .trim()
                    .to_string(),
            );
        }

        // Try X-Real-IP
        if let Some(real_ip) = request.headers.get("x-real-ip") {
            return Some(real_ip.clone());
        }

        None
    }

    /// Extract user agent from request
    fn extract_user_agent(&self, request: &HttpRequest) -> Option<String> {
        request.headers.get("user-agent").cloned()
    }

    /// Truncate body if too large
    fn truncate_body(&self, body: &[u8]) -> Option<String> {
        if body.is_empty() {
            return None;
        }

        if body.len() > self.max_body_size {
            let truncated = &body[..self.max_body_size];
            let mut text = String::from_utf8_lossy(truncated).to_string();
            text.push_str("... [TRUNCATED]");
            Some(text)
        } else {
            Some(String::from_utf8_lossy(body).to_string())
        }
    }
}

#[async_trait::async_trait]
impl Middleware for AuditMiddleware {
    async fn handle(
        &self,
        request: HttpRequest,
        next: armature_core::middleware::Next,
    ) -> Result<HttpResponse, Error> {
        let start = Instant::now();

        // Extract request information
        let method = request.method.clone();
        let path = request.path.clone();
        let user_id = self.extract_user_id(&request);
        let ip_address = self.extract_ip(&request);
        let user_agent = self.extract_user_agent(&request);

        // Optionally log request body
        let request_body = if self.log_request_body {
            self.truncate_body(&request.body)
        } else {
            None
        };

        // Process request
        let result = next(request).await;

        // Calculate duration
        let duration_ms = start.elapsed().as_millis() as u64;

        // Create audit event based on result
        let event = match &result {
            Ok(response) => {
                let status_code = response.status;
                let response_body = if self.log_response_body {
                    self.truncate_body(&response.body)
                } else {
                    None
                };

                let status = if status_code < 400 {
                    AuditStatus::Success
                } else if status_code < 500 {
                    AuditStatus::Denied
                } else {
                    AuditStatus::Error
                };

                let severity = if status_code < 400 {
                    AuditSeverity::Info
                } else if status_code < 500 {
                    AuditSeverity::Warning
                } else {
                    AuditSeverity::Error
                };

                let mut event = AuditEvent::new("http.request")
                    .action("http_request")
                    .method(method)
                    .path(path)
                    .status_code(status_code)
                    .status(status)
                    .severity(severity)
                    .duration_ms(duration_ms);

                if let Some(user) = user_id {
                    event = event.user(user);
                }
                if let Some(ip) = ip_address {
                    event = event.ip(ip);
                }
                if let Some(ua) = user_agent {
                    event = event.user_agent(ua);
                }
                if let Some(body) = request_body {
                    event = event.request_body(body);
                }
                if let Some(body) = response_body {
                    event = event.response_body(body);
                }

                event
            }
            Err(err) => {
                let status_code = err.status_code();

                let mut event = AuditEvent::new("http.request")
                    .action("http_request")
                    .method(method)
                    .path(path)
                    .status_code(status_code)
                    .status(AuditStatus::Error)
                    .severity(AuditSeverity::Error)
                    .duration_ms(duration_ms)
                    .error(err.to_string());

                if let Some(user) = user_id {
                    event = event.user(user);
                }
                if let Some(ip) = ip_address {
                    event = event.ip(ip);
                }
                if let Some(ua) = user_agent {
                    event = event.user_agent(ua);
                }
                if let Some(body) = request_body {
                    event = event.request_body(body);
                }

                event
            }
        };

        // Log audit event (don't fail request if logging fails)
        if let Err(e) = self.logger.log(event).await {
            tracing::error!("Failed to log audit event: {}", e);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AuditLogger, MemoryBackend};

    #[test]
    fn test_audit_middleware_creation() {
        let logger = Arc::new(AuditLogger::builder().backend(MemoryBackend::new()).build());

        let middleware = AuditMiddleware::new(logger);
        assert!(middleware.log_request_body);
        assert!(middleware.log_response_body);
    }

    #[test]
    fn test_audit_middleware_configuration() {
        let logger = Arc::new(AuditLogger::builder().backend(MemoryBackend::new()).build());

        let middleware = AuditMiddleware::new(logger)
            .log_request_body(false)
            .log_response_body(false)
            .max_body_size(5000);

        assert!(!middleware.log_request_body);
        assert!(!middleware.log_response_body);
        assert_eq!(middleware.max_body_size, 5000);
    }

    #[test]
    fn test_truncate_body() {
        let logger = Arc::new(AuditLogger::builder().backend(MemoryBackend::new()).build());

        let middleware = AuditMiddleware::new(logger).max_body_size(10);

        let body = b"This is a very long body that should be truncated";
        let truncated = middleware.truncate_body(body).unwrap();

        assert!(truncated.len() <= 30); // 10 + "... [TRUNCATED]"
        assert!(truncated.contains("[TRUNCATED]"));
    }
}
