//! Fuzz target for query parameter parsing.
//!
//! Tests query string parsing with various encodings and formats.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

use std::collections::HashMap;

/// Arbitrary query string for fuzzing.
#[derive(Debug, Arbitrary)]
struct FuzzQuery {
    /// Raw query string
    raw: String,
    /// Individual parameters
    params: Vec<(String, String)>,
}

/// URL decode a string.
fn url_decode(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    
    while let Some(c) = chars.next() {
        match c {
            '%' => {
                let hex: String = chars.by_ref().take(2).collect();
                if hex.len() == 2 {
                    if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                        result.push(byte as char);
                    } else {
                        result.push('%');
                        result.push_str(&hex);
                    }
                } else {
                    result.push('%');
                    result.push_str(&hex);
                }
            }
            '+' => result.push(' '),
            _ => result.push(c),
        }
    }
    
    result
}

/// Parse query string into key-value pairs.
fn parse_query(query: &str) -> Vec<(String, String)> {
    let mut params = Vec::new();
    
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        
        if let Some((key, value)) = pair.split_once('=') {
            let key = url_decode(key);
            let value = url_decode(value);
            params.push((key, value));
        } else {
            // Key without value
            let key = url_decode(pair);
            params.push((key, String::new()));
        }
    }
    
    params
}

fuzz_target!(|data: FuzzQuery| {
    // Test 1: Parse raw query string
    let params = parse_query(&data.raw);
    let _ = params.len();
    
    // Test 2: Build HashMap from params
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for (key, value) in &params {
        map.entry(key.clone()).or_default().push(value.clone());
    }
    
    // Test 3: Look for common parameter names
    let common_params = ["page", "limit", "offset", "sort", "order", "filter", "q", "search"];
    for param in &common_params {
        let _ = map.get(*param);
    }
    
    // Test 4: Parse numeric parameters
    for (key, values) in &map {
        for value in values {
            // Try parsing as various numeric types
            let _ = value.parse::<i32>();
            let _ = value.parse::<u64>();
            let _ = value.parse::<f64>();
            let _ = value.parse::<bool>();
        }
        let _ = key.len();
    }
    
    // Test 5: Build query string from structured params
    let mut built_query = String::new();
    for (i, (key, value)) in data.params.iter().enumerate() {
        if i > 0 {
            built_query.push('&');
        }
        // Simple URL encoding (just spaces and &)
        let encoded_key = key.replace(' ', "+").replace('&', "%26");
        let encoded_value = value.replace(' ', "+").replace('&', "%26");
        built_query.push_str(&encoded_key);
        built_query.push('=');
        built_query.push_str(&encoded_value);
    }
    
    // Test 6: Re-parse built query
    let reparsed = parse_query(&built_query);
    let _ = reparsed.len();
    
    // Test 7: Handle edge cases
    // Empty query
    let _ = parse_query("");
    
    // Just &
    let _ = parse_query("&");
    
    // Multiple &
    let _ = parse_query("&&&&");
    
    // Key with empty value
    let _ = parse_query("key=");
    
    // Empty key with value
    let _ = parse_query("=value");
    
    // Multiple = signs
    let _ = parse_query("key=value=extra");
});

