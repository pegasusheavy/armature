//! Fuzz target for route matching.
//!
//! Tests the router's ability to handle arbitrary route patterns
//! and paths without panicking.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

use armature_core::http::HttpMethod;
use armature_core::routing::Router;

/// Arbitrary routing scenario for fuzzing.
#[derive(Debug, Arbitrary)]
struct FuzzRouting {
    /// Routes to register
    routes: Vec<FuzzRoute>,
    /// Paths to match against
    match_paths: Vec<(FuzzMethod, String)>,
}

#[derive(Debug, Arbitrary)]
struct FuzzRoute {
    method: FuzzMethod,
    pattern: String,
}

#[derive(Debug, Arbitrary, Clone)]
enum FuzzMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

impl FuzzMethod {
    fn to_http_method(&self) -> HttpMethod {
        match self {
            FuzzMethod::Get => HttpMethod::Get,
            FuzzMethod::Post => HttpMethod::Post,
            FuzzMethod::Put => HttpMethod::Put,
            FuzzMethod::Delete => HttpMethod::Delete,
            FuzzMethod::Patch => HttpMethod::Patch,
        }
    }
}

// Dummy handler for route registration
async fn dummy_handler(
    _req: armature_core::http::HttpRequest,
) -> Result<armature_core::http::HttpResponse, armature_core::error::Error> {
    Ok(armature_core::http::HttpResponse::ok())
}

fuzz_target!(|data: FuzzRouting| {
    // Limit route count to prevent OOM
    let max_routes = 100;
    let routes: Vec<_> = data.routes.into_iter().take(max_routes).collect();
    
    // Create router
    let mut router = Router::new();
    
    // Register routes - should handle arbitrary patterns
    for route in &routes {
        // Normalize pattern to start with /
        let pattern = if route.pattern.starts_with('/') {
            route.pattern.clone()
        } else {
            format!("/{}", route.pattern)
        };
        
        // Skip extremely long patterns
        if pattern.len() > 1000 {
            continue;
        }
        
        // Try to add route - may fail for invalid patterns but should not panic
        let _ = router.add_route(
            route.method.to_http_method(),
            &pattern,
            dummy_handler,
        );
    }
    
    // Match paths - should handle arbitrary input
    for (method, path) in &data.match_paths {
        // Normalize path
        let path = if path.starts_with('/') {
            path.clone()
        } else {
            format!("/{}", path)
        };
        
        // Skip extremely long paths
        if path.len() > 10000 {
            continue;
        }
        
        // Match should not panic
        let _ = router.match_route(method.to_http_method(), &path);
    }
});

