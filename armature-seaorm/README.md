# armature-seaorm

SeaORM integration for the Armature framework.

## Features

- **Async ORM** - Full async/await support
- **Active Record** - Entity-based CRUD operations
- **Connection Pooling** - Built-in connection management
- **Multiple Databases** - PostgreSQL, MySQL, SQLite
- **Migrations** - SeaORM migration support
- **Query Builder** - Type-safe query construction

## Installation

```toml
[dependencies]
armature-seaorm = "0.1"
```

## Quick Start

```rust
use armature_seaorm::{Database, DbConn};
use entity::user;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::connect("postgres://localhost/mydb").await?;

    // Find by ID
    let user = user::Entity::find_by_id(1).one(&db).await?;

    // Find all
    let users = user::Entity::find().all(&db).await?;

    // Insert
    let new_user = user::ActiveModel {
        name: Set("Alice".to_owned()),
        ..Default::default()
    };
    let user = new_user.insert(&db).await?;

    // Update
    let mut user: user::ActiveModel = user.into();
    user.name = Set("Bob".to_owned());
    user.update(&db).await?;

    // Delete
    user::Entity::delete_by_id(1).exec(&db).await?;

    Ok(())
}
```

## Pagination

```rust
use armature_seaorm::Paginator;

let paginator = user::Entity::find()
    .paginate(&db, 10);

let page = paginator.fetch_page(0).await?;
let total = paginator.num_pages().await?;
```

## License

MIT OR Apache-2.0

