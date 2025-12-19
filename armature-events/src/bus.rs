//! Event Bus implementation

use crate::event::{DynEventHandler, Event, EventHandlerError};
use dashmap::DashMap;
use std::any::TypeId;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Event bus for in-process event publishing and handling
#[derive(Clone)]
pub struct EventBus {
    /// Handlers registered for each event type
    handlers: Arc<DashMap<TypeId, Vec<Arc<dyn DynEventHandler>>>>,

    /// Configuration
    config: Arc<EventBusConfig>,
}

/// Event bus configuration
#[derive(Debug, Clone)]
pub struct EventBusConfig {
    /// Enable async event handling
    pub async_handling: bool,

    /// Continue on handler error
    pub continue_on_error: bool,

    /// Enable event logging
    pub enable_logging: bool,
}

impl Default for EventBusConfig {
    fn default() -> Self {
        Self {
            async_handling: true,
            continue_on_error: true,
            enable_logging: true,
        }
    }
}

impl EventBus {
    /// Create new event bus
    pub fn new() -> Self {
        Self::with_config(EventBusConfig::default())
    }

    /// Create event bus with custom config
    pub fn with_config(config: EventBusConfig) -> Self {
        Self {
            handlers: Arc::new(DashMap::new()),
            config: Arc::new(config),
        }
    }

    /// Subscribe a handler to an event type
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let bus = EventBus::new();
    /// bus.subscribe::<MyEvent, _>(MyHandler::new());
    /// ```
    pub fn subscribe<E, H>(&self, handler: H)
    where
        E: Event + Clone + 'static,
        H: DynEventHandler + 'static,
    {
        let type_id = TypeId::of::<E>();
        let handler = Arc::new(handler);

        self.handlers.entry(type_id).or_default().push(handler);

        if self.config.enable_logging {
            debug!("Subscribed handler for event type: {:?}", type_id);
        }
    }

    /// Publish an event
    ///
    /// All registered handlers for this event type will be invoked.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let bus = EventBus::new();
    /// bus.publish(MyEvent::new("data")).await?;
    /// ```
    pub async fn publish<E: Event>(&self, event: E) -> Result<(), EventBusError> {
        let type_id = TypeId::of::<E>();

        if self.config.enable_logging {
            info!(
                "Publishing event: {} (id: {})",
                event.event_name(),
                event.event_id()
            );
        }

        // Get handlers for this event type
        let handlers = match self.handlers.get(&type_id) {
            Some(handlers) => handlers.clone(),
            None => {
                if self.config.enable_logging {
                    warn!("No handlers registered for event: {}", event.event_name());
                }
                return Ok(());
            }
        };

        let event: Arc<dyn Event> = Arc::new(event);
        let mut errors = Vec::new();

        // Handle events asynchronously or synchronously
        if self.config.async_handling {
            // Spawn tasks for each handler
            let mut tasks = Vec::new();

            for handler in handlers.iter() {
                let handler = handler.clone();
                let event = event.clone();
                let task = tokio::spawn(async move { handler.handle_dyn(event.as_ref()).await });
                tasks.push(task);
            }

            // Wait for all handlers to complete
            for task in tasks {
                match task.await {
                    Ok(Ok(())) => {}
                    Ok(Err(e)) => {
                        error!("Handler failed: {}", e);
                        errors.push(e);
                        if !self.config.continue_on_error {
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Handler task panicked: {}", e);
                        errors.push(EventHandlerError::HandlerFailed(e.to_string()));
                        if !self.config.continue_on_error {
                            break;
                        }
                    }
                }
            }
        } else {
            // Handle events synchronously
            for handler in handlers.iter() {
                match handler.handle_dyn(event.as_ref()).await {
                    Ok(()) => {}
                    Err(e) => {
                        error!("Handler failed: {}", e);
                        errors.push(e);
                        if !self.config.continue_on_error {
                            break;
                        }
                    }
                }
            }
        }

        if !errors.is_empty() && !self.config.continue_on_error {
            return Err(EventBusError::HandlersFailed(errors));
        }

        if self.config.enable_logging {
            debug!("Event published successfully: {}", event.event_name());
        }

        Ok(())
    }

    /// Unsubscribe all handlers for an event type
    pub fn unsubscribe<E: Event + 'static>(&self) {
        let type_id = TypeId::of::<E>();
        self.handlers.remove(&type_id);

        if self.config.enable_logging {
            debug!("Unsubscribed all handlers for event type: {:?}", type_id);
        }
    }

