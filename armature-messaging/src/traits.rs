//! Traits for message broker implementations.

use crate::config::{ExchangeConfig, QueueConfig};
use crate::error::MessagingError;
use crate::message::{AckResult, ConsumeOptions, PublishOptions, RawMessage};
use async_trait::async_trait;
use futures::Stream;
use serde::Serialize;
use std::pin::Pin;

/// A message consumer stream.
pub type MessageStream = Pin<Box<dyn Stream<Item = Result<RawMessage, MessagingError>> + Send>>;

/// Trait for message broker connections.
#[async_trait]
pub trait BrokerConnection: Send + Sync {
    /// Get a channel for publishing and consuming messages.
    async fn channel(&self) -> Result<Box<dyn BrokerChannel>, MessagingError>;

    /// Check if the connection is healthy.
    async fn is_healthy(&self) -> bool;

    /// Close the connection gracefully.
    async fn close(&self) -> Result<(), MessagingError>;
}

/// Trait for message broker channels.
#[async_trait]
pub trait BrokerChannel: Send + Sync {
    /// Declare a queue.
    async fn declare_queue(&self, config: &QueueConfig) -> Result<(), MessagingError>;

    /// Declare an exchange (RabbitMQ specific, no-op for others).
    async fn declare_exchange(&self, config: &ExchangeConfig) -> Result<(), MessagingError>;

    /// Bind a queue to an exchange.
    async fn bind_queue(
        &self,
        queue: &str,
        exchange: &str,
        routing_key: &str,
    ) -> Result<(), MessagingError>;

    /// Publish raw bytes.
    async fn publish_bytes(
        &self,
        options: &PublishOptions,
        body: &[u8],
    ) -> Result<(), MessagingError>;

    /// Start consuming messages from a queue.
    async fn consume(&self, options: ConsumeOptions) -> Result<MessageStream, MessagingError>;

    /// Acknowledge a message.
    async fn ack(&self, delivery_tag: u64) -> Result<(), MessagingError>;

    /// Reject a message.
    async fn reject(&self, delivery_tag: u64, requeue: bool) -> Result<(), MessagingError>;

    /// Close the channel.
    async fn close(&self) -> Result<(), MessagingError>;
}

/// Extension trait for publishing serializable messages.
#[async_trait]
pub trait BrokerChannelExt: BrokerChannel {
    /// Publish a serializable message.
    async fn publish<T: Serialize + Send + Sync>(
        &self,
        options: &PublishOptions,
        message: &T,
    ) -> Result<(), MessagingError> {
        let bytes = serde_json::to_vec(message)
            .map_err(|e| MessagingError::SerializationError(e.to_string()))?;
        self.publish_bytes(options, &bytes).await
    }
}

// Implement BrokerChannelExt for all BrokerChannel implementations
impl<T: BrokerChannel + ?Sized> BrokerChannelExt for T {}

/// Handler function type for processing messages.
pub type MessageHandler<T> =
    Box<dyn Fn(T) -> Pin<Box<dyn std::future::Future<Output = AckResult> + Send>> + Send + Sync>;

/// Trait for typed message consumers.
#[async_trait]
pub trait Consumer: Send + Sync {
    /// The message type this consumer handles.
    type Message: serde::de::DeserializeOwned + Send;

    /// Process a message and return the acknowledgment result.
    async fn handle(&self, message: Self::Message) -> AckResult;

    /// Get the consumer options.
    fn options(&self) -> ConsumeOptions;
}

/// Trait for message publishers.
#[async_trait]
pub trait Publisher: Send + Sync {
    /// Publish a message to the given routing key.
    async fn publish<T: Serialize + Send + Sync>(
        &self,
        routing_key: &str,
        message: &T,
    ) -> Result<(), MessagingError>;

    /// Publish a message with options.
    async fn publish_with_options<T: Serialize + Send + Sync>(
        &self,
        options: &PublishOptions,
        message: &T,
    ) -> Result<(), MessagingError>;
}

/// Trait for request-reply messaging patterns.
#[async_trait]
pub trait RpcClient: Send + Sync {
    /// Send a request and wait for a response.
    async fn call<Req, Res>(
        &self,
        routing_key: &str,
        request: &Req,
        timeout_ms: u64,
    ) -> Result<Res, MessagingError>
    where
        Req: Serialize + Send + Sync,
        Res: serde::de::DeserializeOwned + Send;
}

/// Trait for health checking broker connections.
#[async_trait]
pub trait HealthCheck: Send + Sync {
    /// Check if the broker is healthy.
    async fn check_health(&self) -> Result<(), MessagingError>;

    /// Get the broker name for health reporting.
    fn broker_name(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Verify traits are object-safe
    fn _assert_object_safe(_: &dyn BrokerConnection) {}

    #[test]
    fn test_ack_result_variants() {
        assert_eq!(AckResult::Acked, AckResult::Acked);
        assert_ne!(AckResult::Acked, AckResult::Rejected);
    }
}

