// Test Application Builder

use armature_core::{Application, Container, Module, Provider, Router};
use std::sync::Arc;

/// Test application for integration testing
pub struct TestApp {
    pub app: Application,
    pub container: Arc<Container>,
}

impl TestApp {
    /// Create a new test application
    pub fn new(container: Container, router: Router) -> Self {
        let container_arc = Arc::new(container);
        let app = Application::new(Container::new(), router);
        Self {
            app,
            container: container_arc,
        }
    }

    /// Get a service from the container
    pub fn get<T: Provider + Clone + 'static>(&self) -> Option<T> {
        // This would need actual container implementation
        None
    }

    /// Create a test client for making requests
    pub fn client(&self) -> crate::TestClient {
        crate::TestClient::new(self.app.router.clone())
    }
}

/// Builder for test applications
pub struct TestAppBuilder {
    container: Container,
    router: Router,
}

impl TestAppBuilder {
    /// Create a new test app builder
    pub fn new() -> Self {
        Self {
            container: Container::new(),
            router: Router::new(),
        }
    }

    /// Register a provider
    pub fn register<T: Provider + Clone + 'static>(self, provider: T) -> Self {
        self.container.register(provider);
        self
    }

    /// Add a module
    pub fn add_module<M: Module>(self, _module: M) -> Self {
        // Module registration logic
        self
    }

    /// Set custom container
    pub fn with_container(self, container: Container) -> Self {
        Self { container, ..self }
    }

    /// Set custom router
    pub fn with_router(self, router: Router) -> Self {
        Self { router, ..self }
    }

    /// Build the test application
    pub fn build(self) -> TestApp {
        TestApp::new(self.container, self.router)
    }
}

impl Default for TestAppBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_creation() {
        let _builder = TestAppBuilder::new();
    }

    #[test]
    fn test_app_creation() {
        let builder = TestAppBuilder::new();
        let _app = builder.build();
    }
}
