//! Webhook endpoint configuration

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

/// A registered webhook endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEndpoint {
    /// Unique endpoint ID
    pub id: String,

    /// Target URL for webhook delivery
    pub url: String,

    /// Signing secret for this endpoint
    #[serde(skip_serializing)]
    pub secret: String,

    /// Events this endpoint is subscribed to
    pub events: HashSet<String>,

    /// Whether this endpoint is active
    pub enabled: bool,

    /// Description for this endpoint
    pub description: Option<String>,

    /// Custom headers to include with requests
    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,

    /// Created timestamp
    pub created_at: DateTime<Utc>,

    /// Updated timestamp
    pub updated_at: DateTime<Utc>,

    /// Last successful delivery timestamp
    pub last_success: Option<DateTime<Utc>>,

    /// Last failed delivery timestamp
    pub last_failure: Option<DateTime<Utc>>,

    /// Consecutive failure count
    pub failure_count: u32,
}

impl WebhookEndpoint {
    /// Create a new endpoint with the given URL
    pub fn new(url: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            url: url.into(),
            secret: generate_secret(),
            events: HashSet::new(),
            enabled: true,
            description: None,
            headers: std::collections::HashMap::new(),
            created_at: now,
            updated_at: now,
            last_success: None,
            last_failure: None,
            failure_count: 0,
        }
    }

    /// Create a builder for custom configuration
    pub fn builder(url: impl Into<String>) -> WebhookEndpointBuilder {
        WebhookEndpointBuilder::new(url)
    }

    /// Set the signing secret
    pub fn with_secret(mut self, secret: impl Into<String>) -> Self {
        self.secret = secret.into();
        self
    }

    /// Subscribe to specific events
    pub fn with_events(mut self, events: Vec<&str>) -> Self {
        self.events = events.into_iter().map(String::from).collect();
        self
    }

    /// Add a single event subscription
    pub fn subscribe(&mut self, event: impl Into<String>) {
        self.events.insert(event.into());
    }

    /// Remove an event subscription
    pub fn unsubscribe(&mut self, event: &str) {
        self.events.remove(event);
    }

    /// Check if this endpoint is subscribed to an event
    pub fn is_subscribed_to(&self, event: &str) -> bool {
        // Check exact match
        if self.events.contains(event) {
            return true;
        }

        // Check wildcard subscriptions (e.g., "user.*" matches "user.created")
        for subscribed in &self.events {
            if subscribed.ends_with(".*") {
                let prefix = &subscribed[..subscribed.len() - 2];
                if event.starts_with(prefix) {
                    return true;
                }
            } else if subscribed == "*" {
                return true;
            }
        }

        false
    }

    /// Record a successful delivery
    pub fn record_success(&mut self) {
        self.last_success = Some(Utc::now());
        self.failure_count = 0;
        self.updated_at = Utc::now();
    }

    /// Record a failed delivery
    pub fn record_failure(&mut self) {
        self.last_failure = Some(Utc::now());
        self.failure_count += 1;
        self.updated_at = Utc::now();
    }

    /// Disable the endpoint
    pub fn disable(&mut self) {
        self.enabled = false;
        self.updated_at = Utc::now();
    }

    /// Enable the endpoint
    pub fn enable(&mut self) {
        self.enabled = true;
        self.failure_count = 0;
        self.updated_at = Utc::now();
    }

    /// Regenerate the signing secret
    pub fn rotate_secret(&mut self) -> String {
        self.secret = generate_secret();
        self.updated_at = Utc::now();
        self.secret.clone()
    }
}

/// Builder for WebhookEndpoint
#[derive(Debug, Clone)]
pub struct WebhookEndpointBuilder {
    endpoint: WebhookEndpoint,
}

