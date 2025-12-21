# armature-azure

Microsoft Azure services integration for the Armature framework.

## Features

- **Blob Storage** - Object storage
- **Cosmos DB** - Global database
- **Service Bus** - Message queues and topics
- **Key Vault** - Secrets management
- **App Configuration** - Feature flags and config

## Installation

```toml
[dependencies]
armature-azure = "0.1"
```

## Quick Start

```rust
use armature_azure::{BlobClient, CosmosClient, ServiceBusClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Blob Storage
    let blob = BlobClient::new("connection_string").await?;
    blob.upload("container", "blob", bytes).await?;

    // Cosmos DB
    let cosmos = CosmosClient::new("endpoint", "key").await?;
    cosmos.create_item("database", "container", item).await?;

    // Service Bus
    let bus = ServiceBusClient::new("connection_string").await?;
    bus.send("queue", message).await?;

    Ok(())
}
```

## Authentication

Supports:
- Connection strings
- Managed Identity
- Service Principal
- Azure CLI credentials

## License

MIT OR Apache-2.0

