# SQLx Database Integration Example

This example demonstrates how to integrate SQLx (PostgreSQL) with Armature's dependency injection system and lifecycle hooks.

## Features Demonstrated

- ✅ **SQLx Connection Pool** - Managed database connections
- ✅ **Lifecycle Hooks** - `OnModuleInit` and `OnApplicationShutdown`
- ✅ **Dependency Injection** - Injecting `DatabaseService` into `UserService` and `UserController`
- ✅ **CRUD Operations** - Create, Read, Update, Delete users
- ✅ **Error Handling** - Proper error propagation and handling
- ✅ **Connection Management** - Graceful connection opening and closing
- ✅ **Migrations** - Simple table creation on startup
- ✅ **Business Logic Layer** - Separation of concerns (Controller → Service → Repository)

## Prerequisites

### 1. PostgreSQL Database

You need a running PostgreSQL instance. The easiest way is using Docker:

```bash
# Start PostgreSQL in Docker
docker run -d \
  --name postgres-armature \
  -e POSTGRES_PASSWORD=password \
  -e POSTGRES_DB=armature_demo \
  -p 5432:5432 \
  postgres:15
```

**Alternative: Install PostgreSQL locally**

```bash
# macOS
brew install postgresql@15
brew services start postgresql@15

# Ubuntu/Debian
sudo apt install postgresql postgresql-contrib
sudo systemctl start postgresql

# Create database
createdb armature_demo
```

### 2. Environment Variable

Set the `DATABASE_URL` environment variable:

```bash
export DATABASE_URL="postgres://postgres:password@localhost:5432/armature_demo"
```

**Connection String Format:**
```
postgres://username:password@host:port/database
```

## Running the Example

### Step 1: Start PostgreSQL

```bash
docker run -d \
  --name postgres-armature \
  -e POSTGRES_PASSWORD=password \
  -e POSTGRES_DB=armature_demo \
  -p 5432:5432 \
  postgres:15
```

### Step 2: Set Environment Variable

```bash
export DATABASE_URL="postgres://postgres:password@localhost:5432/armature_demo"
```

### Step 3: Run the Example

```bash
cd /path/to/armature
cargo run --example sqlx_database
```

## Expected Output

```
╔══════════════════════════════════════════════════════════════╗
║         SQLx Database Integration Example                  ║
║       Armature DI + Lifecycle Hooks + PostgreSQL           ║
╚══════════════════════════════════════════════════════════════╝

📋 Configuration:
   Database: postgres://postgres:****@localhost:5432/armature_demo

🔧 Registering lifecycle hooks...
   ✅ Hooks registered

═══════════════════════════════════════════════════════════════
                  INITIALIZATION PHASE
═══════════════════════════════════════════════════════════════

📊 DatabaseService: Connecting to PostgreSQL...
   Connection: postgres://postgres:****@localhost:5432/armature_demo
   ✅ Database connected successfully!
   📋 Running migrations...
   ✅ Migrations complete!

═══════════════════════════════════════════════════════════════
                  DEPENDENCY INJECTION
═══════════════════════════════════════════════════════════════

🔌 Injecting DatabaseService into UserService...
   ✅ UserService created

🔌 Injecting UserService into UserController...
   ✅ UserController created

═══════════════════════════════════════════════════════════════
                  DATABASE OPERATIONS
═══════════════════════════════════════════════════════════════

📝 Creating test users...
   ✅ Created user: alice (ID: 1)
   ✅ Created user: bob (ID: 2)

📋 Listing all users...
   Found 2 users:
   - alice <alice@example.com> (ID: 1)
   - bob <bob@example.com> (ID: 2)

📊 Database statistics...
   {
     "connected": true,
     "database": "PostgreSQL",
     "total_users": 2
   }

═══════════════════════════════════════════════════════════════
                  GRACEFUL SHUTDOWN
═══════════════════════════════════════════════════════════════

📊 DatabaseService: Closing connections (signal: SIGTERM)...
   ✅ Database connections closed!

═══════════════════════════════════════════════════════════════
✅ Example complete! Database integration successful.
═══════════════════════════════════════════════════════════════
```

