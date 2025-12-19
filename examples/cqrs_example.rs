#![allow(clippy::needless_question_mark)]
//! CQRS Example
//!
//! Demonstrates Command Query Responsibility Segregation with commands,
//! queries, and projections.

use armature_cqrs::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

// ============================================================================
// Domain Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: String,
    email: String,
    name: String,
    created_at: String,
}

// ============================================================================
// Commands
// ============================================================================

struct CreateUserCommand {
    email: String,
    name: String,
}

impl Command for CreateUserCommand {
    type Result = String; // Returns user ID
}

struct UpdateUserCommand {
    user_id: String,
    name: String,
}

impl Command for UpdateUserCommand {
    type Result = ();
}

struct DeleteUserCommand {
    user_id: String,
}

impl Command for DeleteUserCommand {
    type Result = ();
}

// ============================================================================
// Command Handlers
// ============================================================================

struct CreateUserHandler {
    users: Arc<RwLock<HashMap<String, User>>>,
}

#[async_trait]
impl CommandHandler<CreateUserCommand> for CreateUserHandler {
    async fn handle(&self, command: CreateUserCommand) -> Result<String, CommandError> {
        // Validate
        if command.email.is_empty() {
            return Err(CommandError::ValidationError(
                "Email is required".to_string(),
            ));
        }

        // Create user
        let user_id = Uuid::new_v4().to_string();
        let user = User {
            id: user_id.clone(),
            email: command.email.clone(),
            name: command.name,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        // Store (write model)
        self.users.write().await.insert(user_id.clone(), user);

        println!("✅ User created: {}", user_id);
        Ok(user_id)
    }
}

struct UpdateUserHandler {
    users: Arc<RwLock<HashMap<String, User>>>,
}

#[async_trait]
impl CommandHandler<UpdateUserCommand> for UpdateUserHandler {
    async fn handle(&self, command: UpdateUserCommand) -> Result<(), CommandError> {
        let mut users = self.users.write().await;

        let user = users
            .get_mut(&command.user_id)
            .ok_or_else(|| CommandError::ExecutionFailed("User not found".to_string()))?;

        user.name = command.name;

        println!("✅ User updated: {}", command.user_id);
        Ok(())
    }
}

struct DeleteUserHandler {
    users: Arc<RwLock<HashMap<String, User>>>,
}

#[async_trait]
impl CommandHandler<DeleteUserCommand> for DeleteUserHandler {
    async fn handle(&self, command: DeleteUserCommand) -> Result<(), CommandError> {
        let mut users = self.users.write().await;

        users
            .remove(&command.user_id)
            .ok_or_else(|| CommandError::ExecutionFailed("User not found".to_string()))?;

        println!("✅ User deleted: {}", command.user_id);
        Ok(())
    }
}

// ============================================================================
// Queries
// ============================================================================

struct GetUserQuery {
    user_id: String,
}

impl Query for GetUserQuery {
    type Result = User;
}

struct ListUsersQuery;

impl Query for ListUsersQuery {
    type Result = Vec<User>;
}

struct SearchUsersQuery {
    email_contains: String,
}

impl Query for SearchUsersQuery {
    type Result = Vec<User>;
}

// ============================================================================
// Query Handlers (Read from Read Model)
// ============================================================================

struct GetUserHandler {
    users: Arc<RwLock<HashMap<String, User>>>,
}

#[async_trait]
impl QueryHandler<GetUserQuery> for GetUserHandler {
    async fn handle(&self, query: GetUserQuery) -> Result<User, QueryError> {
        let users = self.users.read().await;

        users
            .get(&query.user_id)
            .cloned()
            .ok_or_else(|| QueryError::NotFound(format!("User {} not found", query.user_id)))
    }
}

struct ListUsersHandler {
    users: Arc<RwLock<HashMap<String, User>>>,
}

#[async_trait]
impl QueryHandler<ListUsersQuery> for ListUsersHandler {
    async fn handle(&self, _query: ListUsersQuery) -> Result<Vec<User>, QueryError> {
        let users = self.users.read().await;
        Ok(users.values().cloned().collect())
    }
}

struct SearchUsersHandler {
    users: Arc<RwLock<HashMap<String, User>>>,
}

#[async_trait]
impl QueryHandler<SearchUsersQuery> for SearchUsersHandler {
    async fn handle(&self, query: SearchUsersQuery) -> Result<Vec<User>, QueryError> {
        let users = self.users.read().await;

        let filtered: Vec<User> = users
            .values()
            .filter(|u| u.email.contains(&query.email_contains))
            .cloned()
            .collect();

        Ok(filtered)
    }
}

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== CQRS Example ===\n");

    // Shared storage (in real app, write and read models would be separate)
    let users = Arc::new(RwLock::new(HashMap::new()));

