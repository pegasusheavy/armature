//! Fuzz target for HTTP header parsing.
//!
//! Tests header parsing, validation, and common header value parsing
//! with arbitrary input.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

use std::collections::HashMap;

/// Arbitrary headers for fuzzing.
#[derive(Debug, Arbitrary)]
struct FuzzHeaders {
    /// Raw header lines
    raw_lines: Vec<String>,
    /// Key-value pairs
    pairs: Vec<(String, String)>,
}

fuzz_target!(|data: FuzzHeaders| {
    // Test 1: Parse raw header lines (name: value format)
    let mut headers: HashMap<String, String> = HashMap::new();
    for line in &data.raw_lines {
        if let Some((name, value)) = line.split_once(':') {
            let name = name.trim().to_lowercase();
            let value = value.trim().to_string();
            if !name.is_empty() {
                headers.insert(name, value);
            }
        }
    }
    
    // Test 2: Build headers from pairs
    let mut headers2: HashMap<String, String> = HashMap::new();
    for (key, value) in &data.pairs {
        let key_lower = key.to_lowercase();
        if !key_lower.is_empty() && key_lower.len() < 1000 {
            headers2.insert(key_lower, value.clone());
        }
    }
    
    // Test 3: Parse common header values
    
    // Content-Type parsing
    if let Some(ct) = headers.get("content-type") {
        // Extract mime type and parameters
        let parts: Vec<&str> = ct.split(';').collect();
        if let Some(mime) = parts.first() {
            let _ = mime.trim();
        }
        // Look for charset
        for part in parts.iter().skip(1) {
            if let Some((key, value)) = part.split_once('=') {
                if key.trim().to_lowercase() == "charset" {
                    let _ = value.trim();
                }
            }
        }
    }
    
    // Accept header parsing
    if let Some(accept) = headers.get("accept") {
        // Parse media types with quality values
        for media_type in accept.split(',') {
            let parts: Vec<&str> = media_type.split(';').collect();
            if let Some(mime) = parts.first() {
                let _ = mime.trim();
            }
            // Look for q= quality value
            for part in parts.iter().skip(1) {
                if let Some((key, value)) = part.split_once('=') {
                    if key.trim().to_lowercase() == "q" {
                        let _ = value.trim().parse::<f32>();
                    }
                }
            }
        }
    }
    
    // Content-Length parsing
    if let Some(cl) = headers.get("content-length") {
        let _ = cl.trim().parse::<u64>();
    }
    
    // Authorization parsing
    if let Some(auth) = headers.get("authorization") {
        let parts: Vec<&str> = auth.splitn(2, ' ').collect();
        if parts.len() == 2 {
            let scheme = parts[0];
            let credentials = parts[1];
            let _ = scheme.to_lowercase();
            let _ = credentials.len();
        }
    }
    
    // Cookie parsing
    if let Some(cookie) = headers.get("cookie") {
        for pair in cookie.split(';') {
            if let Some((name, value)) = pair.split_once('=') {
                let _ = name.trim();
                let _ = value.trim();
            }
        }
    }
    
    // Cache-Control parsing
    if let Some(cc) = headers.get("cache-control") {
        for directive in cc.split(',') {
            let directive = directive.trim();
            if let Some((key, value)) = directive.split_once('=') {
                let _ = key.trim();
                let _ = value.trim().parse::<u64>();
            } else {
                let _ = directive;
            }
        }
    }
    
    // Test 4: Header validation (check for invalid characters)
    for (key, value) in &headers {
        // Check for control characters
        let has_invalid_key = key.chars().any(|c| c.is_control() || c == ':');
        let has_invalid_value = value.chars().any(|c| c == '\n' || c == '\r');
        let _ = has_invalid_key;
        let _ = has_invalid_value;
    }
});

