// Mock utilities for testing

use armature_core::Provider;
use std::sync::{Arc, Mutex};

/// Mock service for testing
#[derive(Clone)]
pub struct MockService<T> {
    calls: Arc<Mutex<Vec<String>>>,
    return_value: Arc<Mutex<Option<T>>>,
}

impl<T> MockService<T> {
    /// Create a new mock service
    pub fn new() -> Self {
        Self {
            calls: Arc::new(Mutex::new(Vec::new())),
            return_value: Arc::new(Mutex::new(None)),
        }
    }

    /// Set the return value
    pub fn with_return(self, value: T) -> Self {
        *self.return_value.lock().unwrap() = Some(value);
        self
    }

    /// Record a method call
    pub fn record_call(&self, method: &str) {
        self.calls.lock().unwrap().push(method.to_string());
    }

    /// Get the number of calls
    pub fn call_count(&self) -> usize {
        self.calls.lock().unwrap().len()
    }

    /// Get all recorded calls
    pub fn get_calls(&self) -> Vec<String> {
        self.calls.lock().unwrap().clone()
    }

    /// Check if a method was called
    pub fn was_called(&self, method: &str) -> bool {
        self.calls.lock().unwrap().contains(&method.to_string())
    }

    /// Clear all recorded calls
    pub fn clear_calls(&self) {
        self.calls.lock().unwrap().clear();
    }

    /// Get the mock return value
    pub fn get_return(&self) -> Option<T>
    where
        T: Clone,
    {
        self.return_value.lock().unwrap().clone()
    }
}

impl<T> Default for MockService<T> {
    fn default() -> Self {
        Self::new()
    }
}

type CallLog = Arc<Mutex<Vec<(String, Vec<String>)>>>;

/// Mock controller for testing
#[derive(Clone)]
pub struct MockController {
    _name: String,
    calls: CallLog,
}

impl MockController {
    /// Create a new mock controller
    pub fn new(name: &str) -> Self {
        Self {
            _name: name.to_string(),
            calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Record a method call with arguments
    pub fn record_call(&self, method: &str, args: Vec<String>) {
        self.calls.lock().unwrap().push((method.to_string(), args));
    }

    /// Get the number of calls to a specific method
    pub fn method_call_count(&self, method: &str) -> usize {
        self.calls
            .lock()
            .unwrap()
            .iter()
            .filter(|(m, _)| m == method)
            .count()
    }

    /// Get all calls
    pub fn get_all_calls(&self) -> Vec<(String, Vec<String>)> {
        self.calls.lock().unwrap().clone()
    }

    /// Clear all calls
    pub fn clear(&self) {
        self.calls.lock().unwrap().clear();
    }
}

/// Mock provider trait implementation
pub trait MockProvider: Provider + Clone {
    /// Reset the mock to its initial state
    fn reset(&mut self);

    /// Get the number of calls
    fn call_count(&self) -> usize;
}

/// Spy wrapper for providers
#[allow(dead_code)]
#[derive(Clone)]
pub struct Spy<T: Clone> {
    inner: T,
    calls: Arc<Mutex<Vec<String>>>,
}

impl<T: Clone> Spy<T> {
    /// Create a new spy wrapping a provider
    #[allow(dead_code)]
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Record a method call
    #[allow(dead_code)]
    pub fn record(&self, method: &str) {
        self.calls.lock().unwrap().push(method.to_string());
    }

    /// Get the number of calls
    #[allow(dead_code)]
    pub fn call_count(&self) -> usize {
        self.calls.lock().unwrap().len()
    }

    /// Check if a method was called
    #[allow(dead_code)]
    pub fn was_called(&self, method: &str) -> bool {
        self.calls.lock().unwrap().contains(&method.to_string())
    }

    /// Get the wrapped provider
    #[allow(dead_code)]
    pub fn inner(&self) -> &T {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_service() {
        let mock = MockService::<String>::new();
        mock.record_call("test_method");
        assert_eq!(mock.call_count(), 1);
        assert!(mock.was_called("test_method"));
    }

    #[test]
    fn test_mock_controller() {
        let mock = MockController::new("TestController");
        mock.record_call("get", vec!["id".to_string()]);
        assert_eq!(mock.method_call_count("get"), 1);
    }

    #[test]
    fn test_spy() {
        let spy = Spy::new("test_value");
        spy.record("method1");
        spy.record("method2");
        assert_eq!(spy.call_count(), 2);
        assert!(spy.was_called("method1"));
    }
}
