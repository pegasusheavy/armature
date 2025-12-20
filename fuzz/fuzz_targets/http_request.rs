//! Fuzz target for HTTP request parsing.
//!
//! This target tests the robustness of HTTP request parsing against
//! arbitrary input, looking for panics, hangs, or memory issues.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

use armature_core::http::{HttpMethod, HttpRequest};
use bytes::Bytes;
use std::collections::HashMap;

/// Arbitrary HTTP request for fuzzing.
#[derive(Debug, Arbitrary)]
struct FuzzRequest {
    method: FuzzMethod,
    path: String,
    query: Option<String>,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

#[derive(Debug, Arbitrary)]
enum FuzzMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
    Connect,
    Trace,
    Custom(String),
}

impl FuzzMethod {
    fn to_http_method(&self) -> HttpMethod {
        match self {
            FuzzMethod::Get => HttpMethod::Get,
            FuzzMethod::Post => HttpMethod::Post,
            FuzzMethod::Put => HttpMethod::Put,
            FuzzMethod::Delete => HttpMethod::Delete,
            FuzzMethod::Patch => HttpMethod::Patch,
            FuzzMethod::Head => HttpMethod::Head,
            FuzzMethod::Options => HttpMethod::Options,
            FuzzMethod::Connect => HttpMethod::Connect,
            FuzzMethod::Trace => HttpMethod::Trace,
            FuzzMethod::Custom(_) => HttpMethod::Get, // Default for custom
        }
    }
}

fuzz_target!(|data: FuzzRequest| {
    // Build path with optional query string
    let path = if let Some(query) = &data.query {
        format!("{}?{}", data.path, query)
    } else {
        data.path.clone()
    };

    // Create headers map
    let mut headers = HashMap::new();
    for (key, value) in &data.headers {
        headers.insert(key.clone(), value.clone());
    }

    // Create the request - should not panic
    let request = HttpRequest {
        method: data.method.to_http_method(),
        path: path.clone(),
        query: data.query.clone(),
        headers,
        body: Bytes::from(data.body.clone()),
        params: HashMap::new(),
        extensions: armature_core::http::Extensions::new(),
    };

    // Test various accessors - should not panic
    let _ = request.method.as_str();
    let _ = request.path.len();
    let _ = request.body.len();
    
    // Test header access
    for (key, _) in &request.headers {
        let _ = request.headers.get(key);
    }
    
    // Test query parsing if present
    if let Some(query) = &request.query {
        // Parse query string manually - should handle malformed input
        for pair in query.split('&') {
            let _ = pair.split_once('=');
        }
    }
});

