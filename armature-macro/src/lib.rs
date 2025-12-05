// Procedural macros for the Armature HTTP framework
// These macros provide Angular-style decorator syntax for Rust

use proc_macro::TokenStream;

mod controller;
mod injectable;
mod module;
mod params;
mod routes;

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
