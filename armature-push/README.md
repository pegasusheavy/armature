# armature-push

Push notifications for the Armature framework.

## Features

- **Web Push** - Browser push notifications
- **FCM** - Firebase Cloud Messaging
- **APNS** - Apple Push Notification Service
- **Multi-Platform** - Send to all platforms at once
- **Topics** - Subscribe to notification topics

## Installation

```toml
[dependencies]
armature-push = "0.1"
```

## Quick Start

```rust
use armature_push::{PushService, Notification};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let push = PushService::new()
        .fcm("server-key")
        .apns("key.p8", "team-id", "key-id")
        .web_push("vapid-private-key");

    // Send notification
    push.send(Notification {
        title: "Hello!",
        body: "You have a new message",
        data: Some(json!({"message_id": 123})),
    }, &device_token).await?;

    Ok(())
}
```

## Web Push

```rust
// Register subscription
let subscription = WebPushSubscription {
    endpoint: "https://...",
    keys: PushKeys { p256dh: "...", auth: "..." },
};

push.web_push().send(&subscription, notification).await?;
```

## FCM

```rust
push.fcm().send(&fcm_token, notification).await?;
```

## APNS

```rust
push.apns().send(&device_token, notification).await?;
```

## License

MIT OR Apache-2.0

