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
