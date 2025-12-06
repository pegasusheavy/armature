// HTTP request and response types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// HTTP request wrapper
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub path_params: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
}

impl HttpRequest {
    pub fn new(method: String, path: String) -> Self {
        Self {
            method,
            path,
            headers: HashMap::new(),
            body: Vec::new(),
            path_params: HashMap::new(),
            query_params: HashMap::new(),
        }
    }

    /// Parse the request body as JSON
    pub fn json<T: for<'de> Deserialize<'de>>(&self) -> Result<T, crate::Error> {
        serde_json::from_slice(&self.body).map_err(|e| crate::Error::Deserialization(e.to_string()))
    }

    /// Parse URL-encoded form data
    pub fn form<T: for<'de> Deserialize<'de>>(&self) -> Result<T, crate::Error> {
        crate::form::parse_form(&self.body)
    }

    /// Parse URL-encoded form data into a HashMap
    pub fn form_map(&self) -> Result<HashMap<String, String>, crate::Error> {
        crate::form::parse_form_map(&self.body)
    }

    /// Parse multipart form data
    pub fn multipart(&self) -> Result<Vec<crate::form::FormField>, crate::Error> {
        let content_type = self.headers
            .get("Content-Type")
            .or_else(|| self.headers.get("content-type"))
            .ok_or_else(|| crate::Error::BadRequest("Missing Content-Type header".to_string()))?;
        
        let parser = crate::form::MultipartParser::from_content_type(content_type)?;
        parser.parse(&self.body)
    }

    /// Get a path parameter by name
    pub fn param(&self, name: &str) -> Option<&String> {
        self.path_params.get(name)
    }

    /// Get a query parameter by name
    pub fn query(&self, name: &str) -> Option<&String> {
        self.query_params.get(name)
    }
}

/// HTTP response wrapper
#[derive(Debug)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    pub fn new(status: u16) -> Self {
        Self {
            status,
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    pub fn ok() -> Self {
        Self::new(200)
    }

    pub fn created() -> Self {
        Self::new(201)
    }

    pub fn no_content() -> Self {
        Self::new(204)
    }

    pub fn bad_request() -> Self {
        Self::new(400)
    }

    pub fn not_found() -> Self {
        Self::new(404)
    }

    pub fn internal_server_error() -> Self {
        Self::new(500)
    }

    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }

    pub fn with_json<T: Serialize>(mut self, value: &T) -> Result<Self, crate::Error> {
        self.body =
            serde_json::to_vec(value).map_err(|e| crate::Error::Serialization(e.to_string()))?;
        self.headers
            .insert("Content-Type".to_string(), "application/json".to_string());
        Ok(self)
    }

    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }
}

/// JSON response helper
#[derive(Debug)]
pub struct Json<T: Serialize>(pub T);

