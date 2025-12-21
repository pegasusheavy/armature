# armature-webhooks

Webhook handling for the Armature framework.

## Features

- **Signature Verification** - Validate webhook signatures
- **Retry Support** - Automatic delivery retries
- **Event Registry** - Type-safe event handling
- **Idempotency** - Prevent duplicate processing
- **Provider Support** - Stripe, GitHub, Slack, etc.

## Installation

```toml
[dependencies]
armature-webhooks = "0.1"
```

## Quick Start

```rust
use armature_webhooks::{WebhookHandler, WebhookConfig};

let handler = WebhookHandler::new(WebhookConfig {
    secret: "your-webhook-secret",
    signature_header: "X-Signature",
});

let app = Application::new()
    .post("/webhooks", move |req| {
        let handler = handler.clone();
        async move {
            handler.handle(req, |event| async move {
                match event.event_type.as_str() {
                    "order.created" => process_order(event.data).await,
                    _ => Ok(()),
                }
            }).await
        }
    });
```

## Provider-Specific Handlers

### Stripe

```rust
let stripe = StripeWebhook::new("whsec_...");
stripe.handle(req, |event| async move {
    match event {
        StripeEvent::PaymentSucceeded(payment) => { ... }
        StripeEvent::CustomerCreated(customer) => { ... }
        _ => Ok(()),
    }
}).await
```

### GitHub

```rust
let github = GitHubWebhook::new("secret");
github.handle(req, |event| async move {
    match event {
        GitHubEvent::Push(push) => { ... }
        GitHubEvent::PullRequest(pr) => { ... }
        _ => Ok(()),
    }
}).await
```

## License

MIT OR Apache-2.0