    // 1. Create command bus
    println!("1. Creating Command Bus:");
    let command_bus = CommandBus::new();

    command_bus.register::<CreateUserCommand, _>(CreateUserHandler {
        users: users.clone(),
    });
    command_bus.register::<UpdateUserCommand, _>(UpdateUserHandler {
        users: users.clone(),
    });
    command_bus.register::<DeleteUserCommand, _>(DeleteUserHandler {
        users: users.clone(),
    });

    println!("   ✅ Registered 3 command handlers\n");

    // 2. Create query bus
    println!("2. Creating Query Bus:");
    let query_bus = QueryBus::new();

    query_bus.register::<GetUserQuery, _>(GetUserHandler {
        users: users.clone(),
    });
    query_bus.register::<ListUsersQuery, _>(ListUsersHandler {
        users: users.clone(),
    });
    query_bus.register::<SearchUsersQuery, _>(SearchUsersHandler {
        users: users.clone(),
    });

    println!("   ✅ Registered 3 query handlers\n");

    // 3. Execute commands
    println!("3. Executing Commands:");
    println!();

    // Create users
    println!("   Creating users...");
    let user1_id = command_bus
        .execute(CreateUserCommand {
            email: "alice@example.com".to_string(),
            name: "Alice".to_string(),
        })
        .await?;

    let user2_id = command_bus
        .execute(CreateUserCommand {
            email: "bob@example.com".to_string(),
            name: "Bob".to_string(),
        })
        .await?;

    let _user3_id = command_bus
        .execute(CreateUserCommand {
            email: "charlie@example.com".to_string(),
            name: "Charlie".to_string(),
        })
        .await?;

    println!();

    // Update user
    println!("   Updating user...");
    command_bus
        .execute(UpdateUserCommand {
            user_id: user1_id.clone(),
            name: "Alice Smith".to_string(),
        })
        .await?;

    println!();

    // 4. Execute queries
    println!("4. Executing Queries:");
    println!();

    // Get single user
    println!("   Get User by ID:");
    let user = query_bus
        .execute(GetUserQuery {
            user_id: user1_id.clone(),
        })
        .await?;
    println!("   ID: {}", user.id);
    println!("   Email: {}", user.email);
    println!("   Name: {}", user.name);
    println!("   Created: {}", user.created_at);
    println!();

    // List all users
    println!("   List All Users:");
    let all_users = query_bus.execute(ListUsersQuery).await?;
    println!("   Total users: {}", all_users.len());
    for user in &all_users {
        println!("     - {} ({})", user.name, user.email);
    }
    println!();

    // Search users
    println!("   Search Users (email contains 'example'):");
    let search_results = query_bus
        .execute(SearchUsersQuery {
            email_contains: "example".to_string(),
        })
        .await?;
    println!("   Found: {} users", search_results.len());
    for user in &search_results {
        println!("     - {} ({})", user.name, user.email);
    }
    println!();

    // 5. Delete user
    println!("5. Deleting User:");
    command_bus
        .execute(DeleteUserCommand {
            user_id: user2_id.clone(),
        })
        .await?;

    // Verify deletion
    let remaining_users = query_bus.execute(ListUsersQuery).await?;
    println!("   Remaining users: {}", remaining_users.len());
    for user in &remaining_users {
        println!("     - {} ({})", user.name, user.email);
    }
    println!();

    // 6. Error handling
    println!("6. Error Handling:");

    // Try to get deleted user
    println!("   Attempting to get deleted user...");
    match query_bus
        .execute(GetUserQuery {
            user_id: user2_id.clone(),
        })
        .await
    {
        Ok(_) => println!("   ❌ Unexpected: Query succeeded"),
        Err(e) => println!("   ✅ Expected error: {}", e),
    }
    println!();

    // Try to create user with empty email
    println!("   Attempting to create user with empty email...");
    match command_bus
        .execute(CreateUserCommand {
            email: String::new(),
            name: "Invalid".to_string(),
        })
        .await
    {
        Ok(_) => println!("   ❌ Unexpected: Command succeeded"),
        Err(e) => println!("   ✅ Expected error: {}", e),
    }
    println!();

    println!("=== CQRS Example Complete ===\n");
    println!("💡 Key Features Demonstrated:");
    println!("   ✅ Command/Query separation");
    println!("   ✅ Command bus for writes");
    println!("   ✅ Query bus for reads");
    println!("   ✅ Type-safe commands and queries");
    println!("   ✅ Business logic in handlers");
    println!("   ✅ Error handling");
    println!("   ✅ Validation");
    println!();
    println!("💡 In Production:");
    println!("   - Commands would update aggregates (event sourcing)");
    println!("   - Events would be published to event bus");
    println!("   - Projections would build read models");
    println!("   - Read/write models would use separate databases");
    println!();

    Ok(())
}