impl<T: Serialize> Json<T> {
    pub fn into_response(self) -> Result<HttpResponse, crate::Error> {
        HttpResponse::ok().with_json(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_request_new() {
        let req = HttpRequest::new("GET".to_string(), "/test".to_string());
        assert_eq!(req.method, "GET");
        assert_eq!(req.path, "/test");
        assert!(req.headers.is_empty());
        assert!(req.body.is_empty());
    }

    #[test]
    fn test_http_request_with_body() {
        let mut req = HttpRequest::new("POST".to_string(), "/api".to_string());
        req.body = vec![1, 2, 3, 4];
        assert_eq!(req.body.len(), 4);
    }

    #[test]
    fn test_http_request_json_deserialization() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct TestData {
            name: String,
            age: u32,
        }

        let mut req = HttpRequest::new("POST".to_string(), "/api".to_string());
        req.body = serde_json::to_vec(&serde_json::json!({
            "name": "John",
            "age": 30
        }))
        .unwrap();

        let data: TestData = req.json().unwrap();
        assert_eq!(data.name, "John");
        assert_eq!(data.age, 30);
    }

    #[test]
    fn test_http_request_param() {
        let mut req = HttpRequest::new("GET".to_string(), "/users/123".to_string());
        req.path_params
            .insert("id".to_string(), "123".to_string());

        assert_eq!(req.param("id"), Some(&"123".to_string()));
        assert_eq!(req.param("name"), None);
    }

    #[test]
    fn test_http_request_query() {
        let mut req = HttpRequest::new("GET".to_string(), "/users".to_string());
        req.query_params
            .insert("sort".to_string(), "asc".to_string());

        assert_eq!(req.query("sort"), Some(&"asc".to_string()));
        assert_eq!(req.query("limit"), None);
    }

    #[test]
    fn test_http_request_clone() {
        let req1 = HttpRequest::new("GET".to_string(), "/test".to_string());
        let req2 = req1.clone();

        assert_eq!(req1.method, req2.method);
        assert_eq!(req1.path, req2.path);
    }

    #[test]
    fn test_http_response_ok() {
        let res = HttpResponse::ok();
        assert_eq!(res.status, 200);
    }

    #[test]
    fn test_http_response_created() {
        let res = HttpResponse::created();
        assert_eq!(res.status, 201);
    }

    #[test]
    fn test_http_response_no_content() {
        let res = HttpResponse::no_content();
        assert_eq!(res.status, 204);
    }

    #[test]
    fn test_http_response_bad_request() {
        let res = HttpResponse::bad_request();
        assert_eq!(res.status, 400);
    }

    #[test]
    fn test_http_response_not_found() {
        let res = HttpResponse::not_found();
        assert_eq!(res.status, 404);
    }

    #[test]
    fn test_http_response_internal_server_error() {
        let res = HttpResponse::internal_server_error();
        assert_eq!(res.status, 500);
    }

    #[test]
    fn test_http_response_with_body() {
        let body = b"Hello, World!".to_vec();
        let res = HttpResponse::ok().with_body(body.clone());
        assert_eq!(res.body, body);
    }

    #[test]
    fn test_http_response_with_json() {
        #[derive(Serialize)]
        struct TestData {
            message: String,
        }

        let data = TestData {
            message: "test".to_string(),
        };

        let res = HttpResponse::ok().with_json(&data).unwrap();
        assert!(!res.body.is_empty());
        assert_eq!(
            res.headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
    }

    #[test]
    fn test_http_response_with_header() {
        let res = HttpResponse::ok()
            .with_header("X-Custom".to_string(), "value".to_string());

        assert_eq!(res.headers.get("X-Custom"), Some(&"value".to_string()));
    }

    #[test]
    fn test_http_response_multiple_headers() {
        let res = HttpResponse::ok()
            .with_header("X-Header-1".to_string(), "value1".to_string())
            .with_header("X-Header-2".to_string(), "value2".to_string());

        assert_eq!(res.headers.len(), 2);
    }

    #[test]
    fn test_json_helper() {
        #[derive(Serialize)]
        struct Data {
            value: i32,
        }

        let json = Json(Data { value: 42 });
        let response = json.into_response().unwrap();

        assert_eq!(response.status, 200);
        assert!(!response.body.is_empty());
    }

    #[test]
    fn test_http_request_with_headers() {
        let mut req = HttpRequest::new("GET".to_string(), "/api".to_string());
        req.headers
            .insert("Authorization".to_string(), "Bearer token".to_string());
        req.headers
            .insert("Content-Type".to_string(), "application/json".to_string());

        assert_eq!(req.headers.len(), 2);
    }

    #[test]
    fn test_http_request_json_invalid() {
        #[derive(Deserialize)]
        struct TestData {
            name: String,
        }

        let mut req = HttpRequest::new("POST".to_string(), "/api".to_string());
        req.body = b"invalid json".to_vec();

        let result: Result<TestData, crate::Error> = req.json();
        assert!(result.is_err());
    }

    #[test]
    fn test_http_response_new_custom_status() {
        let res = HttpResponse::new(418); // I'm a teapot
        assert_eq!(res.status, 418);
    }

    #[test]
    fn test_http_response_with_json_complex() {
        #[derive(Serialize)]
        struct ComplexData {
            nested: Vec<HashMap<String, i32>>,
        }

        let mut map = HashMap::new();
        map.insert("key".to_string(), 123);

        let data = ComplexData {
            nested: vec![map],
        };

        let res = HttpResponse::ok().with_json(&data);
        assert!(res.is_ok());
    }
}