## Code Structure

### DatabaseService

The main database service with lifecycle hooks:

```rust
#[derive(Clone)]
pub struct DatabaseService {
    connection_string: String,
    pool: Arc<RwLock<Option<PgPool>>>,
}

impl Provider for DatabaseService {}

#[async_trait]
impl OnModuleInit for DatabaseService {
    async fn on_module_init(&self) -> LifecycleResult {
        // Connect to database
        // Run migrations
        // Store pool
    }
}

#[async_trait]
impl OnApplicationShutdown for DatabaseService {
    async fn on_application_shutdown(&self, signal: Option<String>) -> LifecycleResult {
        // Close connections gracefully
    }
}
```

### UserService (Business Logic)

Service layer that depends on `DatabaseService`:

```rust
#[derive(Clone)]
pub struct UserService {
    db: Arc<DatabaseService>,
}

impl UserService {
    pub fn new(db: DatabaseService) -> Self {
        Self { db: Arc::new(db) }
    }

    pub async fn list_users(&self) -> Result<Vec<User>, String> {
        self.db.get_all_users().await
            .map_err(|e| format!("Failed to fetch users: {}", e))
    }

    // ... other methods
}

impl Provider for UserService {}
```

### UserController

Controller that depends on `UserService`:

```rust
pub struct UserController {
    user_service: Arc<UserService>,
}

impl UserController {
    pub fn new(user_service: UserService) -> Self {
        Self {
            user_service: Arc::new(user_service),
        }
    }

    pub async fn list_users(&self, _req: HttpRequest) -> Result<HttpResponse, ArmatureError> {
        match self.user_service.list_users().await {
            Ok(users) => {
                // Return JSON response
            }
            Err(e) => {
                // Return error response
            }
        }
    }
}
```

### Dependency Injection Flow

```
DatabaseService (initialized via lifecycle hooks)
        ↓
   UserService (injected with DatabaseService)
        ↓
   UserController (injected with UserService)
```

## Database Operations

The example demonstrates:

### Create
```rust
let user = user_service.create_user(CreateUserRequest {
    username: "alice".to_string(),
    email: "alice@example.com".to_string(),
}).await?;
```

### Read (List)
```rust
let users = user_service.list_users().await?;
```

### Read (Single)
```rust
let user = user_service.get_user(1).await?;
```

### Update
```rust
let updated = user_service.update_user(1, UpdateUserRequest {
    username: Some("alice_updated".to_string()),
    email: None,
}).await?;
```

### Delete
```rust
let deleted = user_service.delete_user(1).await?;
```

### Stats
```rust
let stats = user_service.stats().await?;
```

## Configuration Options

### Connection Pool Configuration

```rust
let pool = PgPoolOptions::new()
    .max_connections(5)           // Maximum connections
    .min_connections(1)            // Minimum connections
    .acquire_timeout(Duration::from_secs(5))  // Acquisition timeout
    .idle_timeout(Duration::from_secs(600))   // Idle timeout
    .connect(&connection_string)
    .await?;
```

### Environment Variables

```bash
# Required
DATABASE_URL="postgres://user:pass@host:port/db"

# Optional (for production)
DATABASE_MAX_CONNECTIONS=20
DATABASE_MIN_CONNECTIONS=5
DATABASE_TIMEOUT=30
```

## Production Considerations

### 1. Connection Pool Size

```rust
let max_connections = std::env::var("DATABASE_MAX_CONNECTIONS")
    .unwrap_or_else(|_| "20".to_string())
    .parse()
    .unwrap_or(20);

let pool = PgPoolOptions::new()
    .max_connections(max_connections)
    .connect(&connection_string)
    .await?;
```

### 2. Health Checks

