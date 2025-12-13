//! Webhook endpoint registry

use crate::{Result, WebhookEndpoint, WebhookError};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Registry for managing webhook endpoints
#[derive(Debug, Clone, Default)]
pub struct WebhookRegistry {
    endpoints: Arc<RwLock<HashMap<String, WebhookEndpoint>>>,
}

impl WebhookRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            endpoints: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new endpoint
    pub fn register(&self, endpoint: WebhookEndpoint) -> String {
        let id = endpoint.id.clone();
        let mut endpoints = self.endpoints.write().unwrap();
        endpoints.insert(id.clone(), endpoint);
        id
    }

    /// Unregister an endpoint by ID
    pub fn unregister(&self, id: &str) -> Option<WebhookEndpoint> {
        let mut endpoints = self.endpoints.write().unwrap();
        endpoints.remove(id)
    }

    /// Get an endpoint by ID
    pub fn get(&self, id: &str) -> Option<WebhookEndpoint> {
        let endpoints = self.endpoints.read().unwrap();
        endpoints.get(id).cloned()
    }

    /// Get a mutable reference to an endpoint (via callback)
    pub fn with_endpoint<F, R>(&self, id: &str, f: F) -> Result<R>
    where
        F: FnOnce(&mut WebhookEndpoint) -> R,
    {
        let mut endpoints = self.endpoints.write().unwrap();
        match endpoints.get_mut(id) {
            Some(endpoint) => Ok(f(endpoint)),
            None => Err(WebhookError::EndpointNotFound(id.to_string())),
        }
    }

    /// Get all endpoints subscribed to an event
    pub fn get_endpoints_for_event(&self, event: &str) -> Vec<WebhookEndpoint> {
        let endpoints = self.endpoints.read().unwrap();
        endpoints
            .values()
            .filter(|e| e.enabled && e.is_subscribed_to(event))
            .cloned()
            .collect()
    }

    /// Get all registered endpoints
    pub fn get_all(&self) -> Vec<WebhookEndpoint> {
        let endpoints = self.endpoints.read().unwrap();
        endpoints.values().cloned().collect()
    }

    /// Get all enabled endpoints
    pub fn get_enabled(&self) -> Vec<WebhookEndpoint> {
        let endpoints = self.endpoints.read().unwrap();
        endpoints.values().filter(|e| e.enabled).cloned().collect()
    }

    /// Get all disabled endpoints
    pub fn get_disabled(&self) -> Vec<WebhookEndpoint> {
        let endpoints = self.endpoints.read().unwrap();
        endpoints.values().filter(|e| !e.enabled).cloned().collect()
    }

    /// Get the number of registered endpoints
    pub fn count(&self) -> usize {
        let endpoints = self.endpoints.read().unwrap();
        endpoints.len()
    }

    /// Check if an endpoint exists
    pub fn exists(&self, id: &str) -> bool {
        let endpoints = self.endpoints.read().unwrap();
        endpoints.contains_key(id)
    }

    /// Update an endpoint
    pub fn update(&self, id: &str, endpoint: WebhookEndpoint) -> Result<()> {
        let mut endpoints = self.endpoints.write().unwrap();
        if endpoints.contains_key(id) {
            endpoints.insert(id.to_string(), endpoint);
            Ok(())
        } else {
            Err(WebhookError::EndpointNotFound(id.to_string()))
        }
    }

    /// Enable an endpoint
    pub fn enable(&self, id: &str) -> Result<()> {
        self.with_endpoint(id, |e| e.enable())
    }

    /// Disable an endpoint
    pub fn disable(&self, id: &str) -> Result<()> {
        self.with_endpoint(id, |e| e.disable())
    }

    /// Record a successful delivery for an endpoint
    pub fn record_success(&self, id: &str) -> Result<()> {
        self.with_endpoint(id, |e| e.record_success())
    }

    /// Record a failed delivery for an endpoint
    pub fn record_failure(&self, id: &str) -> Result<()> {
        self.with_endpoint(id, |e| e.record_failure())
    }

    /// Get endpoints with high failure counts
    pub fn get_failing_endpoints(&self, threshold: u32) -> Vec<WebhookEndpoint> {
        let endpoints = self.endpoints.read().unwrap();
        endpoints
            .values()
            .filter(|e| e.failure_count >= threshold)
            .cloned()
            .collect()
    }

    /// Clear all endpoints
    pub fn clear(&self) {
        let mut endpoints = self.endpoints.write().unwrap();
        endpoints.clear();
    }

    /// Get endpoints by URL pattern
    pub fn find_by_url(&self, url_pattern: &str) -> Vec<WebhookEndpoint> {
        let endpoints = self.endpoints.read().unwrap();
        endpoints
            .values()
            .filter(|e| e.url.contains(url_pattern))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_endpoint(id: &str, events: Vec<&str>) -> WebhookEndpoint {
        WebhookEndpoint::builder(format!("https://example.com/{}", id))
            .id(id)
            .events(events)
            .build()
    }

    #[test]
    fn test_register_and_get() {
        let registry = WebhookRegistry::new();

        let endpoint = create_test_endpoint("test-1", vec!["user.created"]);
        let id = registry.register(endpoint);

        let retrieved = registry.get(&id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, "test-1");
    }

    #[test]
    fn test_unregister() {
        let registry = WebhookRegistry::new();

        let endpoint = create_test_endpoint("test-1", vec![]);
        let id = registry.register(endpoint);

        assert!(registry.exists(&id));
        registry.unregister(&id);
        assert!(!registry.exists(&id));
    }

    #[test]
    fn test_get_endpoints_for_event() {
        let registry = WebhookRegistry::new();

        registry.register(create_test_endpoint(
            "ep-1",
            vec!["user.created", "user.updated"],
        ));
        registry.register(create_test_endpoint("ep-2", vec!["user.created"]));
        registry.register(create_test_endpoint("ep-3", vec!["order.created"]));

        let user_created_endpoints = registry.get_endpoints_for_event("user.created");
        assert_eq!(user_created_endpoints.len(), 2);

        let order_endpoints = registry.get_endpoints_for_event("order.created");
        assert_eq!(order_endpoints.len(), 1);

        let no_endpoints = registry.get_endpoints_for_event("product.created");
        assert!(no_endpoints.is_empty());
    }

    #[test]
    fn test_enable_disable() {
        let registry = WebhookRegistry::new();

        let endpoint = create_test_endpoint("test-1", vec!["user.created"]);
        let id = registry.register(endpoint);

        // Initially enabled
        assert!(registry.get(&id).unwrap().enabled);

        // Disable
        registry.disable(&id).unwrap();
        assert!(!registry.get(&id).unwrap().enabled);

        // Should not appear in event queries when disabled
        let endpoints = registry.get_endpoints_for_event("user.created");
        assert!(endpoints.is_empty());

        // Enable
        registry.enable(&id).unwrap();
        assert!(registry.get(&id).unwrap().enabled);
    }

    #[test]
    fn test_failure_tracking() {
        let registry = WebhookRegistry::new();

        let endpoint = create_test_endpoint("test-1", vec![]);
        let id = registry.register(endpoint);

        // Record failures
        registry.record_failure(&id).unwrap();
        registry.record_failure(&id).unwrap();
        registry.record_failure(&id).unwrap();

        assert_eq!(registry.get(&id).unwrap().failure_count, 3);

        // Get failing endpoints
        let failing = registry.get_failing_endpoints(3);
        assert_eq!(failing.len(), 1);

        // Record success resets count
        registry.record_success(&id).unwrap();
        assert_eq!(registry.get(&id).unwrap().failure_count, 0);
    }

    #[test]
    fn test_find_by_url() {
        let registry = WebhookRegistry::new();

        registry.register(WebhookEndpoint::new("https://api.example.com/webhook"));
        registry.register(WebhookEndpoint::new("https://hooks.example.com/receive"));
        registry.register(WebhookEndpoint::new("https://other.com/webhook"));

        let example_endpoints = registry.find_by_url("example.com");
        assert_eq!(example_endpoints.len(), 2);

        let webhook_endpoints = registry.find_by_url("webhook");
        assert_eq!(webhook_endpoints.len(), 2);
    }

    #[test]
    fn test_count() {
        let registry = WebhookRegistry::new();
        assert_eq!(registry.count(), 0);

        registry.register(create_test_endpoint("1", vec![]));
        registry.register(create_test_endpoint("2", vec![]));
        assert_eq!(registry.count(), 2);

        registry.clear();
        assert_eq!(registry.count(), 0);
    }
}
