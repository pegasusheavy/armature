//! Message types and utilities.

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;
use uuid::Uuid;

/// A message with typed payload.
#[derive(Debug, Clone)]
pub struct Message<T> {
    /// Unique message ID
    pub id: String,
    /// Message payload
    pub payload: T,
    /// Message headers/properties
    pub headers: HashMap<String, String>,
    /// Routing key (for RabbitMQ) or topic (for Kafka/NATS)
    pub routing_key: String,
    /// Timestamp when the message was created
    pub timestamp: SystemTime,
    /// Content type
    pub content_type: String,
    /// Correlation ID for request-reply patterns
    pub correlation_id: Option<String>,
    /// Reply-to address
    pub reply_to: Option<String>,
    /// Message priority (0-9)
    pub priority: Option<u8>,
    /// Delivery tag for acknowledgment
    pub delivery_tag: Option<u64>,
    /// Whether the message was redelivered
    pub redelivered: bool,
}

impl<T> Message<T> {
    /// Create a new message with the given payload.
    pub fn new(payload: T) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            payload,
            headers: HashMap::new(),
            routing_key: String::new(),
            timestamp: SystemTime::now(),
            content_type: "application/json".to_string(),
            correlation_id: None,
            reply_to: None,
            priority: None,
            delivery_tag: None,
            redelivered: false,
        }
    }

    /// Create a new message with routing key.
    pub fn with_routing_key(payload: T, routing_key: &str) -> Self {
        let mut msg = Self::new(payload);
        msg.routing_key = routing_key.to_string();
        msg
    }

    /// Set a header.
    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    /// Set the correlation ID.
    pub fn with_correlation_id(mut self, id: &str) -> Self {
        self.correlation_id = Some(id.to_string());
        self
    }

    /// Set the reply-to address.
    pub fn with_reply_to(mut self, reply_to: &str) -> Self {
        self.reply_to = Some(reply_to.to_string());
        self
    }

    /// Set the priority.
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = Some(priority.min(9));
        self
    }

    /// Get the message age in seconds.
    pub fn age_seconds(&self) -> u64 {
        SystemTime::now()
            .duration_since(self.timestamp)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }
}

impl<T: Serialize> Message<T> {
    /// Serialize the message payload to JSON bytes.
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(&self.payload)
    }
}

impl<T: DeserializeOwned> Message<T> {
    /// Create a message from raw bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        let payload: T = serde_json::from_slice(bytes)?;
        Ok(Self::new(payload))
    }
}

/// Builder for creating publish options.
#[derive(Debug, Clone, Default)]
pub struct PublishOptions {
    /// Exchange name (RabbitMQ) or topic prefix
    pub exchange: Option<String>,
    /// Routing key (RabbitMQ) or topic (Kafka/NATS)
    pub routing_key: String,
    /// Whether the message should be mandatory (RabbitMQ)
    pub mandatory: bool,
    /// Whether the message should be persistent
    pub persistent: bool,
    /// Message expiration in milliseconds
    pub expiration: Option<u64>,
    /// Headers to include
    pub headers: HashMap<String, String>,
    /// Correlation ID
    pub correlation_id: Option<String>,
    /// Reply-to queue/topic
    pub reply_to: Option<String>,
    /// Priority (0-9)
    pub priority: Option<u8>,
    /// Partition key (Kafka)
    pub partition_key: Option<String>,
}

impl PublishOptions {
    /// Create new publish options with routing key.
    pub fn new(routing_key: &str) -> Self {
        Self {
            routing_key: routing_key.to_string(),
            persistent: true,
            ..Default::default()
        }
    }

    /// Set the exchange.
    pub fn exchange(mut self, exchange: &str) -> Self {
        self.exchange = Some(exchange.to_string());
        self
    }

    /// Set as mandatory.
    pub fn mandatory(mut self) -> Self {
        self.mandatory = true;
        self
    }

    /// Set as non-persistent.
    pub fn transient(mut self) -> Self {
        self.persistent = false;
        self
    }

    /// Set expiration.
    pub fn expires_in(mut self, ms: u64) -> Self {
        self.expiration = Some(ms);
        self
    }

