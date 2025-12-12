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
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use armature_testing::TestAppBuilder;
    /// use armature_core::HttpResponse;
    ///
    /// # tokio_test::block_on(async {
    /// let app = TestAppBuilder::new()
    ///     .with_route("/test", |_req| async {
    ///         Ok(HttpResponse::ok().with_body(b"OK".to_vec()))
    ///     })
    ///     .build();
    ///
    /// let client = app.client();
    /// let response = client.get("/test").await;
    /// assert_eq!(response.status(), Some(200));
    /// # });
    /// ```
    pub fn client(&self) -> crate::TestClient {
        crate::TestClient::new(self.app.router.clone())
    }
}

/// Builder for test applications.
///
/// Provides a fluent API for creating test applications with custom
/// routes, services, and configuration.
///
/// # Examples
///
/// Basic test app:
///
/// ```no_run
/// use armature_testing::TestAppBuilder;
/// use armature_core::{HttpResponse, Error};
///
/// # tokio_test::block_on(async {
/// let app = TestAppBuilder::new()
///     .with_route("/test", |_req| async {
///         Ok(HttpResponse::ok().with_body(b"test".to_vec()))
///     })
///     .build();
///
/// let client = app.client();
/// let response = client.get("/test").await;
/// assert_eq!(response.status(), Some(200));
/// # });
/// ```
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

    /// Add a test route with a handler
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use armature_testing::TestAppBuilder;
    /// use armature_core::HttpResponse;
    ///
    /// let app = TestAppBuilder::new()
    ///     .with_route("/api/health", |_req| async {
    ///         Ok(HttpResponse::ok().with_body(b"OK".to_vec()))
    ///     })
    ///     .build();
    /// ```
    pub fn with_route<F, Fut>(mut self, path: &str, handler: F) -> Self
    where
        F: Fn(armature_core::HttpRequest) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<armature_core::HttpResponse, armature_core::Error>>
            + Send
            + 'static,
    {
        use armature_core::{HttpMethod, Route};
        self.router.add_route(Route {
            method: HttpMethod::GET,
            path: path.to_string(),
            handler: Arc::new(move |req| Box::pin(handler(req))),
        });
        self
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
