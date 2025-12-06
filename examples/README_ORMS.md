# Rust ORM Integration Examples

Comprehensive examples showing how to integrate popular Rust ORMs with Armature's dependency injection and lifecycle management system.

## Available Examples

### 1. **SQLx** - SQL-First Async Database Library
📄 **File:** `sqlx_database.rs`  
🔗 **Docs:** [README_SQLX.md](README_SQLX.md)

**Best For:**
- Raw SQL queries with compile-time verification
- Maximum performance
- Direct database access
- Projects that prefer SQL over ORM abstractions

**Key Features:**
- ✅ Async/await first-class support
- ✅ Compile-time query verification
- ✅ Zero-cost abstractions
- ✅ Multiple database support (PostgreSQL, MySQL, SQLite)

### 2. **Diesel** - The Most Mature Rust ORM
📄 **File:** `diesel_orm.rs`

**Best For:**
- Type-safe query building
- Compile-time guarantees
- Synchronous applications
- Migrations via CLI

**Key Features:**
- ✅ Type-safe query builder
- ✅ Compile-time SQL verification
- ✅ Schema generation & migrations
- ✅ Connection pooling (R2D2)
- ✅ Most mature and battle-tested

### 3. **SeaORM** - Modern Async-First ORM
📄 **File:** `seaorm_database.rs`

**Best For:**
- Modern async applications
- Entity-based modeling
- ActiveRecord pattern
- Migrations and CLI tools

**Key Features:**
- ✅ Async/await throughout
- ✅ ActiveModel & Entity patterns
- ✅ Built on SQLx
- ✅ Schema generation from entities
- ✅ Migration tools

---

## Quick Comparison

| Feature | SQLx | Diesel | SeaORM |
|---------|------|--------|--------|
| **Paradigm** | SQL-first | Query Builder | ActiveRecord/Entity |
| **Async** | ✅ Native | ❌ Sync | ✅ Native |
| **Type Safety** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| **Compile-Time Checks** | ✅ Yes | ✅ Yes | ⚠️ Partial |
| **Learning Curve** | Low | Medium | Low-Medium |
| **Maturity** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ |
| **Performance** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| **Migrations** | Manual/File | CLI Tools | CLI Tools |
| **Connection Pool** | Built-in | R2D2 | Built-in (SQLx) |
| **Databases** | PostgreSQL, MySQL, SQLite | PostgreSQL, MySQL, SQLite | PostgreSQL, MySQL, SQLite |

---

## Prerequisites

### PostgreSQL Database

All examples use PostgreSQL. Start it with Docker:

```bash
# Start PostgreSQL
docker run -d \
  --name postgres-armature \
  -e POSTGRES_PASSWORD=password \
  -p 5432:5432 \
  postgres:15

# Create databases for each example
docker exec -it postgres-armature psql -U postgres -c "CREATE DATABASE armature_demo;"
docker exec -it postgres-armature psql -U postgres -c "CREATE DATABASE armature_diesel;"
docker exec -it postgres-armature psql -U postgres -c "CREATE DATABASE armature_seaorm;"
```

---

## Running Examples

### SQLx Example

```bash
# Set environment variable
export DATABASE_URL="postgres://postgres:password@localhost:5432/armature_demo"

# Run example
cargo run --example sqlx_database
```

### Diesel Example

```bash
# Set environment variable
export DATABASE_URL="postgres://postgres:password@localhost:5432/armature_diesel"

# Run example
cargo run --example diesel_orm
```

### SeaORM Example

```bash
# Set environment variable
export DATABASE_URL="postgres://postgres:password@localhost:5432/armature_seaorm"

# Run example
cargo run --example seaorm_database
```

---

## Architecture Pattern

All examples follow the same three-layer architecture:

