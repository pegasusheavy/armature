// Core traits for the Armature framework

use async_trait::async_trait;
use std::any::TypeId;
use std::fmt::Debug;

/// Trait for types that can be provided by the DI container
pub trait Provider: Send + Sync + 'static {
    /// Returns the TypeId of the provider
    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

/// Trait for HTTP controllers
#[async_trait]
pub trait Controller: Send + Sync + 'static {
    /// Returns the base path for this controller
    fn base_path(&self) -> &'static str;

    /// Returns the routes registered on this controller
    fn routes(&self) -> Vec<RouteDefinition>;
}

/// Trait for modules that organize components
pub trait Module: Send + Sync + 'static {
    /// Returns the list of provider types to register
    fn providers(&self) -> Vec<ProviderRegistration>;

    /// Returns the list of controller types to register
    fn controllers(&self) -> Vec<ControllerRegistration>;

    /// Returns the list of imported modules
    fn imports(&self) -> Vec<Box<dyn Module>>;

    /// Returns the list of exported provider types
    fn exports(&self) -> Vec<TypeId>;
}

/// Trait for request handlers (route methods)
#[async_trait]
pub trait RequestHandler: Send + Sync {
    /// Handle an HTTP request and return a response
    async fn handle(
        &self,
        request: crate::HttpRequest,
    ) -> Result<crate::HttpResponse, crate::Error>;
}

/// Trait for validators
pub trait Validator: Send + Sync {
    /// Validate a value
    fn validate(&self, value: &str) -> Result<(), String>;
}

/// Definition of a route
#[derive(Clone, Debug)]
pub struct RouteDefinition {
    pub method: HttpMethod,
    pub path: String,
    pub handler_name: String,
}

/// HTTP methods
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
}

impl HttpMethod {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "GET" => Some(HttpMethod::GET),
            "POST" => Some(HttpMethod::POST),
            "PUT" => Some(HttpMethod::PUT),
            "DELETE" => Some(HttpMethod::DELETE),
            "PATCH" => Some(HttpMethod::PATCH),
            "HEAD" => Some(HttpMethod::HEAD),
            "OPTIONS" => Some(HttpMethod::OPTIONS),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::PATCH => "PATCH",
            HttpMethod::HEAD => "HEAD",
            HttpMethod::OPTIONS => "OPTIONS",
        }
    }
}

/// Registration information for a provider
#[derive(Clone)]
pub struct ProviderRegistration {
    pub type_id: TypeId,
    pub type_name: &'static str,
    pub register_fn: fn(&crate::Container),
}

impl std::fmt::Debug for ProviderRegistration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProviderRegistration")
            .field("type_id", &self.type_id)
            .field("type_name", &self.type_name)
            .finish()
    }
}

/// Registration information for a controller
#[derive(Clone)]
pub struct ControllerRegistration {
    pub type_id: TypeId,
    pub type_name: &'static str,
    pub base_path: &'static str,
    pub factory:
        fn(&crate::Container) -> Result<Box<dyn std::any::Any + Send + Sync>, crate::Error>,
    #[allow(clippy::type_complexity)]
    pub route_registrar: fn(
        &crate::Container,
        &mut crate::Router,
        Box<dyn std::any::Any + Send + Sync>,
    ) -> Result<(), crate::Error>,
}

impl std::fmt::Debug for ControllerRegistration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ControllerRegistration")
            .field("type_id", &self.type_id)
            .field("type_name", &self.type_name)
            .field("base_path", &self.base_path)
            .finish()
    }
}
