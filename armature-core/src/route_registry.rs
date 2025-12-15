//! Route registry for compile-time route collection using inventory
//!
//! This module provides a mechanism for route macros to register routes
//! that can be collected at runtime by controllers.

use crate::{Error, HttpRequest, HttpResponse};
use std::any::TypeId;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Type alias for async route handler functions
pub type RouteHandlerFn = Arc<
    dyn Fn(HttpRequest) -> Pin<Box<dyn Future<Output = Result<HttpResponse, Error>> + Send>>
        + Send
        + Sync,
>;

/// A route entry that can be collected via inventory
pub struct RouteEntry {
    /// The type ID of the controller this route belongs to
    pub controller_type_id: TypeId,
    /// The controller type name (for debugging)
    pub controller_type_name: &'static str,
    /// HTTP method (GET, POST, etc.)
    pub method: &'static str,
    /// Route path (e.g., "/hello", "/:id")
    pub path: &'static str,
    /// Handler function name (for debugging)
    pub handler_name: &'static str,
    /// The actual handler function
    pub handler: RouteHandlerFn,
}

// Register RouteEntry with inventory for compile-time collection
inventory::collect!(RouteEntry);

impl RouteEntry {
    /// Create a new route entry
    pub fn new<C: 'static>(
        method: &'static str,
        path: &'static str,
        handler_name: &'static str,
        handler: RouteHandlerFn,
    ) -> Self {
        Self {
            controller_type_id: TypeId::of::<C>(),
            controller_type_name: std::any::type_name::<C>(),
            method,
            path,
            handler_name,
            handler,
        }
    }
}

/// Get all registered routes for a specific controller type
pub fn get_routes_for_controller<C: 'static>() -> Vec<&'static RouteEntry> {
    let target_type_id = TypeId::of::<C>();
    inventory::iter::<RouteEntry>
        .into_iter()
        .filter(|entry| entry.controller_type_id == target_type_id)
        .collect()
}

/// Get all registered routes for a controller by type ID
pub fn get_routes_by_type_id(type_id: TypeId) -> Vec<&'static RouteEntry> {
    inventory::iter::<RouteEntry>
        .into_iter()
        .filter(|entry| entry.controller_type_id == type_id)
        .collect()
}

/// Macro to register a route handler with the inventory
///
/// This is used internally by the route macros (#[get], #[post], etc.)
#[macro_export]
macro_rules! register_route {
    ($controller:ty, $method:expr, $path:expr, $handler_name:expr, $handler:expr) => {
        inventory::submit! {
            $crate::route_registry::RouteEntry::new::<$controller>(
                $method,
                $path,
                $handler_name,
                std::sync::Arc::new($handler),
            )
        }
    };
}

