//! Azure Functions request conversion.

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Azure Functions HTTP request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionRequest {
    /// HTTP method.
    pub method: String,
    /// Request URL.
    pub url: String,
    /// Request path.
    pub path: String,
    /// Query string parameters.
    pub query: HashMap<String, String>,
    /// Request headers.
    pub headers: HashMap<String, String>,
    /// Request body (base64-encoded for serialization).
    #[serde(default, with = "body_serde")]
    pub body: Bytes,
    /// Route parameters.
    #[serde(default)]
    pub params: HashMap<String, String>,
    /// Request context.
    #[serde(default)]
    pub context: RequestContext,
}

/// Custom serde module for Bytes <-> base64 string conversion.
mod body_serde {
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    use bytes::Bytes;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &Bytes, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let encoded = STANDARD.encode(bytes);
        serializer.serialize_str(&encoded)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Bytes, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.is_empty() {
            return Ok(Bytes::new());
        }
        STANDARD
            .decode(&s)
            .map(Bytes::from)
            .map_err(serde::de::Error::custom)
    }
}

/// Request context from Azure Functions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RequestContext {
    /// Invocation ID.
    pub invocation_id: Option<String>,
    /// Function name.
    pub function_name: Option<String>,
    /// Function directory.
    pub function_directory: Option<String>,
    /// Trace context for distributed tracing.
    pub trace_context: Option<TraceContext>,
    /// Retry context (if using retry policies).
    pub retry_context: Option<RetryContext>,
}

/// Distributed tracing context.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TraceContext {
    /// Trace parent header.
    pub trace_parent: Option<String>,
    /// Trace state header.
    pub trace_state: Option<String>,
}

/// Retry context for durable functions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RetryContext {
    /// Current retry count.
    pub retry_count: u32,
    /// Maximum retries.
    pub max_retry_count: u32,
}

impl FunctionRequest {
    /// Create a new function request.
    pub fn new(method: impl Into<String>, path: impl Into<String>) -> Self {
        let path = path.into();
        Self {
            method: method.into(),
            url: path.clone(),
            path,
            query: HashMap::new(),
            headers: HashMap::new(),
            body: Bytes::new(),
            params: HashMap::new(),
            context: RequestContext::default(),
        }
    }

    /// Parse from Azure Functions JSON input.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Get a header value (case-insensitive).
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .get(&name.to_lowercase())
            .or_else(|| self.headers.get(name))
            .map(|s| s.as_str())
    }

    /// Get a query parameter.
    pub fn query_param(&self, name: &str) -> Option<&str> {
        self.query.get(name).map(|s| s.as_str())
    }

    /// Get a route parameter.
    pub fn route_param(&self, name: &str) -> Option<&str> {
        self.params.get(name).map(|s| s.as_str())
    }

    /// Get the content type.
    pub fn content_type(&self) -> Option<&str> {
        self.header("content-type")
    }

    /// Check if the request is JSON.
    pub fn is_json(&self) -> bool {
        self.content_type()
            .map(|ct| ct.contains("application/json"))
            .unwrap_or(false)
    }

    /// Get the HTTP method.
    pub fn http_method(&self) -> http::Method {
        self.method.parse().unwrap_or(http::Method::GET)
    }

    /// Get the client IP address.
    pub fn client_ip(&self) -> Option<&str> {
        self.header("x-forwarded-for")
            .or_else(|| self.header("x-client-ip"))
    }

    /// Get the authorization header.
    pub fn authorization(&self) -> Option<&str> {
        self.header("authorization")
    }

    /// Get a bearer token from the authorization header.
    pub fn bearer_token(&self) -> Option<&str> {
        self.authorization()
            .and_then(|auth| auth.strip_prefix("Bearer "))
    }
}

impl Default for FunctionRequest {
    fn default() -> Self {
        Self::new("GET", "/")
    }
}
