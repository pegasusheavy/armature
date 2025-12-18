#![allow(clippy::needless_question_mark)]
//! Event Bus Example
//!
//! Demonstrates in-process event publishing and handling.

use armature_events::*;
use async_trait::async_trait;
use chrono::Utc;
use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use uuid::Uuid;

// Define custom events
#[derive(Debug, Clone)]
struct UserCreatedEvent {
    metadata: EventMetadata,
    user_id: String,
    email: String,
}

impl UserCreatedEvent {
    fn new(user_id: String, email: String) -> Self {
        Self {
            metadata: EventMetadata::new("user_created"),
            user_id,
            email,
        }
    }
}

impl Event for UserCreatedEvent {
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

#[derive(Debug, Clone)]
struct UserDeletedEvent {
    metadata: EventMetadata,
    user_id: String,
}

impl UserDeletedEvent {
    fn new(user_id: String) -> Self {
        Self {
            metadata: EventMetadata::new("user_deleted"),
            user_id,
        }
    }
}

impl Event for UserDeletedEvent {
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

// Define event handlers
#[derive(Clone)]
struct EmailHandler {
    sent_count: Arc<AtomicU32>,
}

impl EmailHandler {
    fn new() -> Self {
        Self {
            sent_count: Arc::new(AtomicU32::new(0)),
        }
    }

    fn count(&self) -> u32 {
        self.sent_count.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl EventHandler<UserCreatedEvent> for EmailHandler {
    async fn handle(&self, event: &UserCreatedEvent) -> Result<(), EventHandlerError> {
        println!("📧 Sending welcome email to {}", event.email);
        self.sent_count.fetch_add(1, Ordering::SeqCst);
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        println!("✅ Email sent to {}", event.email);
        Ok(())
    }
}

#[derive(Clone)]
struct AnalyticsHandler {
    event_count: Arc<AtomicU32>,
}

impl AnalyticsHandler {
    fn new() -> Self {
        Self {
            event_count: Arc::new(AtomicU32::new(0)),
        }
    }

    fn count(&self) -> u32 {
        self.event_count.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl EventHandler<UserCreatedEvent> for AnalyticsHandler {
    async fn handle(&self, event: &UserCreatedEvent) -> Result<(), EventHandlerError> {
        println!("📊 Recording user creation in analytics: {}", event.user_id);
        self.event_count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[derive(Clone)]
struct AuditHandler;

#[async_trait]
impl EventHandler<UserCreatedEvent> for AuditHandler {
    async fn handle(&self, event: &UserCreatedEvent) -> Result<(), EventHandlerError> {
        println!(
            "📝 Audit log: User {} created at {}",
            event.user_id,
            event.timestamp()
        );
        Ok(())
    }
}

#[async_trait]
impl EventHandler<UserDeletedEvent> for AuditHandler {
    async fn handle(&self, event: &UserDeletedEvent) -> Result<(), EventHandlerError> {
        println!(
            "📝 Audit log: User {} deleted at {}",
            event.user_id,
            event.timestamp()
        );
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Event Bus Example ===\n");

    // 1. Create event bus
    println!("1. Creating Event Bus:");
    let bus = EventBusBuilder::new()
        .async_handling(true) // Handle events concurrently
        .continue_on_error(true) // Don't stop on handler errors
        .enable_logging(true) // Log events
        .build();
    println!("   ✅ Event bus created\n");

    // 2. Register handlers
    println!("2. Registering Event Handlers:");
    let email_handler = EmailHandler::new();
    let analytics_handler = AnalyticsHandler::new();
    let audit_handler = AuditHandler;

    let email_clone = email_handler.clone();
    let analytics_clone = analytics_handler.clone();

    bus.subscribe::<UserCreatedEvent, _>(TypedEventHandler::<UserCreatedEvent, _>::new(
        email_handler,
    ));
    bus.subscribe::<UserCreatedEvent, _>(TypedEventHandler::<UserCreatedEvent, _>::new(
        analytics_handler,
    ));
    bus.subscribe::<UserCreatedEvent, _>(TypedEventHandler::<UserCreatedEvent, _>::new(
        audit_handler.clone(),
    ));
    bus.subscribe::<UserDeletedEvent, _>(TypedEventHandler::<UserDeletedEvent, _>::new(
        audit_handler,
    ));

    println!("   📧 EmailHandler registered");
    println!("   📊 AnalyticsHandler registered");
    println!("   📝 AuditHandler registered (for both events)");
    println!(
        "   Handlers for UserCreatedEvent: {}",
        bus.handler_count::<UserCreatedEvent>()
    );
    println!(
        "   Handlers for UserDeletedEvent: {}",
        bus.handler_count::<UserDeletedEvent>()
    );
    println!();

    // 3. Publish events
    println!("3. Publishing Events:");
    println!();

    println!("   Publishing UserCreatedEvent...");
    let event = UserCreatedEvent::new("user-123".to_string(), "alice@example.com".to_string());
    bus.publish(event).await?;

    // Wait for async handlers to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    println!();

    println!("   Publishing another UserCreatedEvent...");
    let event2 = UserCreatedEvent::new("user-456".to_string(), "bob@example.com".to_string());
    bus.publish(event2).await?;

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    println!();

    println!("   Publishing UserDeletedEvent...");
    let event3 = UserDeletedEvent::new("user-123".to_string());
    bus.publish(event3).await?;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    println!();

    // 4. Check handler statistics
    println!("4. Handler Statistics:");
    println!("   📧 Emails sent: {}", email_clone.count());
    println!(
        "   📊 Analytics events recorded: {}",
        analytics_clone.count()
    );
    println!();

    // 5. Demonstrate event with correlation ID
    println!("5. Event with Correlation ID:");
    let correlation_id = Uuid::new_v4();
    let metadata = EventMetadata::new("user_created").with_correlation_id(correlation_id);

    let correlated_event = UserCreatedEvent {
        metadata,
        user_id: "user-789".to_string(),
        email: "charlie@example.com".to_string(),
    };

    println!(
        "   Publishing event with correlation ID: {}",
        correlation_id
    );
    bus.publish(correlated_event).await?;

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    println!();

    println!("=== Event Bus Example Complete ===\n");
    println!("💡 Key Features Demonstrated:");
    println!("   ✅ In-process event publishing");
    println!("   ✅ Multiple handlers per event");
    println!("   ✅ Async concurrent handling");
    println!("   ✅ Type-safe event dispatch");
    println!("   ✅ Event metadata (ID, timestamp, correlation)");
    println!("   ✅ Continue on error");
    println!();

    Ok(())
}