```
┌─────────────────────────────────────────────────────────────────┐
│                    DATABASE SERVICE                             │
│  • Connection pool management                                   │
│  • OnModuleInit → Connect & migrate                            │
│  • OnApplicationShutdown → Close connections                   │
│  • CRUD operations                                              │
└─────────────────────────────────────────────────────────────────┘
                            ↓ (injected via DI)
┌─────────────────────────────────────────────────────────────────┐
│                     USER SERVICE                                │
│  • Business logic layer                                         │
│  • Validation                                                   │
│  • Error handling                                               │
│  • Depends on DatabaseService                                  │
└─────────────────────────────────────────────────────────────────┘
                            ↓ (injected via DI)
┌─────────────────────────────────────────────────────────────────┐
│                    USER CONTROLLER                              │
│  • HTTP request handling                                        │
│  • JSON serialization                                           │
│  • Response formatting                                          │
│  • Depends on UserService                                      │
└─────────────────────────────────────────────────────────────────┘
```

---

## Detailed Feature Comparison

### 1. SQLx

**Pros:**
- ✅ Extremely fast (zero-cost abstractions)
- ✅ Compile-time query verification with `sqlx::query!` macro
- ✅ Async/await first-class
- ✅ Direct SQL control
- ✅ Great for complex queries

**Cons:**
- ❌ Requires running database for compile-time checks
- ❌ More boilerplate than ORM
- ❌ Manual schema management

**Use When:**
- You want maximum performance
- You prefer writing SQL
- You need complex queries
- You want compile-time query verification

**Code Example:**

```rust
// SQLx - Direct SQL with type safety
let users = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY id")
    .fetch_all(&pool)
    .await?;

// Or with compile-time verification
let users = sqlx::query_as!(
    User,
    "SELECT id, username, email, created_at FROM users ORDER BY id"
)
.fetch_all(&pool)
.await?;
```

### 2. Diesel

**Pros:**
- ✅ Type-safe query builder
- ✅ Compile-time SQL verification
- ✅ Excellent tooling (diesel CLI)
- ✅ Battle-tested and mature
- ✅ Great documentation

**Cons:**
- ❌ Synchronous only (blocking)
- ❌ Steeper learning curve
- ❌ More boilerplate than modern ORMs
- ❌ Requires diesel CLI for schema

**Use When:**
- You want type-safe query building
- You need compile-time guarantees
- Your application is synchronous
- You want mature, proven technology

**Code Example:**

```rust
// Diesel - Type-safe query builder
use schema::users::dsl::*;

let results = users
    .filter(username.like("%john%"))
    .order_by(id.desc())
    .limit(10)
    .load::<User>(&mut conn)?;

// Insert with type safety
let new_user = NewUser {
    username: "alice",
    email: "alice@example.com",
};

diesel::insert_into(users)
    .values(&new_user)
    .returning(User::as_returning())
    .get_result(&mut conn)?;
```

### 3. SeaORM

**Pros:**
- ✅ Modern async API
- ✅ ActiveRecord/Entity patterns
- ✅ Great ergonomics
- ✅ Built on SQLx (inherits benefits)
- ✅ Good tooling (sea-orm-cli)

**Cons:**
- ❌ Less mature than Diesel
- ❌ Smaller community
- ❌ Runtime query building (no compile-time verification)
- ❌ More abstractions (slightly slower)

**Use When:**
- You want modern async patterns
- You prefer ActiveRecord style
- You want good ergonomics
- You're building new async applications

**Code Example:**

```rust
// SeaORM - Entity-based with async
use entities::prelude::*;

// Find with filters
let users = UserEntity::find()
    .filter(Column::Username.contains("john"))
    .order_by_desc(Column::Id)
    .limit(10)
    .all(&db)
    .await?;

// Insert with ActiveModel
let user = ActiveModel {
    username: Set("alice".to_string()),
    email: Set("alice@example.com".to_string()),
    ..Default::default()
};

let result = user.insert(&db).await?;
```

---

## Lifecycle Hook Integration

All examples demonstrate proper lifecycle management:

