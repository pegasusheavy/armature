//! GraphQL request batching.

use serde::Serialize;
use serde_json::Value;

use crate::GraphQLResponse;
use crate::request::GraphQLRequest;

/// A batch of GraphQL requests.
#[derive(Debug, Clone, Serialize)]
pub struct BatchRequest {
    requests: Vec<GraphQLRequest>,
}

impl BatchRequest {
    /// Create a new batch request.
    pub fn new() -> Self {
        Self {
            requests: Vec::new(),
        }
    }

    /// Add a request to the batch.
    pub fn with_request(mut self, request: GraphQLRequest) -> Self {
        self.requests.push(request);
        self
    }

    /// Add a query to the batch.
    pub fn query(self, query: impl Into<String>) -> Self {
        self.with_request(GraphQLRequest::new(query))
    }

    /// Get the number of requests in the batch.
    pub fn len(&self) -> usize {
        self.requests.len()
    }

    /// Check if the batch is empty.
    pub fn is_empty(&self) -> bool {
        self.requests.is_empty()
    }

    /// Get the requests.
    pub fn requests(&self) -> &[GraphQLRequest] {
        &self.requests
    }

    /// Consume and return the requests.
    pub fn into_requests(self) -> Vec<GraphQLRequest> {
        self.requests
    }
}

impl Default for BatchRequest {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoIterator for BatchRequest {
    type Item = GraphQLRequest;
    type IntoIter = std::vec::IntoIter<GraphQLRequest>;

    fn into_iter(self) -> Self::IntoIter {
        self.requests.into_iter()
    }
}

/// A batch of GraphQL responses.
#[derive(Debug, Clone)]
pub struct BatchResponse {
    responses: Vec<GraphQLResponse<Value>>,
}

impl BatchResponse {
    /// Create a new batch response.
    pub fn new(responses: Vec<GraphQLResponse<Value>>) -> Self {
        Self { responses }
    }

    /// Get the number of responses.
    pub fn len(&self) -> usize {
        self.responses.len()
    }

    /// Check if the batch is empty.
    pub fn is_empty(&self) -> bool {
        self.responses.is_empty()
    }

    /// Get a specific response by index.
    pub fn get(&self, index: usize) -> Option<&GraphQLResponse<Value>> {
        self.responses.get(index)
    }

    /// Get the responses.
    pub fn responses(&self) -> &[GraphQLResponse<Value>] {
        &self.responses
    }

    /// Consume and return the responses.
    pub fn into_responses(self) -> Vec<GraphQLResponse<Value>> {
        self.responses
    }

    /// Check if any response has errors.
    pub fn has_errors(&self) -> bool {
        self.responses.iter().any(|r| r.has_errors())
    }

    /// Get all errors from all responses.
    pub fn all_errors(&self) -> Vec<&crate::GraphQLResponseError> {
        self.responses
            .iter()
            .filter_map(|r| r.errors.as_ref())
            .flatten()
            .collect()
    }
}

impl IntoIterator for BatchResponse {
    type Item = GraphQLResponse<Value>;
    type IntoIter = std::vec::IntoIter<GraphQLResponse<Value>>;

    fn into_iter(self) -> Self::IntoIter {
        self.responses.into_iter()
    }
}

impl<'a> IntoIterator for &'a BatchResponse {
    type Item = &'a GraphQLResponse<Value>;
    type IntoIter = std::slice::Iter<'a, GraphQLResponse<Value>>;

    fn into_iter(self) -> Self::IntoIter {
        self.responses.iter()
    }
}

