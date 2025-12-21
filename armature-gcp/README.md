# armature-gcp

Google Cloud Platform services integration for the Armature framework.

## Features

- **Cloud Storage** - Object storage
- **Firestore** - NoSQL database
- **Pub/Sub** - Message queues
- **Secret Manager** - Secrets management
- **BigQuery** - Data warehouse

## Installation

```toml
[dependencies]
armature-gcp = "0.1"
```

## Quick Start

```rust
use armature_gcp::{StorageClient, FirestoreClient, PubSubClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Cloud Storage
    let storage = StorageClient::new().await?;
    storage.upload("bucket", "object", bytes).await?;

    // Firestore
    let firestore = FirestoreClient::new("project-id").await?;
    firestore.set("collection", "doc", data).await?;

    // Pub/Sub
    let pubsub = PubSubClient::new("project-id").await?;
    pubsub.publish("topic", message).await?;

    Ok(())
}
```

## Authentication

Credentials are loaded from:
1. `GOOGLE_APPLICATION_CREDENTIALS` environment variable
2. Default application credentials
3. Compute Engine metadata service

## License

MIT OR Apache-2.0

