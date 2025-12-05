// Test assertions for HTTP responses

use crate::TestResponse;
use armature_core::{HttpResponse, HttpStatus};

/// Assert that a response has a specific status code
pub fn assert_status(response: &TestResponse, expected: u16) {
    let actual = response.status().unwrap_or(0);
    assert_eq!(
        actual, expected,
        "Expected status {}, got {}",
        expected, actual
    );
}

/// Assert that a response has a specific HTTP status
#[allow(dead_code)]
pub fn assert_http_status(response: &HttpResponse, expected: HttpStatus) {
    assert_eq!(
        response.status,
        expected.code(),
        "Expected status {}, got {}",
        expected.code(),
        response.status
    );
}

/// Assert that a response body contains JSON matching expected value
pub fn assert_json<T>(response: &TestResponse, expected: &T)
where
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
{
    let actual: T = response
        .body_json()
        .expect("Failed to deserialize response body");
    assert_eq!(actual, *expected, "JSON bodies do not match");
}

/// Assert that a response has a specific header
pub fn assert_header(response: &TestResponse, key: &str, expected: &str) {
    let actual = response.header(key).map(|s| s.as_str());
    assert_eq!(
        actual,
        Some(expected),
        "Expected header '{}' to be '{}', got {:?}",
        key,
        expected,
        actual
    );
}

/// Assert that a response body contains a string
#[allow(dead_code)]
pub fn assert_body_contains(response: &TestResponse, expected: &str) {
    let body = response.body_string().unwrap_or_default();
    assert!(
        body.contains(expected),
        "Expected body to contain '{}', but it didn't. Body: {}",
        expected,
        body
    );
}

/// Assert that a response is successful (2xx status)
#[allow(dead_code)]
pub fn assert_success(response: &TestResponse) {
    let status = response.status().unwrap_or(0);
    assert!(
        (200..300).contains(&status),
        "Expected successful status (2xx), got {}",
        status
    );
}

/// Assert that a response is a client error (4xx status)
#[allow(dead_code)]
pub fn assert_client_error(response: &TestResponse) {
    let status = response.status().unwrap_or(0);
    assert!(
        (400..500).contains(&status),
        "Expected client error status (4xx), got {}",
        status
    );
}

/// Assert that a response is a server error (5xx status)
#[allow(dead_code)]
pub fn assert_server_error(response: &TestResponse) {
    let status = response.status().unwrap_or(0);
    assert!(
        (500..600).contains(&status),
        "Expected server error status (5xx), got {}",
        status
    );
}

/// Assert that a response has JSON content type
#[allow(dead_code)]
pub fn assert_json_content_type(response: &TestResponse) {
    let content_type = response.header("Content-Type");
    assert!(
        content_type
            .map(|ct| ct.contains("application/json"))
            .unwrap_or(false),
        "Expected JSON content type, got {:?}",
        content_type
    );
}

/// Assert that a response has HTML content type
#[allow(dead_code)]
pub fn assert_html_content_type(response: &TestResponse) {
    let content_type = response.header("Content-Type");
    assert!(
        content_type
            .map(|ct| ct.contains("text/html"))
            .unwrap_or(false),
        "Expected HTML content type, got {:?}",
        content_type
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use armature_core::HttpResponse;
    use std::collections::HashMap;

    fn create_test_response(status: u16, body: &str) -> TestResponse {
        TestResponse::Success(HttpResponse {
            status,
            headers: HashMap::new(),
            body: body.as_bytes().to_vec(),
        })
    }

    #[test]
    fn test_assert_status() {
        let response = create_test_response(200, "OK");
        assert_status(&response, 200);
    }

    #[test]
    fn test_assert_success() {
        let response = create_test_response(200, "OK");
        assert_success(&response);
    }

    #[test]
    fn test_assert_body_contains() {
        let response = create_test_response(200, "Hello World");
        assert_body_contains(&response, "Hello");
    }
}
