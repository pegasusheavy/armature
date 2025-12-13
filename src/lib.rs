// Armature - An Angular-inspired HTTP framework for Rust
//
// This library provides a decorator-based approach to building HTTP applications
// with dependency injection, routing, and validation.

// Re-export core functionality
pub use armature_core::*;

// Re-export procedural macros
pub use armature_macro::{
    Body, Param, Query, controller, delete, get, injectable, module, patch, post, put,
};

// Re-export optional crates
#[cfg(feature = "graphql")]
pub use armature_graphql;

#[cfg(feature = "config")]
pub use armature_config;

#[cfg(feature = "jwt")]
pub use armature_jwt;

#[cfg(feature = "auth")]
pub use armature_auth;

#[cfg(feature = "testing")]
pub use armature_testing;

#[cfg(feature = "validation")]
pub use armature_validation;

#[cfg(feature = "openapi")]
pub use armature_openapi;

#[cfg(feature = "cache")]
pub use armature_cache;

#[cfg(feature = "cron")]
pub use armature_cron;

#[cfg(feature = "queue")]
pub use armature_queue;

#[cfg(feature = "opentelemetry")]
pub use armature_opentelemetry;

#[cfg(feature = "security")]
pub use armature_security;

#[cfg(feature = "acme")]
pub use armature_acme;

#[cfg(feature = "ratelimit")]
pub use armature_ratelimit;

#[cfg(feature = "compression")]
pub use armature_compression;

#[cfg(feature = "webhooks")]
pub use armature_webhooks;

// Prelude for common imports
pub mod prelude {
    pub use crate::{
        Application,
        Body,
        Container,
        Controller,
        Error,
        HttpMethod,
        HttpRequest,
        HttpResponse,
        Json,
        Module,
        Param,
        Provider,
        Query,
        RequestHandler,
        Route,
        Router,
        // SSE types
        ServerSentEvent,
        SseBroadcaster,
        SseStream,
        WebSocketConnection,
        WebSocketManager,
        // WebSocket types
        WebSocketMessage,
        WebSocketRoom,
        controller,
        delete,
        get,
        injectable,
        module,
        patch,
        post,
        put,
    };
}