```rust
pub async fn health_check(&self) -> Result<bool, sqlx::Error> {
    let pool = self.pool().await?;
    sqlx::query("SELECT 1")
        .fetch_one(&pool)
        .await?;
    Ok(true)
}
```

### 3. Migrations

For production, use SQLx migrations:

```bash
# Create migrations directory
mkdir -p migrations

# Create migration
sqlx migrate add create_users_table

# Edit migrations/TIMESTAMP_create_users_table.sql
# Then run:
sqlx migrate run
```

### 4. Error Handling

```rust
match user_service.create_user(req).await {
    Ok(user) => Ok(HttpResponse::created().with_json(&user)?),
    Err(e) if e.contains("duplicate") => {
        Ok(HttpResponse::conflict()
            .with_json(&json!({"error": "User already exists"}))?
        )
    }
    Err(e) => Ok(HttpResponse::internal_server_error()
        .with_json(&json!({"error": e}))?
    ),
}
```

### 5. Transactions

```rust
pub async fn transfer_ownership(
    &self,
    from_id: i32,
    to_id: i32,
    resource_id: i32,
) -> Result<(), sqlx::Error> {
    let pool = self.pool().await?;

    let mut tx = pool.begin().await?;

    // Update resource owner
    sqlx::query("UPDATE resources SET owner_id = $1 WHERE id = $2")
        .bind(to_id)
        .bind(resource_id)
        .execute(&mut *tx)
        .await?;

    // Log transfer
    sqlx::query("INSERT INTO transfers (from_id, to_id, resource_id) VALUES ($1, $2, $3)")
        .bind(from_id)
        .bind(to_id)
        .bind(resource_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(())
}
```

## Troubleshooting

### "connection refused"

**Problem:** PostgreSQL is not running

**Solution:**
```bash
# Check if PostgreSQL is running
docker ps | grep postgres

# Start PostgreSQL
docker start postgres-armature
```

### "database does not exist"

**Problem:** Database hasn't been created

**Solution:**
```bash
# Connect to PostgreSQL
docker exec -it postgres-armature psql -U postgres

# Create database
CREATE DATABASE armature_demo;
\q
```

### "authentication failed"

**Problem:** Wrong credentials in DATABASE_URL

**Solution:**
```bash
# Check your connection string
echo $DATABASE_URL

# Reset password
docker exec -it postgres-armature psql -U postgres -c "ALTER USER postgres PASSWORD 'newpassword';"

# Update DATABASE_URL
export DATABASE_URL="postgres://postgres:newpassword@localhost:5432/armature_demo"
```

### "too many connections"

**Problem:** Connection pool exhausted

**Solution:**
```rust
// Reduce max_connections or increase PostgreSQL max_connections
let pool = PgPoolOptions::new()
    .max_connections(10)  // Lower value
    .connect(&connection_string)
    .await?;
```

## Further Reading

- [SQLx Documentation](https://docs.rs/sqlx)
- [SQLx GitHub](https://github.com/launchbadge/sqlx)
- [PostgreSQL Documentation](https://www.postgresql.org/docs/)
- [Armature Lifecycle Hooks](../docs/LIFECYCLE_HOOKS.md)
- [Armature DI Guide](../docs/DI_GUIDE.md)

## Other Database Examples

This example uses PostgreSQL, but SQLx supports multiple databases:

### MySQL
```toml
sqlx = { version = "0.7", features = ["mysql", "runtime-tokio-native-tls"] }
```

```bash
export DATABASE_URL="mysql://user:pass@localhost:3306/armature_demo"
```

### SQLite
```toml
sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio-native-tls"] }
```

```bash
export DATABASE_URL="sqlite://armature_demo.db"
```

### MariaDB
```toml
sqlx = { version = "0.7", features = ["mysql", "runtime-tokio-native-tls"] }
```

```bash
export DATABASE_URL="mysql://user:pass@localhost:3306/armature_demo"
```

## License

This example is part of the Armature project and follows the same MIT license.

