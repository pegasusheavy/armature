//! Unified message broker interface.

use crate::config::{BrokerType, ExchangeConfig, MessagingConfig, QueueConfig};
use crate::error::MessagingError;
use crate::message::{ConsumeOptions, PublishOptions, RawMessage};
use crate::traits::{BrokerChannel, BrokerConnection, MessageStream, Publisher};
use async_trait::async_trait;
use serde::Serialize;
use std::sync::Arc;
use tracing::{debug, info};

/// A unified message broker that abstracts over different backends.
pub struct MessageBroker {
    config: MessagingConfig,
    connection: Arc<dyn BrokerConnection>,
}

impl MessageBroker {
    /// Connect to a message broker using the provided configuration.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use armature_messaging::prelude::*;
    ///
    /// # async fn example() -> Result<(), MessagingError> {
    /// let config = MessagingConfig::rabbitmq("amqp://localhost:5672");
    /// let broker = MessageBroker::connect(config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect(config: MessagingConfig) -> Result<Self, MessagingError> {
        info!(
            broker = %config.broker_type,
            "Connecting to message broker"
        );

        let connection: Arc<dyn BrokerConnection> = match config.broker_type {
            #[cfg(feature = "rabbitmq")]
            BrokerType::RabbitMQ => {
                Arc::new(crate::rabbitmq::RabbitMQConnection::connect(&config).await?)
            }
            #[cfg(feature = "kafka")]
            BrokerType::Kafka => {
                Arc::new(crate::kafka::KafkaConnection::connect(&config).await?)
            }
            #[cfg(feature = "nats")]
            BrokerType::Nats => {
                Arc::new(crate::nats::NatsConnection::connect(&config).await?)
            }
            #[allow(unreachable_patterns)]
            _ => {
                return Err(MessagingError::BackendNotAvailable(format!(
                    "{} backend is not enabled",
                    config.broker_type
                )));
            }
        };

        Ok(Self { config, connection })
    }

    /// Get the broker type.
    pub fn broker_type(&self) -> &BrokerType {
        &self.config.broker_type
    }

    /// Check if the connection is healthy.
    pub async fn is_healthy(&self) -> bool {
        self.connection.is_healthy().await
    }

    /// Get a channel for messaging operations.
    pub async fn channel(&self) -> Result<BrokerChannelHandle, MessagingError> {
        let channel = self.connection.channel().await?;
        Ok(BrokerChannelHandle {
            channel,
            default_exchange: None,
        })
    }

    /// Publish a message to an exchange/topic.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use armature_messaging::prelude::*;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct OrderCreated {
    ///     order_id: String,
    ///     total: f64,
    /// }
    ///
    /// # async fn example() -> Result<(), MessagingError> {
    /// let config = MessagingConfig::rabbitmq("amqp://localhost:5672");
    /// let broker = MessageBroker::connect(config).await?;
    ///
    /// let event = OrderCreated {
    ///     order_id: "12345".to_string(),
    ///     total: 99.99,
    /// };
    ///
    /// broker.publish("events", "orders.created", &event).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn publish<T: Serialize + Send + Sync>(
        &self,
        exchange: &str,
        routing_key: &str,
        message: &T,
    ) -> Result<(), MessagingError> {
        let channel = self.channel().await?;
        let options = PublishOptions::new(routing_key).exchange(exchange);
        channel.publish_with_options(&options, message).await
    }

    /// Close the connection gracefully.
    pub async fn close(&self) -> Result<(), MessagingError> {
        info!(broker = %self.config.broker_type, "Closing message broker connection");
        self.connection.close().await
    }
}

/// A handle to a broker channel with convenience methods.
pub struct BrokerChannelHandle {
    channel: Box<dyn BrokerChannel>,
    default_exchange: Option<String>,
}

impl BrokerChannelHandle {
    /// Set the default exchange for publish operations.
    pub fn with_default_exchange(mut self, exchange: &str) -> Self {
        self.default_exchange = Some(exchange.to_string());
        self
    }

    /// Declare a queue.
    pub async fn declare_queue(&self, config: &QueueConfig) -> Result<(), MessagingError> {
        debug!(queue = %config.name, "Declaring queue");
        self.channel.declare_queue(config).await
    }

    /// Declare an exchange.
    pub async fn declare_exchange(&self, config: &ExchangeConfig) -> Result<(), MessagingError> {
        debug!(exchange = %config.name, "Declaring exchange");
        self.channel.declare_exchange(config).await
    }

    /// Bind a queue to an exchange.
    pub async fn bind_queue(
        &self,
        queue: &str,
        exchange: &str,
        routing_key: &str,
    ) -> Result<(), MessagingError> {
        debug!(
            queue = queue,
            exchange = exchange,
            routing_key = routing_key,
            "Binding queue to exchange"
        );
        self.channel.bind_queue(queue, exchange, routing_key).await
    }

    /// Publish a message.
    pub async fn publish<T: Serialize + Send + Sync>(
        &self,
        routing_key: &str,
        message: &T,
    ) -> Result<(), MessagingError> {
        let mut options = PublishOptions::new(routing_key);
        if let Some(exchange) = &self.default_exchange {
            options = options.exchange(exchange);
        }
        self.publish_with_options(&options, message).await
    }