    /// Add a header.
    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    /// Set correlation ID.
    pub fn correlation_id(mut self, id: &str) -> Self {
        self.correlation_id = Some(id.to_string());
        self
    }

    /// Set reply-to.
    pub fn reply_to(mut self, reply_to: &str) -> Self {
        self.reply_to = Some(reply_to.to_string());
        self
    }

    /// Set priority.
    pub fn priority(mut self, priority: u8) -> Self {
        self.priority = Some(priority.min(9));
        self
    }

    /// Set partition key (Kafka).
    pub fn partition(mut self, key: &str) -> Self {
        self.partition_key = Some(key.to_string());
        self
    }
}

/// Options for consuming messages.
#[derive(Debug, Clone)]
pub struct ConsumeOptions {
    /// Queue name (RabbitMQ) or topic (Kafka/NATS)
    pub queue: String,
    /// Consumer tag
    pub consumer_tag: Option<String>,
    /// Whether to auto-acknowledge messages
    pub auto_ack: bool,
    /// Whether this is an exclusive consumer
    pub exclusive: bool,
    /// Number of messages to prefetch
    pub prefetch: Option<u16>,
    /// Consumer group (Kafka)
    pub consumer_group: Option<String>,
}

impl ConsumeOptions {
    /// Create new consume options.
    pub fn new(queue: &str) -> Self {
        Self {
            queue: queue.to_string(),
            consumer_tag: None,
            auto_ack: false,
            exclusive: false,
            prefetch: None,
            consumer_group: None,
        }
    }

    /// Set consumer tag.
    pub fn tag(mut self, tag: &str) -> Self {
        self.consumer_tag = Some(tag.to_string());
        self
    }

    /// Enable auto-acknowledge.
    pub fn auto_ack(mut self) -> Self {
        self.auto_ack = true;
        self
    }

    /// Set as exclusive consumer.
    pub fn exclusive(mut self) -> Self {
        self.exclusive = true;
        self
    }

    /// Set prefetch count.
    pub fn prefetch(mut self, count: u16) -> Self {
        self.prefetch = Some(count);
        self
    }

    /// Set consumer group (Kafka).
    pub fn group(mut self, group: &str) -> Self {
        self.consumer_group = Some(group.to_string());
        self
    }
}

/// Acknowledgment result for a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AckResult {
    /// Message was acknowledged successfully
    Acked,
    /// Message was rejected and should be requeued
    Requeued,
    /// Message was rejected and should be discarded/dead-lettered
    Rejected,
}

/// Raw message bytes received from a broker.
#[derive(Debug, Clone)]
pub struct RawMessage {
    /// Raw message body
    pub body: Vec<u8>,
    /// Message headers
    pub headers: HashMap<String, String>,
    /// Routing key or topic
    pub routing_key: String,
    /// Delivery tag for acknowledgment
    pub delivery_tag: u64,
    /// Whether the message was redelivered
    pub redelivered: bool,
    /// Exchange/topic the message was received from
    pub exchange: String,
    /// Timestamp
    pub timestamp: Option<SystemTime>,
    /// Correlation ID
    pub correlation_id: Option<String>,
    /// Reply-to
    pub reply_to: Option<String>,
    /// Message ID
    pub message_id: Option<String>,
}

impl RawMessage {
    /// Deserialize the message body into a typed payload.
    pub fn deserialize<T: DeserializeOwned>(&self) -> Result<Message<T>, serde_json::Error> {
        let payload: T = serde_json::from_slice(&self.body)?;
        Ok(Message {
            id: self.message_id.clone().unwrap_or_else(|| Uuid::new_v4().to_string()),
            payload,
            headers: self.headers.clone(),
            routing_key: self.routing_key.clone(),
            timestamp: self.timestamp.unwrap_or_else(SystemTime::now),
            content_type: "application/json".to_string(),
            correlation_id: self.correlation_id.clone(),
            reply_to: self.reply_to.clone(),
            priority: None,
            delivery_tag: Some(self.delivery_tag),
            redelivered: self.redelivered,
        })
    }

