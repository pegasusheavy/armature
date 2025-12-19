//! Route registry for compile-time route collection using inventory
//!
//! This module provides a mechanism for route macros to register routes
//! that can be collected at runtime by controllers.
//!
//! ## Optimized Handler Dispatch
//!
//! Routes registered via this registry support the optimized handler system
//! that enables monomorphization and inlining of handler code.

use crate::handler::{BoxedHandler, IntoHandler};
use crate::{Error, HttpRequest, HttpResponse};
use std::any::TypeId;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Type alias for async route handler functions (legacy)
///
/// **Deprecated**: Prefer using `BoxedHandler` for better performance.
/// This type exists for backwards compatibility with existing macro-generated code.
pub type RouteHandlerFn = Arc<
    dyn Fn(HttpRequest) -> Pin<Box<dyn Future<Output = Result<HttpResponse, Error>> + Send>>
        + Send
        + Sync,
>;

/// Optimized handler type for route registry.
///
/// This uses the optimized BoxedHandler which enables monomorphization.
pub type OptimizedRouteHandler = BoxedHandler;

/// A route entry that can be collected via inventory.
///
/// This struct stores route metadata and an optimized handler that
/// supports monomorphization for better performance.
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
    /// The actual handler function (legacy format for compatibility)
    pub handler: RouteHandlerFn,
    /// Optimized handler (uses BoxedHandler for monomorphization)
    /// This is the preferred handler to use when available.
    pub optimized_handler: Option<BoxedHandler>,
}

// Register RouteEntry with inventory for compile-time collection
inventory::collect!(RouteEntry);

impl RouteEntry {
    /// Create a new route entry with a legacy handler.
    ///
    /// For backwards compatibility with existing macro-generated code.
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
            optimized_handler: None,
        }
    }

    /// Create a new route entry with an optimized handler.
    ///
    /// This is the preferred method for creating route entries as it
    /// enables handler inlining via monomorphization.
    #[inline]
    pub fn new_optimized<C: 'static, H, Args>(
        method: &'static str,
        path: &'static str,
        handler_name: &'static str,
        handler: H,
    ) -> Self
    where
        H: IntoHandler<Args> + Clone,
    {
        let boxed = BoxedHandler::new(handler.clone().into_handler());

        // Create a legacy handler wrapper for compatibility
        let legacy: RouteHandlerFn = Arc::new(move |req| {
            let h = handler.clone().into_handler();
            Box::pin(async move { crate::handler::Handler::call(&h, req).await })
        });

        Self {
            controller_type_id: TypeId::of::<C>(),
            controller_type_name: std::any::type_name::<C>(),
            method,
            path,
            handler_name,
            handler: legacy,
            optimized_handler: Some(boxed),
        }
    }

    /// Get the best available handler.
    ///
    /// Returns the optimized handler if available, otherwise wraps the legacy handler.
    #[inline]
    pub fn get_handler(&self) -> BoxedHandler {
        if let Some(ref h) = self.optimized_handler {
            h.clone()
        } else {
            crate::handler::from_legacy_handler(self.handler.clone())
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

/// Macro to register a route handler with the inventory (legacy)
///
/// This is used internally by the route macros (#[get], #[post], etc.)
/// for backwards compatibility.
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

/// Macro to register an optimized route handler with the inventory
///
/// This version uses the optimized handler system that enables
/// monomorphization and handler inlining for better performance.
///
/// # Example
///
/// ```ignore
/// register_route_optimized!(
///     MyController,
///     "GET",
///     "/users",
///     "get_users",
///     get_users
/// );
/// ```
#[macro_export]
macro_rules! register_route_optimized {
    ($controller:ty, $method:expr, $path:expr, $handler_name:expr, $handler:expr) => {
        inventory::submit! {
            $crate::route_registry::RouteEntry::new_optimized::<$controller, _, _>(
                $method,
                $path,
                $handler_name,
                $handler,
            )
        }
    };
}
