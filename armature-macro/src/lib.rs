// Procedural macros for the Armature HTTP framework
// These macros provide Angular-style decorator syntax for Rust

use proc_macro::TokenStream;

mod body_limit_attr;
mod catch_attr;
mod controller;
mod injectable;
mod module;
mod params;
mod routes;
mod timeout_attr;

/// Marks a struct as injectable, allowing it to be registered in the DI container
#[proc_macro_attribute]
pub fn injectable(attr: TokenStream, item: TokenStream) -> TokenStream {
    injectable::injectable_impl(attr, item)
}

/// Marks a struct as a controller with a base path
#[proc_macro_attribute]
pub fn controller(attr: TokenStream, item: TokenStream) -> TokenStream {
    controller::controller_impl(attr, item)
}

/// Defines a module with providers, controllers, and imports
#[proc_macro_attribute]
pub fn module(attr: TokenStream, item: TokenStream) -> TokenStream {
    module::module_impl(attr, item)
}

/// HTTP GET route decorator
#[proc_macro_attribute]
pub fn get(attr: TokenStream, item: TokenStream) -> TokenStream {
    routes::route_impl(attr, item, "GET")
}

/// HTTP POST route decorator
#[proc_macro_attribute]
pub fn post(attr: TokenStream, item: TokenStream) -> TokenStream {
    routes::route_impl(attr, item, "POST")
}

/// HTTP PUT route decorator
#[proc_macro_attribute]
pub fn put(attr: TokenStream, item: TokenStream) -> TokenStream {
    routes::route_impl(attr, item, "PUT")
}

/// HTTP DELETE route decorator
#[proc_macro_attribute]
pub fn delete(attr: TokenStream, item: TokenStream) -> TokenStream {
    routes::route_impl(attr, item, "DELETE")
}

/// HTTP PATCH route decorator
#[proc_macro_attribute]
pub fn patch(attr: TokenStream, item: TokenStream) -> TokenStream {
    routes::route_impl(attr, item, "PATCH")
}

/// Extracts and deserializes the request body
#[proc_macro_derive(Body)]
pub fn body_derive(input: TokenStream) -> TokenStream {
    params::body_derive_impl(input)
}

/// Extracts a path parameter
#[proc_macro_derive(Param)]
pub fn param_derive(input: TokenStream) -> TokenStream {
    params::param_derive_impl(input)
}

/// Extracts and deserializes query parameters
#[proc_macro_derive(Query)]
pub fn query_derive(input: TokenStream) -> TokenStream {
    params::query_derive_impl(input)
}

/// Request timeout decorator
///
/// Applies a timeout to the decorated route handler. If the handler doesn't
/// complete within the specified duration, a 408 Request Timeout error is returned.
///
/// # Usage
///
/// ```ignore
/// use armature::{get, timeout};
///
/// // Timeout in seconds (default unit)
/// #[timeout(5)]
/// #[get("/quick")]
/// async fn quick_handler(req: HttpRequest) -> Result<HttpResponse, Error> {
///     Ok(HttpResponse::ok())
/// }
///
/// // Timeout with explicit unit
/// #[timeout(seconds = 30)]
/// #[get("/slow")]
/// async fn slow_handler(req: HttpRequest) -> Result<HttpResponse, Error> {
///     Ok(HttpResponse::ok())
/// }
///
/// // Timeout in milliseconds
/// #[timeout(ms = 500)]
/// #[get("/fast")]
/// async fn fast_handler(req: HttpRequest) -> Result<HttpResponse, Error> {
///     Ok(HttpResponse::ok())
/// }
///
/// // Timeout in minutes
/// #[timeout(minutes = 2)]
/// #[get("/long-running")]
/// async fn long_handler(req: HttpRequest) -> Result<HttpResponse, Error> {
///     Ok(HttpResponse::ok())
/// }
/// ```
#[proc_macro_attribute]
pub fn timeout(attr: TokenStream, item: TokenStream) -> TokenStream {
    timeout_attr::timeout_impl(attr, item)
}

/// Request body size limit decorator
///
/// Applies a body size limit to the decorated route handler. If the request body
/// exceeds the specified size, a 413 Payload Too Large error is returned.
///
/// # Usage
///
/// ```ignore
/// use armature::{post, body_limit};
///
/// // Limit in bytes
/// #[body_limit(1024)]
/// #[post("/tiny")]
/// async fn tiny_handler(req: HttpRequest) -> Result<HttpResponse, Error> {
///     Ok(HttpResponse::ok())
/// }
///
/// // Limit with unit suffix (as string)
/// #[body_limit("10mb")]
/// #[post("/upload")]
/// async fn upload_handler(req: HttpRequest) -> Result<HttpResponse, Error> {
///     Ok(HttpResponse::ok())
/// }
///
/// // Limit with named parameter
/// #[body_limit(mb = 5)]
/// #[post("/medium")]
/// async fn medium_handler(req: HttpRequest) -> Result<HttpResponse, Error> {
///     Ok(HttpResponse::ok())
/// }
///
/// // Various formats supported:
/// #[body_limit(512kb)]      // 512 kilobytes (identifier style)
/// #[body_limit(kb = 512)]   // 512 kilobytes (named parameter)
/// #[body_limit("1.5mb")]    // 1.5 megabytes (string with float)
/// #[body_limit(1gb)]        // 1 gigabyte
/// #[body_limit(bytes = 2048)] // 2048 bytes
/// ```
#[proc_macro_attribute]
pub fn body_limit(attr: TokenStream, item: TokenStream) -> TokenStream {
    body_limit_attr::body_limit_impl(attr, item)
}

/// Exception filter decorator
///
/// Creates an exception filter from a function. The function should return
/// `Option<HttpResponse>` - return `Some(response)` to handle the error,
/// or `None` to pass it to the next filter.
///
/// # Usage
///
/// ```ignore
/// use armature::{catch, Error, HttpResponse};
/// use armature_core::exception_filter::ExceptionContext;
///
/// // Catch all errors
/// #[catch]
/// async fn handle_all(error: &Error, ctx: &ExceptionContext) -> Option<HttpResponse> {
///     Some(HttpResponse::internal_server_error())
/// }
///
/// // Catch specific error types
/// #[catch(NotFound, RouteNotFound)]
/// async fn handle_not_found(error: &Error, ctx: &ExceptionContext) -> Option<HttpResponse> {
///     Some(HttpResponse::not_found())
/// }
///
/// // With priority (higher = called earlier)
/// #[catch(Validation, priority = 100)]
/// async fn handle_validation(error: &Error, ctx: &ExceptionContext) -> Option<HttpResponse> {
///     Some(HttpResponse::unprocessable_entity())
/// }
///
/// // With custom name
/// #[catch(BadRequest, name = "CustomBadRequestFilter")]
/// async fn handle_bad_request(error: &Error, ctx: &ExceptionContext) -> Option<HttpResponse> {
///     Some(HttpResponse::bad_request())
/// }
/// ```
///
/// # Generated Code
///
/// The macro generates:
/// - A struct named `{FunctionName}ExceptionFilter`
/// - Implementation of `ExceptionFilter` trait
/// - A convenience function with the original name to create the filter
#[proc_macro_attribute]
pub fn catch(attr: TokenStream, item: TokenStream) -> TokenStream {
    catch_attr::catch_impl(attr, item)
}
