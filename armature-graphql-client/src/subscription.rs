//! GraphQL subscription support.

use futures::Stream;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::{GraphQLResponse, Result};

/// A GraphQL subscription stream.
pub struct SubscriptionStream<T = Value> {
    inner: Pin<Box<dyn Stream<Item = Result<GraphQLResponse<T>>> + Send>>,
}

impl<T> SubscriptionStream<T> {
    /// Create a new subscription stream.
    pub fn new<S>(stream: S) -> Self
    where
        S: Stream<Item = Result<GraphQLResponse<T>>> + Send + 'static,
    {
        Self {
            inner: Box::pin(stream),
        }
    }
}

impl<T: DeserializeOwned + Unpin> Stream for SubscriptionStream<T> {
    type Item = Result<T>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Ready(Some(Ok(response))) => Poll::Ready(Some(response.into_result())),
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Subscription state for graphql-ws protocol.
#[derive(Debug, Clone)]
pub struct Subscription {
    /// Subscription ID.
    pub id: String,
    /// Whether the subscription is active.
    pub active: bool,
}

impl Subscription {
    /// Create a new subscription.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            active: true,
        }
    }

    /// Check if the subscription is active.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Mark the subscription as inactive.
    pub fn deactivate(&mut self) {
        self.active = false;
    }
}

/// graphql-ws protocol messages.
pub mod protocol {
    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    /// Client to server message types.
    #[derive(Debug, Clone, Serialize)]
    #[serde(tag = "type")]
    pub enum ClientMessage {
        /// Initialize connection.
        #[serde(rename = "connection_init")]
        ConnectionInit {
            #[serde(skip_serializing_if = "Option::is_none")]
            payload: Option<Value>,
        },
        /// Start a subscription.
        #[serde(rename = "subscribe")]
        Subscribe {
            id: String,
            payload: SubscribePayload,
        },
        /// Complete a subscription.
        #[serde(rename = "complete")]
        Complete { id: String },
        /// Ping for keep-alive.
        #[serde(rename = "ping")]
        Ping {
            #[serde(skip_serializing_if = "Option::is_none")]
            payload: Option<Value>,
        },
        /// Pong response.
        #[serde(rename = "pong")]
        Pong {
            #[serde(skip_serializing_if = "Option::is_none")]
            payload: Option<Value>,
        },
    }

    /// Subscribe payload.
    #[derive(Debug, Clone, Serialize)]
    pub struct SubscribePayload {
        pub query: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub operation_name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub variables: Option<Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub extensions: Option<Value>,
    }

    /// Server to client message types.
    #[derive(Debug, Clone, Deserialize)]
    #[serde(tag = "type")]
    pub enum ServerMessage {
        /// Connection acknowledged.
        #[serde(rename = "connection_ack")]
        ConnectionAck {
            #[serde(default)]
            payload: Option<Value>,
        },
        /// Subscription data.
        #[serde(rename = "next")]
        Next { id: String, payload: Value },
        /// Subscription error.
        #[serde(rename = "error")]
        Error { id: String, payload: Vec<Value> },
        /// Subscription complete.
        #[serde(rename = "complete")]
        Complete { id: String },
        /// Ping from server.
        #[serde(rename = "ping")]
        Ping {
            #[serde(default)]
            payload: Option<Value>,
        },
        /// Pong from server.
        #[serde(rename = "pong")]
        Pong {
            #[serde(default)]
            payload: Option<Value>,
        },
    }
}
