//! Fuzz target for URL and path parsing.
//!
//! Tests URL parsing, path normalization, and segment extraction
//! with arbitrary input.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

/// Arbitrary URL components for fuzzing.
#[derive(Debug, Arbitrary)]
struct FuzzUrl {
    /// Raw path string
    path: String,
    /// Query string
    query: Option<String>,
    /// Fragment
    fragment: Option<String>,
}

fuzz_target!(|data: FuzzUrl| {
    // Test 1: Path normalization
    let path = &data.path;

    // Split path into segments - should handle arbitrary input
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    let _ = segments.len();

    // Test path traversal detection
    let has_traversal = segments.iter().any(|s| *s == ".." || *s == ".");
    let _ = has_traversal;

    // Test 2: URL decoding
    // Simulate percent-decoding
    let mut decoded = String::new();
    let mut chars = path.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                decoded.push(byte as char);
            } else {
                decoded.push('%');
                decoded.push_str(&hex);
            }
        } else {
            decoded.push(c);
        }
    }
    let _ = decoded;

    // Test 3: Query string parsing
    if let Some(query) = &data.query {
        // Parse query parameters
        let mut params: Vec<(&str, &str)> = Vec::new();
        for pair in query.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                params.push((key, value));
            } else if !pair.is_empty() {
                params.push((pair, ""));
            }
        }
        let _ = params.len();
    }

    // Test 4: Full URL reconstruction
    let mut full_url = data.path.clone();
    if let Some(query) = &data.query {
        full_url.push('?');
        full_url.push_str(query);
    }
    if let Some(fragment) = &data.fragment {
        full_url.push('#');
        full_url.push_str(fragment);
    }
    let _ = full_url.len();

    // Test 5: Path prefix matching
    let test_prefixes = ["/api", "/v1", "/users", "/"];
    for prefix in &test_prefixes {
        let _ = data.path.starts_with(prefix);
    }

    // Test 6: Path suffix matching
    let test_suffixes = [".json", ".xml", ".html", "/"];
    for suffix in &test_suffixes {
        let _ = data.path.ends_with(suffix);
    }
});

