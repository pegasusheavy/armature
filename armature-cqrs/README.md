# armature-cqrs

CQRS (Command Query Responsibility Segregation) for the Armature framework.

## Features

- **Commands** - Write operations
- **Queries** - Read operations
- **Handlers** - Command/query processing
- **Mediator** - Request dispatching

## Installation

```toml
[dependencies]
armature-cqrs = "0.1"
```

## Quick Start

```rust
use armature_cqrs::{Command, Query, Mediator};

#[derive(Command)]
struct CreateUser { name: String }

#[derive(Query)]
struct GetUser { id: String }

let mediator = Mediator::new()
    .register_command::<CreateUser, _>(create_user_handler)
    .register_query::<GetUser, _>(get_user_handler);

// Send command
mediator.send(CreateUser { name: "Alice".into() }).await?;

// Execute query
let user = mediator.query(GetUser { id: "123".into() }).await?;
```

## License

MIT OR Apache-2.0

