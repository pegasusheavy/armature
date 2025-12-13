//! NATS message broker implementation

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use async_nats::Client;
use async_trait::async_trait;
use futures_util::StreamExt;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

use crate::{
    Message, MessageBroker, MessageHandler, MessagingConfig, MessagingError, ProcessingResult,
    PublishOptions, SubscribeOptions, Subscription, config::NatsConfig,
};

/// NATS message broker
pub struct NatsBroker {
    client: Client,
    #[allow(dead_code)]
    config: NatsConfig,
    active_flags: Arc<RwLock<Vec<Arc<AtomicBool>>>>,
    connected: Arc<AtomicBool>,
}

impl NatsBroker {
    /// Connect to NATS
    pub async fn connect(config: &MessagingConfig) -> Result<Self, MessagingError> {
        let nats_config = NatsConfig {
            base: config.clone(),
            ..Default::default()
        };
        Self::connect_with_config(nats_config).await
    }

    /// Connect with NATS-specific configuration
    pub async fn connect_with_config(config: NatsConfig) -> Result<Self, MessagingError> {
        info!(url = %config.base.url, "Connecting to NATS");

        let mut options = async_nats::ConnectOptions::new();

        if let Some(ref name) = config.name {
            options = options.name(name);
        }

        if let Some(ref username) = config.base.username
            && let Some(ref password) = config.base.password
        {
            options = options.user_and_password(username.clone(), password.clone());
        }

        if let Some(ref creds_file) = config.credentials_file {
            options = options
                .credentials_file(creds_file)
                .await
                .map_err(|e| MessagingError::Configuration(e.to_string()))?;
        }

        let client = options
            .connect(&config.base.url)
            .await
            .map_err(MessagingError::from)?;

        info!("Connected to NATS successfully");

        Ok(Self {
            client,
            config,
            active_flags: Arc::new(RwLock::new(Vec::new())),
            connected: Arc::new(AtomicBool::new(true)),
        })
    }

    fn build_headers(message: &Message) -> async_nats::HeaderMap {
        let mut headers = async_nats::HeaderMap::new();

        headers.insert("Nats-Msg-Id", message.id.as_str());
        headers.insert("timestamp", message.timestamp.to_rfc3339().as_str());

        if let Some(ref correlation_id) = message.correlation_id {
            headers.insert("correlation-id", correlation_id.as_str());
        }

        if let Some(ref content_type) = message.content_type {
            headers.insert("content-type", content_type.as_str());
        }

        if let Some(ref reply_to) = message.reply_to {
            headers.insert("reply-to", reply_to.as_str());
        }

        for (key, value) in &message.headers {
            headers.insert(key.as_str(), value.as_str());
        }

        headers
    }
}

#[async_trait]
impl MessageBroker for NatsBroker {
    type Subscription = NatsSubscription;

    async fn publish(&self, message: Message) -> Result<(), MessagingError> {
        self.publish_with_options(message, PublishOptions::default())
            .await
    }

    async fn publish_with_options(
        &self,
        message: Message,
        _options: PublishOptions,
    ) -> Result<(), MessagingError> {
        let subject = &message.topic;
        let headers = Self::build_headers(&message);

        debug!(subject = subject, message_id = %message.id, "Publishing message to NATS");

        self.client
            .publish_with_headers(subject.clone(), headers, message.payload.into())
            .await
            .map_err(MessagingError::from)?;

        Ok(())
    }

    async fn subscribe(
        &self,
        topic: &str,
        handler: Arc<dyn MessageHandler>,
    ) -> Result<Self::Subscription, MessagingError> {
        self.subscribe_with_options(topic, handler, SubscribeOptions::default())
            .await
    }

    async fn subscribe_with_options(
        &self,
        topic: &str,
        handler: Arc<dyn MessageHandler>,
        options: SubscribeOptions,
    ) -> Result<Self::Subscription, MessagingError> {
        let subscriber = if let Some(ref group) = options.consumer_group {
            // Queue group subscription for load balancing
            self.client
                .queue_subscribe(topic.to_string(), group.clone())
                .await
                .map_err(MessagingError::from)?
        } else {
            self.client
                .subscribe(topic.to_string())
                .await
                .map_err(MessagingError::from)?
        };

        let active = Arc::new(AtomicBool::new(true));

        let subscription = NatsSubscription {
            topic: topic.to_string(),
            active: active.clone(),
        };

        // Store active flag for cleanup
        self.active_flags.write().await.push(active.clone());

        // Spawn consumer task
        let topic_owned = topic.to_string();
        tokio::spawn(async move {
            consume_messages(subscriber, handler, &topic_owned, active).await;
        });

        info!(subject = topic, "Subscribed to NATS subject");
        Ok(subscription)
    }

    fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }

    async fn close(&self) -> Result<(), MessagingError> {
        info!("Closing NATS connection");
        self.connected.store(false, Ordering::SeqCst);

        // Stop all subscriptions by setting active flags to false
        let flags = self.active_flags.read().await;
        for flag in flags.iter() {
            flag.store(false, Ordering::SeqCst);
        }

        // Flush and close client
        self.client
            .flush()
            .await
            .map_err(|e| MessagingError::Connection(e.to_string()))?;

        Ok(())
    }
}

async fn consume_messages(
    mut subscriber: async_nats::Subscriber,
    handler: Arc<dyn MessageHandler>,
    topic: &str,
    active: Arc<AtomicBool>,
) {
    while active.load(Ordering::SeqCst) {
        match subscriber.next().await {
            Some(nats_msg) => {
                let message = nats_message_to_message(&nats_msg, topic);

                match handler.handle(message).await {
                    Ok(result) => match result {
                        ProcessingResult::Success => {
                            debug!("Message processed successfully");
                        }
                        ProcessingResult::Retry => {
                            debug!(
                                "Message retry requested (NATS does not support built-in retry)"
                            );
                        }
                        ProcessingResult::DeadLetter | ProcessingResult::Reject => {
                            debug!("Message rejected");
                        }
                    },
                    Err(e) => {
                        error!(error = %e, "Message handler error");
                    }
                }
            }
            None => {
                debug!("Subscriber stream ended");
                break;
            }
        }
    }
}

fn nats_message_to_message(nats_msg: &async_nats::Message, topic: &str) -> Message {
    let headers = HashMap::new();
    let mut message_id = None;
    let mut correlation_id = None;
    let mut content_type = None;
    let mut reply_to = None;
    let mut timestamp = chrono::Utc::now();

    if let Some(nats_headers) = nats_msg.headers.as_ref() {
        // Get specific headers we care about
        if let Some(value) = nats_headers.get("Nats-Msg-Id") {
            message_id = Some(AsRef::<str>::as_ref(&value).to_string());
        }
        if let Some(value) = nats_headers.get("correlation-id") {
            correlation_id = Some(AsRef::<str>::as_ref(&value).to_string());
        }
        if let Some(value) = nats_headers.get("content-type") {
            content_type = Some(AsRef::<str>::as_ref(&value).to_string());
        }
        if let Some(value) = nats_headers.get("reply-to") {
            reply_to = Some(AsRef::<str>::as_ref(&value).to_string());
        }
        if let Some(value) = nats_headers.get("timestamp") {
            let ts_str: &str = AsRef::<str>::as_ref(&value);
            if let Ok(ts) = chrono::DateTime::parse_from_rfc3339(ts_str) {
                timestamp = ts.with_timezone(&chrono::Utc);
            }
        }
    }

    // Use NATS reply if available
    if reply_to.is_none()
        && let Some(ref nats_reply) = nats_msg.reply
    {
        reply_to = Some(nats_reply.to_string());
    }

    Message {
        id: message_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        payload: nats_msg.payload.to_vec(),
        headers,
        topic: topic.to_string(),
        timestamp,
        correlation_id,
        reply_to,
        content_type,
        priority: None,
        ttl: None,
    }
}

/// NATS subscription handle
pub struct NatsSubscription {
    topic: String,
    active: Arc<AtomicBool>,
}

#[async_trait]
impl Subscription for NatsSubscription {
    async fn unsubscribe(&self) -> Result<(), MessagingError> {
        self.active.store(false, Ordering::SeqCst);
        info!(subject = %self.topic, "Unsubscribed from NATS subject");
        Ok(())
    }

    fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }

    fn topic(&self) -> &str {
        &self.topic
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nats_config() {
        let config = NatsConfig::new("nats://localhost:4222")
            .with_name("test-client")
            .with_jetstream();

        assert_eq!(config.base.url, "nats://localhost:4222");
        assert_eq!(config.name, Some("test-client".to_string()));
        assert!(config.jetstream);
    }
}
