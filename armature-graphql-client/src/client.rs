//! GraphQL client implementation.

use std::sync::Arc;
use std::time::Duration;
use reqwest::Client;
use serde_json::Value;
use tracing::{debug, info};
use futures::StreamExt;

use crate::{
    GraphQLClientConfig, GraphQLError, Result,
    GraphQLResponse, QueryBuilder, MutationBuilder, SubscriptionBuilder,
    BatchRequest, BatchResponse, SubscriptionStream,
};
use crate::request::GraphQLRequest;
use crate::subscription::protocol::{ClientMessage, ServerMessage, SubscribePayload};

/// GraphQL client.
#[derive(Clone)]
pub struct GraphQLClient {
    http_client: Client,
    config: Arc<GraphQLClientConfig>,
}

impl GraphQLClient {
    /// Create a new GraphQL client with the given endpoint.
    pub fn new(endpoint: impl Into<String>) -> Self {
        let config = GraphQLClientConfig::new(endpoint);
        Self::with_config(config)
    }

    /// Create a new GraphQL client with custom configuration.
    pub fn with_config(config: GraphQLClientConfig) -> Self {
        let http_client = Client::builder()
            .timeout(config.timeout)
            .user_agent(&config.user_agent)
            .gzip(true)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            http_client,
            config: Arc::new(config),
        }
    }

    /// Get the configuration.
    pub fn config(&self) -> &GraphQLClientConfig {
        &self.config
    }

    /// Create a query builder.
    pub fn query(&self, query: impl Into<String>) -> QueryBuilder<'_> {
        QueryBuilder::new(self, query)
    }

    /// Create a mutation builder.
    pub fn mutation(&self, mutation: impl Into<String>) -> MutationBuilder<'_> {
        MutationBuilder::new(self, mutation)
    }

    /// Create a subscription builder.
    pub fn subscribe(&self, subscription: impl Into<String>) -> SubscriptionBuilder<'_> {
        SubscriptionBuilder::new(self, subscription)
    }

    /// Execute a batch of requests.
    pub async fn batch(&self, batch: BatchRequest) -> Result<BatchResponse> {
        if batch.is_empty() {
            return Ok(BatchResponse::new(Vec::new()));
        }

        debug!(count = batch.len(), "Executing batch request");

        let mut request = self.http_client.post(&self.config.endpoint);

        // Add default headers
        for (name, value) in &self.config.default_headers {
            request = request.header(name.as_str(), value.as_str());
        }

        request = request.header("Content-Type", "application/json");

        let response = request
            .json(&batch.into_requests())
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(GraphQLError::Http(
                response.error_for_status().unwrap_err()
            ));
        }

        let responses: Vec<GraphQLResponse<Value>> = response.json().await?;
        Ok(BatchResponse::new(responses))
    }

    /// Execute a single request.
    pub(crate) async fn execute_request(
        &self,
        request: GraphQLRequest,
        extra_headers: Vec<(String, String)>,
        timeout: Option<Duration>,
    ) -> Result<GraphQLResponse<Value>> {
        debug!(query = %request.query, "Executing GraphQL request");

        let mut http_request = self.http_client.post(&self.config.endpoint);

        // Add default headers
        for (name, value) in &self.config.default_headers {
            http_request = http_request.header(name.as_str(), value.as_str());
        }

        // Add extra headers
        for (name, value) in extra_headers {
            http_request = http_request.header(name.as_str(), value.as_str());
        }

        http_request = http_request.header("Content-Type", "application/json");

        if let Some(timeout) = timeout {
            http_request = http_request.timeout(timeout);
        }

        let response = http_request
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(GraphQLError::Http(
                response.error_for_status().unwrap_err()
            ));
        }

        let graphql_response: GraphQLResponse<Value> = response.json().await?;
        Ok(graphql_response)
    }

    /// Execute a subscription.
    pub(crate) async fn execute_subscription(
        &self,
        request: GraphQLRequest,
        _extra_headers: Vec<(String, String)>,
    ) -> Result<SubscriptionStream<Value>> {
        let ws_endpoint = self.config.ws_endpoint.as_ref()
            .ok_or_else(|| GraphQLError::Config("WebSocket endpoint not configured".to_string()))?;

        info!(endpoint = %ws_endpoint, "Starting GraphQL subscription");

        // Connect to WebSocket
        let (ws_stream, _) = tokio_tungstenite::connect_async(ws_endpoint)
            .await
            .map_err(|e| GraphQLError::WebSocket(e.to_string()))?;

        let (mut write, mut read) = ws_stream.split();

        // Send connection init
        let init_msg = serde_json::to_string(&ClientMessage::ConnectionInit { payload: None })
            .map_err(GraphQLError::Json)?;

        use futures::SinkExt;
        use tokio_tungstenite::tungstenite::Message;

        write.send(Message::Text(init_msg.into()))
            .await
            .map_err(|e| GraphQLError::WebSocket(e.to_string()))?;

        // Wait for connection_ack
        if let Some(msg) = read.next().await {
            let msg = msg.map_err(|e| GraphQLError::WebSocket(e.to_string()))?;
            if let Message::Text(text) = msg {
                let server_msg: ServerMessage = serde_json::from_str(&text)?;
                match server_msg {
                    ServerMessage::ConnectionAck { .. } => {
                        debug!("WebSocket connection acknowledged");
                    }
                    _ => {
                        return Err(GraphQLError::WebSocket("Expected connection_ack".to_string()));
                    }
                }
            }
        }

        // Send subscribe message
        let subscription_id = format!("{:x}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos());

        let subscribe_msg = serde_json::to_string(&ClientMessage::Subscribe {
            id: subscription_id.clone(),
            payload: SubscribePayload {
                query: request.query,
                operation_name: request.operation_name,
                variables: request.variables,
                extensions: request.extensions,
            },
        })?;

        write.send(Message::Text(subscribe_msg.into()))
            .await
            .map_err(|e| GraphQLError::WebSocket(e.to_string()))?;

        // Create the stream
        let stream = futures::stream::unfold(read, |mut read| async move {
            loop {
                match read.next().await {
                    Some(Ok(Message::Text(text))) => {
                        match serde_json::from_str::<ServerMessage>(&text) {
                            Ok(ServerMessage::Next { payload, .. }) => {
                                let response = GraphQLResponse {
                                    data: Some(payload),
                                    errors: None,
                                    extensions: None,
                                };
                                return Some((Ok(response), read));
                            }
                            Ok(ServerMessage::Error { payload, .. }) => {
                                let errors: Vec<crate::GraphQLResponseError> = payload
                                    .into_iter()
                                    .filter_map(|v| serde_json::from_value(v).ok())
                                    .collect();
                                return Some((Err(GraphQLError::GraphQL(errors)), read));
                            }
                            Ok(ServerMessage::Complete { .. }) => {
                                return None;
                            }
                            Ok(ServerMessage::Ping { payload: _ }) => {
                                // Should respond with pong, but we continue
                                continue;
                            }
                            Ok(_) => continue,
                            Err(e) => {
                                return Some((Err(GraphQLError::Json(e)), read));
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        return None;
                    }
                    Some(Ok(_)) => continue,
                    Some(Err(e)) => {
                        return Some((Err(GraphQLError::WebSocket(e.to_string())), read));
                    }
                    None => return None,
                }
            }
        });

        Ok(SubscriptionStream::new(stream))
    }
}

impl Default for GraphQLClient {
    fn default() -> Self {
        Self::with_config(GraphQLClientConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = GraphQLClient::new("http://localhost:4000/graphql");
        assert_eq!(client.config().endpoint, "http://localhost:4000/graphql");
    }

    #[test]
    fn test_client_with_config() {
        let config = GraphQLClientConfig::builder()
            .endpoint("https://api.example.com/graphql")
            .timeout(Duration::from_secs(60))
            .bearer_auth("token123")
            .build();

        let client = GraphQLClient::with_config(config);
        assert_eq!(client.config().endpoint, "https://api.example.com/graphql");
        assert_eq!(client.config().timeout, Duration::from_secs(60));
    }
}

