# armature-diesel

Diesel async database integration for the Armature framework.

## Features

- **Async/Await** - Non-blocking database operations
- **Connection Pooling** - deadpool, bb8, mobc support
- **PostgreSQL** - Full PostgreSQL support
- **MySQL** - MySQL/MariaDB support
- **Transactions** - ACID-compliant transactions
- **Migrations** - Diesel migration support

## Installation

```toml
[dependencies]
armature-diesel = "0.1"
```

## Quick Start

```rust
use armature_diesel::{Pool, establish_pool};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = establish_pool("postgres://localhost/mydb").await?;

    // Query
    let users = pool.interact(|conn| {
        users::table.load::<User>(conn)
    }).await??;

    // Transaction
    pool.transaction(|conn| {
        diesel::insert_into(users::table)
            .values(&new_user)
            .execute(conn)?;
        diesel::insert_into(profiles::table)
            .values(&profile)
            .execute(conn)
    }).await??;

    Ok(())
}
```

## Pool Configuration

```rust
let pool = Pool::builder()
    .max_size(20)
    .min_idle(5)
    .connection_timeout(Duration::from_secs(5))
    .build("postgres://localhost/mydb")
    .await?;
```

## License

MIT OR Apache-2.0

