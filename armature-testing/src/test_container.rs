// Test Container for Dependency Injection

use armature_core::{Container, Provider};

/// Test container with enhanced testing capabilities
pub struct TestContainer {
    container: Container,
}

impl TestContainer {
    /// Create a new test container
    pub fn new() -> Self {
        Self {
            container: Container::new(),
        }
    }

    /// Register a provider
    pub fn register<T: Provider + Clone + 'static>(&self, provider: T) {
        self.container.register(provider);
    }

    /// Register a mock provider
    pub fn register_mock<T: Provider + Clone + 'static>(&self, mock: T) {
        self.container.register(mock);
    }

    /// Get a provider from the container
    pub fn get<T: Provider + Clone + 'static>(&self) -> Option<T> {
        // This would need actual container implementation
        None
    }

    /// Clear all providers
    pub fn clear(&mut self) {
        self.container = Container::new();
    }

    /// Get the underlying container
    pub fn inner(&self) -> &Container {
        &self.container
    }
}

impl Default for TestContainer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_creation() {
        let _container = TestContainer::new();
    }
}
