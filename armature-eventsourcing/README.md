# armature-eventsourcing

Event sourcing support for the Armature framework.

## Features

- **Event Store** - Persistent event storage
- **Aggregates** - Domain-driven aggregates
- **Projections** - Read model generation
- **Snapshots** - Performance optimization

## Installation

```toml
[dependencies]
armature-eventsourcing = "0.1"
```

## Quick Start

```rust
use armature_eventsourcing::{Aggregate, EventStore};

let store = EventStore::postgres(pool).await?;

// Append events
store.append("order-123", vec![OrderCreated, ItemAdded]).await?;

// Load and replay
let events = store.load("order-123").await?;
```

## License

MIT OR Apache-2.0