```rust
// Database service implements lifecycle hooks
#[async_trait]
impl OnModuleInit for DatabaseService {
    async fn on_module_init(&self) -> LifecycleResult {
        // Connect to database
        // Run migrations
        // Initialize pool
        Ok(())
    }
}

#[async_trait]
impl OnApplicationShutdown for DatabaseService {
    async fn on_application_shutdown(&self, signal: Option<String>) -> LifecycleResult {
        // Close connections gracefully
        Ok(())
    }
}

// Register with lifecycle manager
lifecycle.register_on_init("DatabaseService".to_string(), db_service.clone()).await;
lifecycle.register_on_shutdown("DatabaseService".to_string(), db_service).await;

// Hooks called automatically by Armature
lifecycle.call_module_init_hooks().await?;
```

---

## Dependency Injection Flow

All examples use the same DI pattern:

```rust
// 1. Create database service
let db_service = Arc::new(DatabaseService::new(database_url));

// 2. Register lifecycle hooks
lifecycle.register_on_init("DB".to_string(), db_service.clone()).await;
lifecycle.register_on_shutdown("DB".to_string(), db_service.clone()).await;

// 3. Initialize (calls OnModuleInit)
lifecycle.call_module_init_hooks().await?;

// 4. Inject into service layer
let user_service = Arc::new(UserService::new((*db_service).clone()));

// 5. Inject into controller layer
let user_controller = UserController::new((*user_service).clone());

// 6. Use in your application
let users = user_controller.list_users(request).await?;
```

---

## CRUD Operations

### Create

```rust
// SQLx
sqlx::query_as::<_, User>("INSERT INTO users (username, email) VALUES ($1, $2) RETURNING *")
    .bind(username)
    .bind(email)
    .fetch_one(&pool)
    .await?;

// Diesel
diesel::insert_into(users::table)
    .values(&new_user)
    .returning(User::as_returning())
    .get_result(&mut conn)?;

// SeaORM
let user = ActiveModel {
    username: Set(username),
    email: Set(email),
    ..Default::default()
};
user.insert(&db).await?;
```

### Read

```rust
// SQLx
sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
    .bind(id)
    .fetch_optional(&pool)
    .await?;

// Diesel
users::table.find(id).first::<User>(&mut conn).optional()?;

// SeaORM
UserEntity::find_by_id(id).one(&db).await?;
```

### Update

```rust
// SQLx
sqlx::query_as::<_, User>("UPDATE users SET username = $1 WHERE id = $2 RETURNING *")
    .bind(new_username)
    .bind(id)
    .fetch_optional(&pool)
    .await?;

// Diesel
diesel::update(users::table.find(id))
    .set(users::username.eq(new_username))
    .returning(User::as_returning())
    .get_result(&mut conn)?;

// SeaORM
let mut user: ActiveModel = existing_user.into();
user.username = Set(new_username);
user.update(&db).await?;
```

### Delete

```rust
// SQLx
sqlx::query("DELETE FROM users WHERE id = $1")
    .bind(id)
    .execute(&pool)
    .await?;

// Diesel
diesel::delete(users::table.find(id)).execute(&mut conn)?;

// SeaORM
UserEntity::delete_by_id(id).exec(&db).await?;
```

---

## Performance Considerations

### SQLx
- **Fastest** - Zero-cost abstractions
- Best for high-throughput applications
- Minimal overhead

### Diesel
- **Very Fast** - Compile-time optimizations
- Synchronous can be faster for some workloads
- Excellent for CPU-bound operations

### SeaORM
- **Fast** - Built on SQLx
- Small overhead from abstractions
- Great for most applications

**Benchmark (approximate):**
- SQLx: ~1.0x baseline
- Diesel: ~1.05x
- SeaORM: ~1.15x

All are very fast! Choose based on ergonomics and requirements.

---

## Migration Strategies

### SQLx
```bash
# Create migration directory
mkdir -p migrations

# Create migration
sqlx migrate add create_users_table

# Edit migrations/TIMESTAMP_create_users_table.sql
# Run migrations
sqlx migrate run
```

### Diesel
```bash
# Setup Diesel
diesel setup

# Create migration
diesel migration generate create_users

# Edit up.sql and down.sql
# Run migrations
diesel migration run
```

