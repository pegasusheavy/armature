//! Redis Pub/Sub support.

use futures::StreamExt;
use redis::Client;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

use crate::{RedisConfig, RedisError, Result};

/// A Redis Pub/Sub message.
#[derive(Debug, Clone)]
pub struct Message {
    /// Channel name.
    pub channel: String,
    /// Message payload.
    pub payload: String,
    /// Pattern (for pattern subscriptions).
    pub pattern: Option<String>,
}

/// A subscription handle.
pub struct Subscription {
    /// Receiver for messages.
    receiver: mpsc::Receiver<Message>,
    /// Channel name.
    channel: String,
}

impl Subscription {
    /// Create a new subscription.
    fn new(receiver: mpsc::Receiver<Message>, channel: String) -> Self {
        Self { receiver, channel }
    }

    /// Get the channel name.
    pub fn channel(&self) -> &str {
        &self.channel
    }

    /// Receive the next message.
    pub async fn recv(&mut self) -> Option<Message> {
        self.receiver.recv().await
    }

    /// Try to receive a message without blocking.
    pub fn try_recv(&mut self) -> Option<Message> {
        self.receiver.try_recv().ok()
    }
}

/// Redis Pub/Sub client.
pub struct PubSub {
    config: RedisConfig,
    client: Client,
}

impl PubSub {
    /// Create a new Pub/Sub client.
    pub fn new(config: RedisConfig) -> Result<Self> {
        let url = config.connection_url();
        let client = Client::open(url).map_err(|e| RedisError::Connection(e.to_string()))?;
        Ok(Self { config, client })
    }

    /// Subscribe to a channel.
    pub async fn subscribe(&self, channel: &str) -> Result<Subscription> {
        let (tx, rx) = mpsc::channel(100);
        let channel_name = channel.to_string();

        let mut pubsub = self
            .client
            .get_async_pubsub()
            .await
            .map_err(|e| RedisError::Connection(e.to_string()))?;

        pubsub
            .subscribe(&channel_name)
            .await
            .map_err(|e| RedisError::PubSub(e.to_string()))?;

        info!(channel = %channel_name, "Subscribed to Redis channel");

        // Spawn task to receive messages
        let channel_clone = channel_name.clone();
        tokio::spawn(async move {
            while let Some(msg) = pubsub.on_message().next().await {
                let payload: String = match msg.get_payload() {
                    Ok(p) => p,
                    Err(e) => {
                        error!(error = %e, "Failed to get message payload");
                        continue;
                    }
                };

                let message = Message {
                    channel: msg.get_channel_name().to_string(),
                    payload,
                    pattern: None,
                };

                debug!(channel = %message.channel, "Received pub/sub message");

                if tx.send(message).await.is_err() {
                    debug!(channel = %channel_clone, "Subscription receiver dropped");
                    break;
                }
            }
        });

        Ok(Subscription::new(rx, channel_name))
    }

    /// Subscribe to a pattern.
    pub async fn psubscribe(&self, pattern: &str) -> Result<Subscription> {
        let (tx, rx) = mpsc::channel(100);
        let pattern_str = pattern.to_string();

        let mut pubsub = self
            .client
            .get_async_pubsub()
            .await
            .map_err(|e| RedisError::Connection(e.to_string()))?;

        pubsub
            .psubscribe(&pattern_str)
            .await
            .map_err(|e| RedisError::PubSub(e.to_string()))?;

        info!(pattern = %pattern_str, "Subscribed to Redis pattern");

        let pattern_clone = pattern_str.clone();
        tokio::spawn(async move {
            while let Some(msg) = pubsub.on_message().next().await {
                let payload: String = match msg.get_payload() {
                    Ok(p) => p,
                    Err(e) => {
                        error!(error = %e, "Failed to get message payload");
                        continue;
                    }
                };

                let message = Message {
                    channel: msg.get_channel_name().to_string(),
                    payload,
                    pattern: Some(pattern_clone.clone()),
                };

                if tx.send(message).await.is_err() {
                    break;
                }
            }
        });

        Ok(Subscription::new(rx, pattern_str))
    }

    /// Publish a message to a channel.
    pub async fn publish(&self, channel: &str, message: &str) -> Result<u32> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| RedisError::Connection(e.to_string()))?;

        let receivers: u32 = redis::cmd("PUBLISH")
            .arg(channel)
            .arg(message)
            .query_async(&mut conn)
            .await
            .map_err(|e| RedisError::Command(e.to_string()))?;

        debug!(channel = %channel, receivers = receivers, "Published message");

        Ok(receivers)
    }
}

