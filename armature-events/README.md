# armature-events

Event system for the Armature framework.

## Features

- **Event Bus** - Publish/subscribe events
- **Async Handlers** - Non-blocking event processing
- **Event Sourcing** - Append-only event log
- **Replay** - Rebuild state from events

## Installation

```toml
[dependencies]
armature-events = "0.1"
```

## Quick Start

```rust
use armature_events::{EventBus, Event};

let bus = EventBus::new();

// Subscribe
bus.subscribe::<UserCreated>(|event| async move {
    send_welcome_email(&event.email).await
});

// Publish
bus.publish(UserCreated { user_id: "123".into() }).await;
```

## License

MIT OR Apache-2.0
