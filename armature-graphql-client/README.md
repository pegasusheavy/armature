# armature-graphql-client

GraphQL client with subscription support for the Armature framework.

## Features

- **Query/Mutation** - Execute GraphQL operations
- **Subscriptions** - Real-time updates via WebSocket
- **Type Generation** - Generate types from schema
- **Batching** - Automatic query batching
- **Caching** - Response caching

## Installation

```toml
[dependencies]
armature-graphql-client = "0.1"
```

## Quick Start

```rust
use armature_graphql_client::{Client, gql};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new("https://api.example.com/graphql");

    // Query
    let query = gql! {
        query GetUser($id: ID!) {
            user(id: $id) {
                id
                name
                email
            }
        }
    };

    let response = client
        .query(query)
        .variable("id", "123")
        .execute()
        .await?;

    Ok(())
}
```

## Subscriptions

```rust
let subscription = gql! {
    subscription OnMessage {
        messageAdded {
            id
            text
        }
    }
};

let mut stream = client.subscribe(subscription).await?;

while let Some(message) = stream.next().await {
    println!("New message: {:?}", message);
}
```

## License

MIT OR Apache-2.0