    /// Clear all handlers
    pub fn clear(&self) {
        self.handlers.clear();
        if self.config.enable_logging {
            info!("Cleared all event handlers");
        }
    }

    /// Get handler count for an event type
    pub fn handler_count<E: Event + 'static>(&self) -> usize {
        let type_id = TypeId::of::<E>();
        self.handlers.get(&type_id).map(|h| h.len()).unwrap_or(0)
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Event bus errors
#[derive(Debug, thiserror::Error)]
pub enum EventBusError {
    #[error("One or more handlers failed")]
    HandlersFailed(Vec<EventHandlerError>),

    #[error("Event publishing failed: {0}")]
    PublishFailed(String),
}

/// Event bus builder
pub struct EventBusBuilder {
    config: EventBusConfig,
}

impl EventBusBuilder {
    /// Create new event bus builder
    pub fn new() -> Self {
        Self {
            config: EventBusConfig::default(),
        }
    }

    /// Enable/disable async handling
    pub fn async_handling(mut self, enabled: bool) -> Self {
        self.config.async_handling = enabled;
        self
    }

    /// Enable/disable continue on error
    pub fn continue_on_error(mut self, enabled: bool) -> Self {
        self.config.continue_on_error = enabled;
        self
    }

    /// Enable/disable logging
    pub fn enable_logging(mut self, enabled: bool) -> Self {
        self.config.enable_logging = enabled;
        self
    }

    /// Build the event bus
    pub fn build(self) -> EventBus {
        EventBus::with_config(self.config)
    }
}

impl Default for EventBusBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{Event, EventHandler, EventMetadata};
    use async_trait::async_trait;
    use chrono::Utc;
    use std::any::Any;
    use std::sync::atomic::{AtomicU32, Ordering};
    use uuid::Uuid;

    #[derive(Debug, Clone)]
    struct TestEvent {
        metadata: EventMetadata,
        message: String,
    }

    impl TestEvent {
        fn new(message: String) -> Self {
            Self {
                metadata: EventMetadata::new("test_event"),
                message,
            }
        }
    }

    impl Event for TestEvent {
        fn event_name(&self) -> &str {
            &self.metadata.name
        }

        fn event_id(&self) -> Uuid {
            self.metadata.id
        }

        fn timestamp(&self) -> chrono::DateTime<Utc> {
            self.metadata.timestamp
        }

        fn as_any(&self) -> &dyn Any {
            self
        }

        fn clone_event(&self) -> Box<dyn Event> {
            Box::new(self.clone())
        }
    }

    #[derive(Clone)]
    struct TestHandler {
        counter: Arc<AtomicU32>,
    }

    impl TestHandler {
        fn new() -> Self {
            Self {
                counter: Arc::new(AtomicU32::new(0)),
            }
        }

        fn count(&self) -> u32 {
            self.counter.load(Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl EventHandler<TestEvent> for TestHandler {
        async fn handle(&self, _event: &TestEvent) -> Result<(), EventHandlerError> {
            self.counter.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_event_bus_publish() {
        let bus = EventBus::new();
        let handler = TestHandler::new();
        let handler_clone = handler.clone();

        bus.subscribe::<TestEvent, _>(crate::event::TypedEventHandler::new(handler));

        let event = TestEvent::new("Hello".to_string());
        bus.publish(event).await.unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        assert_eq!(handler_clone.count(), 1);
    }

    #[tokio::test]
    async fn test_multiple_handlers() {
        let bus = EventBus::new();
        let handler1 = TestHandler::new();
        let handler2 = TestHandler::new();
        let h1_clone = handler1.clone();
        let h2_clone = handler2.clone();

        bus.subscribe::<TestEvent, _>(crate::event::TypedEventHandler::new(handler1));
        bus.subscribe::<TestEvent, _>(crate::event::TypedEventHandler::new(handler2));

        let event = TestEvent::new("Hello".to_string());
        bus.publish(event).await.unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        assert_eq!(h1_clone.count(), 1);
        assert_eq!(h2_clone.count(), 1);
    }

    #[tokio::test]
    async fn test_handler_count() {
        let bus = EventBus::new();
        assert_eq!(bus.handler_count::<TestEvent>(), 0);

        bus.subscribe::<TestEvent, _>(crate::event::TypedEventHandler::new(TestHandler::new()));
        assert_eq!(bus.handler_count::<TestEvent>(), 1);

        bus.subscribe::<TestEvent, _>(crate::event::TypedEventHandler::new(TestHandler::new()));
        assert_eq!(bus.handler_count::<TestEvent>(), 2);
    }
}
