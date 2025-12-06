// Core library for the Armature HTTP framework
// This module contains the foundational types, traits, and runtime components

pub mod application;
pub mod container;
pub mod error;
pub mod form;
pub mod guard;
pub mod http;
pub mod interceptor;
pub mod middleware;
pub mod routing;
pub mod sse;
pub mod static_assets;
pub mod status;
pub mod tls;
pub mod traits;
pub mod websocket;

// Re-export commonly used types
pub use application::*;
pub use container::*;
pub use error::*;
pub use form::*;
pub use guard::*;
pub use http::*;
pub use interceptor::*;
pub use middleware::*;
pub use routing::{Route, Router}; // Explicit exports to avoid ambiguous HandlerFn
pub use sse::*;
pub use static_assets::*;
pub use status::*;
pub use tls::*;
pub use traits::*;
pub use websocket::*;
