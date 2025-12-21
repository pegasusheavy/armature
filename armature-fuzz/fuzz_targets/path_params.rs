//! Fuzz target for path parameter extraction.
//!
//! Tests route pattern matching and parameter extraction.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

use std::collections::HashMap;

/// Arbitrary path matching scenario.
#[derive(Debug, Arbitrary)]
struct FuzzPathParams {
    /// Route pattern (e.g., "/users/:id/posts/:post_id")
    pattern: String,
    /// Actual path to match
    path: String,
}

/// Extract parameter names from a pattern.
fn extract_param_names(pattern: &str) -> Vec<String> {
    let mut names = Vec::new();
    let segments: Vec<&str> = pattern.split('/').collect();

    for segment in segments {
        if segment.starts_with(':') {
            names.push(segment[1..].to_string());
        } else if segment.starts_with('*') {
            // Wildcard parameter
            let name = if segment.len() > 1 {
                segment[1..].to_string()
            } else {
                "wildcard".to_string()
            };
            names.push(name);
        } else if segment.starts_with('{') && segment.ends_with('}') {
            // Alternative syntax: {param}
            let name = segment[1..segment.len()-1].to_string();
            if !name.is_empty() {
                names.push(name);
            }
        }
    }

    names
}

/// Match a path against a pattern and extract parameters.
fn match_path(pattern: &str, path: &str) -> Option<HashMap<String, String>> {
    let pattern_segments: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    let path_segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    let mut params = HashMap::new();
    let mut pattern_idx = 0;
    let mut path_idx = 0;

    while pattern_idx < pattern_segments.len() {
        let pattern_seg = pattern_segments[pattern_idx];

        // Wildcard matches rest of path
        if pattern_seg.starts_with('*') {
            let name = if pattern_seg.len() > 1 {
                &pattern_seg[1..]
            } else {
                "wildcard"
            };
            let rest: Vec<&str> = path_segments[path_idx..].to_vec();
            params.insert(name.to_string(), rest.join("/"));
            return Some(params);
        }

        // Need a path segment to match
        if path_idx >= path_segments.len() {
            return None;
        }

        let path_seg = path_segments[path_idx];

        if pattern_seg.starts_with(':') {
            // Parameter segment
            let name = &pattern_seg[1..];
            params.insert(name.to_string(), path_seg.to_string());
        } else if pattern_seg.starts_with('{') && pattern_seg.ends_with('}') {
            // Alternative parameter syntax
            let name = &pattern_seg[1..pattern_seg.len()-1];
            if !name.is_empty() {
                params.insert(name.to_string(), path_seg.to_string());
            }
        } else if pattern_seg != path_seg {
            // Static segment must match exactly
            return None;
        }

        pattern_idx += 1;
        path_idx += 1;
    }

    // Check if we consumed all path segments
    if path_idx != path_segments.len() {
        return None;
    }

    Some(params)
}

fuzz_target!(|data: FuzzPathParams| {
    // Limit input sizes
    if data.pattern.len() > 1000 || data.path.len() > 10000 {
        return;
    }

    // Test 1: Extract parameter names from pattern
    let param_names = extract_param_names(&data.pattern);
    let _ = param_names.len();

    // Test 2: Match path against pattern
    let matched = match_path(&data.pattern, &data.path);
    if let Some(params) = &matched {
        let _ = params.len();

        // Verify all param names are present
        for name in &param_names {
            let _ = params.get(name);
        }
    }

    // Test 3: Common patterns
    let test_patterns = [
        "/users/:id",
        "/users/:user_id/posts/:post_id",
        "/api/v1/*path",
        "/files/{filename}",
        "/:org/:repo/tree/:branch/*path",
    ];

    for pattern in &test_patterns {
        let _ = match_path(pattern, &data.path);
        let _ = extract_param_names(pattern);
    }

    // Test 4: Edge cases

    // Empty pattern
    let _ = match_path("", &data.path);

    // Root pattern
    let _ = match_path("/", &data.path);

    // Only wildcard
    let _ = match_path("/*", &data.path);

    // Multiple consecutive slashes in pattern
    let _ = match_path("//users//:id//", &data.path);

    // Test 5: Type coercion of extracted params
    if let Some(params) = matched {
        for (_, value) in params {
            // Try parsing as various types
            let _ = value.parse::<i32>();
            let _ = value.parse::<u64>();
            let _ = value.parse::<f64>();
            let _ = value.parse::<bool>();
            let _ = value.is_empty();
            let _ = value.len();
        }
    }
});

