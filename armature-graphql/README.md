# armature-graphql

GraphQL support for the Armature framework.

## Features

- **Schema Definition** - Type-safe GraphQL schemas
- **Resolvers** - Async resolver functions
- **Subscriptions** - Real-time updates via WebSocket
- **Playground** - Built-in GraphQL IDE
- **Validation** - Query validation and depth limiting

## Installation

```toml
[dependencies]
armature-graphql = "0.1"
```

## Quick Start

```rust
use armature_graphql::{GraphQLSchema, Query, Mutation};

struct QueryRoot;

#[Query]
impl QueryRoot {
    async fn hello(&self) -> &str {
        "Hello, World!"
    }

    async fn user(&self, id: ID) -> Option<User> {
        User::find(id).await
    }
}

#[tokio::main]
async fn main() {
    let schema = GraphQLSchema::build(QueryRoot, MutationRoot, EmptySubscription)
        .finish();

    let app = Application::new()
        .post("/graphql", graphql_handler(schema))
        .get("/playground", playground_handler());

    app.listen("0.0.0.0:3000").await.unwrap();
}
```

## Subscriptions

```rust
struct SubscriptionRoot;

#[Subscription]
impl SubscriptionRoot {
    async fn messages(&self) -> impl Stream<Item = Message> {
        // Return a stream of messages
    }
}
```

## License

MIT OR Apache-2.0