impl WebhookEndpointBuilder {
    /// Create a new builder
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            endpoint: WebhookEndpoint::new(url),
        }
    }

    /// Set a custom ID
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.endpoint.id = id.into();
        self
    }

    /// Set the signing secret
    pub fn secret(mut self, secret: impl Into<String>) -> Self {
        self.endpoint.secret = secret.into();
        self
    }

    /// Subscribe to events
    pub fn events(mut self, events: Vec<&str>) -> Self {
        self.endpoint.events = events.into_iter().map(String::from).collect();
        self
    }

    /// Subscribe to all events
    pub fn all_events(mut self) -> Self {
        self.endpoint.events.insert("*".to_string());
        self
    }

    /// Set the description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.endpoint.description = Some(desc.into());
        self
    }

    /// Add a custom header
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.endpoint.headers.insert(key.into(), value.into());
        self
    }

    /// Set enabled status
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.endpoint.enabled = enabled;
        self
    }

    /// Build the endpoint
    pub fn build(self) -> WebhookEndpoint {
        self.endpoint
    }
}

/// Generate a random signing secret
fn generate_secret() -> String {
    use std::time::SystemTime;

    // Generate a pseudo-random secret using UUID and timestamp
    let uuid = Uuid::new_v4();
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    format!("whsec_{}_{:x}", uuid.simple(), timestamp % 0xFFFFFFFF)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endpoint_creation() {
        let endpoint = WebhookEndpoint::new("https://example.com/webhook");

        assert!(!endpoint.id.is_empty());
        assert!(endpoint.secret.starts_with("whsec_"));
        assert!(endpoint.enabled);
        assert!(endpoint.events.is_empty());
    }

    #[test]
    fn test_endpoint_builder() {
        let endpoint = WebhookEndpoint::builder("https://example.com/webhook")
            .secret("custom-secret")
            .events(vec!["user.created", "user.updated"])
            .description("My webhook")
            .header("X-Custom", "value")
            .build();

        assert_eq!(endpoint.secret, "custom-secret");
        assert!(endpoint.events.contains("user.created"));
        assert!(endpoint.events.contains("user.updated"));
        assert_eq!(endpoint.description, Some("My webhook".to_string()));
        assert_eq!(endpoint.headers.get("X-Custom"), Some(&"value".to_string()));
    }

    #[test]
    fn test_event_subscription() {
        let mut endpoint = WebhookEndpoint::new("https://example.com");

        endpoint.subscribe("user.created");
        assert!(endpoint.is_subscribed_to("user.created"));
        assert!(!endpoint.is_subscribed_to("user.deleted"));

        endpoint.unsubscribe("user.created");
        assert!(!endpoint.is_subscribed_to("user.created"));
    }

    #[test]
    fn test_wildcard_subscription() {
        let endpoint = WebhookEndpoint::builder("https://example.com")
            .events(vec!["user.*", "order.created"])
            .build();

        // Wildcard should match
        assert!(endpoint.is_subscribed_to("user.created"));
        assert!(endpoint.is_subscribed_to("user.updated"));
        assert!(endpoint.is_subscribed_to("user.deleted"));

        // Exact match
        assert!(endpoint.is_subscribed_to("order.created"));

        // No match
        assert!(!endpoint.is_subscribed_to("order.updated"));
        assert!(!endpoint.is_subscribed_to("product.created"));
    }

    #[test]
    fn test_all_events_subscription() {
        let endpoint = WebhookEndpoint::builder("https://example.com")
            .all_events()
            .build();

        assert!(endpoint.is_subscribed_to("anything"));
        assert!(endpoint.is_subscribed_to("user.created"));
        assert!(endpoint.is_subscribed_to("order.shipped"));
    }

    #[test]
    fn test_failure_tracking() {
        let mut endpoint = WebhookEndpoint::new("https://example.com");

        endpoint.record_failure();
        assert_eq!(endpoint.failure_count, 1);
        assert!(endpoint.last_failure.is_some());

        endpoint.record_failure();
        assert_eq!(endpoint.failure_count, 2);

        endpoint.record_success();
        assert_eq!(endpoint.failure_count, 0);
        assert!(endpoint.last_success.is_some());
    }

    #[test]
    fn test_secret_rotation() {
        let mut endpoint = WebhookEndpoint::new("https://example.com");
        let old_secret = endpoint.secret.clone();

        let new_secret = endpoint.rotate_secret();

        assert_ne!(old_secret, new_secret);
        assert_eq!(endpoint.secret, new_secret);
    }
}