    /// Get the body as a string.
    pub fn body_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.body)
    }
}

/// Event published through the messaging system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event<T> {
    /// Event type/name
    pub event_type: String,
    /// Event payload
    pub data: T,
    /// Timestamp
    pub timestamp: i64,
    /// Event source
    pub source: String,
    /// Correlation ID
    pub correlation_id: Option<String>,
    /// Causation ID (ID of the event that caused this event)
    pub causation_id: Option<String>,
}

impl<T: Serialize> Event<T> {
    /// Create a new event.
    pub fn new(event_type: &str, data: T) -> Self {
        Self {
            event_type: event_type.to_string(),
            data,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64,
            source: String::new(),
            correlation_id: None,
            causation_id: None,
        }
    }

    /// Set the source.
    pub fn from_source(mut self, source: &str) -> Self {
        self.source = source.to_string();
        self
    }

    /// Set the correlation ID.
    pub fn with_correlation(mut self, id: &str) -> Self {
        self.correlation_id = Some(id.to_string());
        self
    }

    /// Set the causation ID.
    pub fn caused_by(mut self, id: &str) -> Self {
        self.causation_id = Some(id.to_string());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestPayload {
        name: String,
        value: i32,
    }

    #[test]
    fn test_message_creation() {
        let msg = Message::new(TestPayload {
            name: "test".to_string(),
            value: 42,
        });

        assert!(!msg.id.is_empty());
        assert_eq!(msg.payload.name, "test");
        assert_eq!(msg.payload.value, 42);
        assert!(!msg.redelivered);
    }

    #[test]
    fn test_message_with_routing_key() {
        let msg = Message::with_routing_key(
            TestPayload {
                name: "test".to_string(),
                value: 42,
            },
            "orders.created",
        );

        assert_eq!(msg.routing_key, "orders.created");
    }

    #[test]
    fn test_message_headers() {
        let msg = Message::new(TestPayload {
            name: "test".to_string(),
            value: 42,
        })
        .header("x-tenant", "acme")
        .header("x-version", "1.0");

        assert_eq!(msg.headers.get("x-tenant"), Some(&"acme".to_string()));
        assert_eq!(msg.headers.get("x-version"), Some(&"1.0".to_string()));
    }

    #[test]
    fn test_message_serialization() {
        let msg = Message::new(TestPayload {
            name: "test".to_string(),
            value: 42,
        });

        let bytes = msg.to_bytes().unwrap();
        let deserialized: Message<TestPayload> = Message::from_bytes(&bytes).unwrap();

        assert_eq!(deserialized.payload.name, "test");
        assert_eq!(deserialized.payload.value, 42);
    }

    #[test]
    fn test_publish_options() {
        let opts = PublishOptions::new("orders.created")
            .exchange("events")
            .mandatory()
            .expires_in(60000)
            .priority(5);

        assert_eq!(opts.routing_key, "orders.created");
        assert_eq!(opts.exchange, Some("events".to_string()));
        assert!(opts.mandatory);
        assert_eq!(opts.expiration, Some(60000));
        assert_eq!(opts.priority, Some(5));
    }

    #[test]
    fn test_consume_options() {
        let opts = ConsumeOptions::new("orders-queue")
            .tag("my-consumer")
            .prefetch(100)
            .group("order-processors");

        assert_eq!(opts.queue, "orders-queue");
        assert_eq!(opts.consumer_tag, Some("my-consumer".to_string()));
        assert_eq!(opts.prefetch, Some(100));
        assert_eq!(opts.consumer_group, Some("order-processors".to_string()));
    }

    #[test]
    fn test_event() {
        let event = Event::new(
            "user.created",
            TestPayload {
                name: "John".to_string(),
                value: 1,
            },
        )
        .from_source("user-service")
        .with_correlation("req-123");

        assert_eq!(event.event_type, "user.created");
        assert_eq!(event.source, "user-service");
        assert_eq!(event.correlation_id, Some("req-123".to_string()));
    }

    #[test]
    fn test_priority_capped_at_9() {
        let msg = Message::new("test").with_priority(15);
        assert_eq!(msg.priority, Some(9));
    }
}