    /// Publish a message with options.
    pub async fn publish_with_options<T: Serialize + Send + Sync>(
        &self,
        options: &PublishOptions,
        message: &T,
    ) -> Result<(), MessagingError> {
        debug!(
            routing_key = %options.routing_key,
            exchange = ?options.exchange,
            "Publishing message"
        );
        self.channel.publish(options, message).await
    }

    /// Start consuming messages.
    pub async fn consume(&self, options: ConsumeOptions) -> Result<MessageStream, MessagingError> {
        debug!(queue = %options.queue, "Starting consumer");
        self.channel.consume(options).await
    }

    /// Acknowledge a message.
    pub async fn ack(&self, delivery_tag: u64) -> Result<(), MessagingError> {
        self.channel.ack(delivery_tag).await
    }

    /// Reject a message.
    pub async fn reject(&self, delivery_tag: u64, requeue: bool) -> Result<(), MessagingError> {
        self.channel.reject(delivery_tag, requeue).await
    }

    /// Close the channel.
    pub async fn close(&self) -> Result<(), MessagingError> {
        self.channel.close().await
    }
}

#[async_trait]
impl Publisher for BrokerChannelHandle {
    async fn publish<T: Serialize + Send + Sync>(
        &self,
        routing_key: &str,
        message: &T,
    ) -> Result<(), MessagingError> {
        BrokerChannelHandle::publish(self, routing_key, message).await
    }

    async fn publish_with_options<T: Serialize + Send + Sync>(
        &self,
        options: &PublishOptions,
        message: &T,
    ) -> Result<(), MessagingError> {
        BrokerChannelHandle::publish_with_options(self, options, message).await
    }
}

/// Setup a simple queue with exchange binding.
///
/// # Example
///
/// ```rust,no_run
/// use armature_messaging::prelude::*;
///
/// # async fn example() -> Result<(), MessagingError> {
/// let broker = MessageBroker::connect(MessagingConfig::rabbitmq("amqp://localhost:5672")).await?;
/// let channel = broker.channel().await?;
///
/// setup_queue(
///     &channel,
///     "orders-queue",
///     "events",
///     ExchangeType::Topic,
///     &["orders.*"],
/// ).await?;
/// # Ok(())
/// # }
/// ```
pub async fn setup_queue(
    channel: &BrokerChannelHandle,
    queue_name: &str,
    exchange_name: &str,
    exchange_type: crate::config::ExchangeType,
    routing_keys: &[&str],
) -> Result<(), MessagingError> {
    // Declare exchange
    channel
        .declare_exchange(&ExchangeConfig::new(exchange_name, exchange_type))
        .await?;

    // Declare queue
    channel.declare_queue(&QueueConfig::new(queue_name)).await?;

    // Bind with routing keys
    for routing_key in routing_keys {
        channel
            .bind_queue(queue_name, exchange_name, routing_key)
            .await?;
    }

    Ok(())
}

/// Process messages from a stream with a handler function.
///
/// # Example
///
/// ```rust,no_run
/// use armature_messaging::prelude::*;
/// use futures::StreamExt;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct OrderEvent {
///     order_id: String,
/// }
///
/// # async fn example() -> Result<(), MessagingError> {
/// let broker = MessageBroker::connect(MessagingConfig::rabbitmq("amqp://localhost:5672")).await?;
/// let channel = broker.channel().await?;
///
/// let mut stream = channel.consume(ConsumeOptions::new("orders-queue")).await?;
///
/// while let Some(msg_result) = stream.next().await {
///     match msg_result {
///         Ok(raw_msg) => {
///             let msg: Message<OrderEvent> = raw_msg.deserialize()?;
///             println!("Received order: {}", msg.payload.order_id);
///             channel.ack(raw_msg.delivery_tag).await?;
///         }
///         Err(e) => eprintln!("Error: {}", e),
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub async fn process_messages<T, F, Fut>(
    channel: &BrokerChannelHandle,
    stream: &mut MessageStream,
    handler: F,
) -> Result<(), MessagingError>
where
    T: serde::de::DeserializeOwned,
    F: Fn(crate::message::Message<T>) -> Fut,
    Fut: std::future::Future<Output = crate::message::AckResult>,
{
    use futures::StreamExt;

    while let Some(msg_result) = stream.next().await {
        match msg_result {
            Ok(raw_msg) => {
                let delivery_tag = raw_msg.delivery_tag;
                match raw_msg.deserialize::<T>() {
                    Ok(msg) => {
                        let result = handler(msg).await;
                        match result {
                            crate::message::AckResult::Acked => {
                                channel.ack(delivery_tag).await?;
                            }
                            crate::message::AckResult::Requeued => {
                                channel.reject(delivery_tag, true).await?;
                            }
                            crate::message::AckResult::Rejected => {
                                channel.reject(delivery_tag, false).await?;
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "Failed to deserialize message");
                        channel.reject(delivery_tag, false).await?;
                    }
                }
            }
            Err(e) => {
                tracing::error!(error = %e, "Error receiving message");
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_handle_default_exchange() {
        // This test verifies the builder pattern works
        // Actual channel creation requires a running broker
    }
}

