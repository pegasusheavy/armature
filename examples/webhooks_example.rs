//! Example: Webhook Orchestration
//!
//! This example demonstrates how to use the webhook module for:
//! - Sending outgoing webhooks with signatures
//! - Receiving and verifying incoming webhooks
//! - Managing webhook endpoints with a registry
//!
//! Run with: `cargo run --example webhooks_example --features webhooks`

#![allow(dead_code, unused_imports)]

use armature_webhooks::{
    RetryPolicy, WebhookClient, WebhookConfig, WebhookDeliveryStatus, WebhookEndpoint,
    WebhookPayload, WebhookReceiver, WebhookRegistry, WebhookSignature,
};
use std::sync::Arc;

fn main() {
    println!("🪝 Armature Webhooks Example");
    println!("============================\n");

    demonstrate_payload_creation();
    demonstrate_signature();
    demonstrate_endpoint_registry();
    demonstrate_receiver();
    demonstrate_client_config();
}

fn demonstrate_payload_creation() {
    println!("1️⃣  Creating Webhook Payloads");
    println!("-----------------------------");

    // Create a simple webhook payload
    let payload = WebhookPayload::new("user.created").with_data(serde_json::json!({
        "user_id": "usr_123456",
        "email": "alice@example.com",
        "name": "Alice Smith",
        "created_at": "2024-01-15T10:30:00Z"
    }));

    println!("   Event: {}", payload.event);
    println!("   ID: {}", payload.id);
    println!("   Timestamp: {}", payload.timestamp);
    println!(
        "   Data: {}",
        serde_json::to_string_pretty(&payload.data).unwrap()
    );
    println!();
}

fn demonstrate_signature() {
    println!("2️⃣  Webhook Signatures");
    println!("----------------------");

    let secret = "whsec_my_super_secret_key";
    let signer = WebhookSignature::new(secret);

    // Create a payload
    let payload = WebhookPayload::new("order.completed")
        .with_data(serde_json::json!({"order_id": "ord_789"}));
    let payload_bytes = payload.to_bytes().unwrap();

    // Sign the payload
    let signature = signer.sign(&payload_bytes);
    println!("   Payload: {} bytes", payload_bytes.len());
    println!("   Signature: {}", signature);

    // Verify the signature
    let is_valid = signer.verify(&payload_bytes, &signature, 300).unwrap();
    println!("   Valid: {}", is_valid);
    println!();
}

fn demonstrate_endpoint_registry() {
    println!("3️⃣  Endpoint Registry");
    println!("---------------------");

    let registry = WebhookRegistry::new();

    // Register endpoints
    let endpoint1 = WebhookEndpoint::builder("https://api.example.com/webhooks")
        .events(vec!["user.*", "order.created"])
        .description("Main API webhook")
        .header("X-Custom-Header", "custom-value")
        .build();

    let endpoint2 = WebhookEndpoint::builder("https://analytics.example.com/events")
        .events(vec!["*"])
        .description("Analytics - all events")
        .build();

    let id1 = registry.register(endpoint1);
    let id2 = registry.register(endpoint2);

    println!("   Registered endpoints: {}", registry.count());
    println!("   Endpoint 1 ID: {}", id1);
    println!("   Endpoint 2 ID: {}", id2);

    // Query endpoints by event
    let user_endpoints = registry.get_endpoints_for_event("user.created");
    println!("   Endpoints for 'user.created': {}", user_endpoints.len());

    let order_endpoints = registry.get_endpoints_for_event("order.shipped");
    println!(
        "   Endpoints for 'order.shipped': {}",
        order_endpoints.len()
    );

    // Track failures
    registry.record_failure(&id1).unwrap();
    registry.record_failure(&id1).unwrap();
    let failing = registry.get_failing_endpoints(2);
    println!("   Failing endpoints (≥2 failures): {}", failing.len());
    println!();
}

fn demonstrate_receiver() {
    println!("4️⃣  Receiving Webhooks");
    println!("----------------------");

    let secret = "whsec_receiver_secret";

    // Sender side
    let signer = WebhookSignature::new(secret);
    let payload = WebhookPayload::new("payment.succeeded").with_data(serde_json::json!({
        "payment_id": "pay_abc123",
        "amount": 9999,
        "currency": "usd"
    }));
    let payload_bytes = payload.to_bytes().unwrap();
    let signature = signer.sign(&payload_bytes);

    // Receiver side
    let receiver = WebhookReceiver::new(secret);
    let received = receiver.receive(&payload_bytes, &signature).unwrap();

    println!("   Received event: {}", received.event);
    println!("   Verified: ✅");
    println!(
        "   Payment ID: {}",
        received.data.get("payment_id").unwrap()
    );
    println!();
}

fn demonstrate_client_config() {
    println!("5️⃣  Client Configuration");
    println!("------------------------");

    // Default configuration
    let default_config = WebhookConfig::default();
    println!("   Default timeout: {:?}", default_config.timeout);
    println!(
        "   Default retries: {}",
        default_config.retry_policy.max_attempts
    );

    // Custom configuration
    let custom_config = WebhookConfig::builder()
        .timeout_secs(60)
        .retry_policy(RetryPolicy::exponential(5))
        .max_payload_size(2 * 1024 * 1024) // 2MB
        .user_agent("MyApp/1.0")
        .build();

    println!("   Custom timeout: {:?}", custom_config.timeout);
    println!(
        "   Custom retries: {}",
        custom_config.retry_policy.max_attempts
    );
    println!(
        "   Custom max payload: {} bytes",
        custom_config.max_payload_size
    );
    println!();

    // Retry policies
    println!("   Retry Policies:");
    let no_retry = RetryPolicy::none();
    println!("      - none: {} attempts", no_retry.max_attempts);

    let fixed = RetryPolicy::fixed(3, std::time::Duration::from_secs(5));
    println!(
        "      - fixed: {} attempts, {:?} delay",
        fixed.max_attempts, fixed.initial_delay
    );

    let exponential = RetryPolicy::exponential(5);
    println!(
        "      - exponential: {} attempts, {}x backoff",
        exponential.max_attempts, exponential.backoff_multiplier
    );
    println!();

    println!("✅ Webhooks module ready!");
    println!();
    println!("Usage in your application:");
    println!("```rust");
    println!("let client = WebhookClient::new(WebhookConfig::default());");
    println!("let payload = WebhookPayload::new(\"user.created\")");
    println!("    .with_data(json!({{\"user_id\": \"123\"}}));");
    println!("client.send(\"https://example.com/webhook\", payload).await?;");
    println!("```");
}