### SeaORM
```bash
# Install CLI
cargo install sea-orm-cli

# Generate entity from database
sea-orm-cli generate entity -u postgres://...

# Create migration
sea-orm-cli migrate generate create_users

# Run migrations
sea-orm-cli migrate up
```

---

## Testing

All ORMs work great with Armature's testing utilities:

```rust
#[tokio::test]
async fn test_user_service() {
    // Create test database service
    let db = DatabaseService::new(test_database_url());
    
    // Initialize
    db.on_module_init().await.unwrap();
    
    // Create service
    let service = UserService::new(db);
    
    // Test operations
    let user = service.create_user(CreateUserRequest {
        username: "test".to_string(),
        email: "test@example.com".to_string(),
    }).await.unwrap();
    
    assert_eq!(user.username, "test");
}
```

---

## Production Recommendations

### Choose SQLx When:
- ✅ You need maximum performance
- ✅ You're comfortable writing SQL
- ✅ You have complex queries
- ✅ You want compile-time verification
- ✅ You're building high-throughput services

### Choose Diesel When:
- ✅ You want type-safe query building
- ✅ You need mature, battle-tested ORM
- ✅ Your application is synchronous
- ✅ You want compile-time guarantees
- ✅ You value ecosystem and docs

### Choose SeaORM When:
- ✅ You're building modern async apps
- ✅ You prefer ActiveRecord patterns
- ✅ You want good ergonomics
- ✅ You're starting a new project
- ✅ You value rapid development

---

## Common Patterns

### Connection Pool Configuration

```rust
// SQLx
let pool = PgPoolOptions::new()
    .max_connections(20)
    .min_connections(5)
    .acquire_timeout(Duration::from_secs(5))
    .connect(&url)
    .await?;

// Diesel (R2D2)
let manager = ConnectionManager::<PgConnection>::new(&url);
let pool = Pool::builder()
    .max_size(20)
    .min_idle(Some(5))
    .build(manager)?;

// SeaORM
let mut opt = ConnectOptions::new(&url);
opt.max_connections(20)
    .min_connections(5);
let db = Database::connect(opt).await?;
```

### Transaction Handling

```rust
// SQLx
let mut tx = pool.begin().await?;
sqlx::query("INSERT INTO users ...").execute(&mut *tx).await?;
sqlx::query("INSERT INTO logs ...").execute(&mut *tx).await?;
tx.commit().await?;

// Diesel
conn.transaction(|conn| {
    diesel::insert_into(users::table).values(&new_user).execute(conn)?;
    diesel::insert_into(logs::table).values(&new_log).execute(conn)?;
    Ok(())
})?;

// SeaORM
let txn = db.begin().await?;
user.insert(&txn).await?;
log.insert(&txn).await?;
txn.commit().await?;
```

---

## Troubleshooting

### "database not found"

```bash
# Create database
docker exec -it postgres-armature psql -U postgres -c "CREATE DATABASE your_db_name;"
```

### "connection refused"

```bash
# Check PostgreSQL is running
docker ps | grep postgres

# Start if not running
docker start postgres-armature
```

### Diesel: "diesel.toml not found"

```bash
# Setup Diesel
diesel setup
```

### SeaORM: "entity not found"

```bash
# Generate entities
sea-orm-cli generate entity -u $DATABASE_URL
```

---

## Summary

All three ORMs integrate seamlessly with Armature:

- ✅ **Lifecycle hooks** for connection management
- ✅ **Dependency injection** for clean architecture
- ✅ **Type safety** for compile-time guarantees
- ✅ **Production-ready** patterns

Choose based on your requirements:
- **Performance-critical?** → SQLx
- **Type-safe queries?** → Diesel
- **Modern async app?** → SeaORM

You can't go wrong with any of them! 🚀

---

## Further Reading

- [SQLx Documentation](https://docs.rs/sqlx)
- [Diesel Documentation](https://diesel.rs/)
- [SeaORM Documentation](https://www.sea-ql.org/SeaORM/)
- [Armature Lifecycle Hooks](../docs/LIFECYCLE_HOOKS.md)
- [Armature DI Guide](../docs/DI_GUIDE.md)

