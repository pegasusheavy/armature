//! # Armature GraphQL Client
//!
//! A type-safe GraphQL client with support for queries, mutations, and subscriptions.
//!
//! ## Features
//!
//! - **Type-safe queries**: Generate Rust types from GraphQL schema
//! - **Subscriptions**: WebSocket-based GraphQL subscriptions
//! - **Batching**: Batch multiple queries into a single request
//! - **Caching**: Optional response caching
//! - **Federation**: Support for Apollo Federation
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use armature_graphql_client::{GraphQLClient, GraphQLClientConfig};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Serialize)]
//! struct GetUserVars {
//!     id: String,
//! }
//!
//! #[derive(Deserialize)]
//! struct User {
//!     id: String,
//!     name: String,
//! }
//!
//! #[derive(Deserialize)]
//! struct GetUserResponse {
//!     user: User,
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = GraphQLClient::new("https://api.example.com/graphql");
//!
//!     let response: GetUserResponse = client
//!         .query("query GetUser($id: ID!) { user(id: $id) { id name } }")
//!         .variables(GetUserVars { id: "123".into() })
//!         .send()
//!         .await?;
//!
//!     println!("User: {}", response.user.name);
//!     Ok(())
//! }
//! ```
//!
//! ## Subscriptions
//!
//! ```rust,ignore
//! use armature_graphql_client::GraphQLClient;
//! use futures::StreamExt;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = GraphQLClient::new("wss://api.example.com/graphql");
//!
//!     let mut subscription = client
//!         .subscribe("subscription { messageAdded { id content } }")
//!         .send()
//!         .await?;
//!
//!     while let Some(result) = subscription.next().await {
//!         match result {
//!             Ok(data) => println!("Received: {:?}", data),
//!             Err(e) => eprintln!("Error: {}", e),
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```

mod batch;
mod client;
mod config;
mod error;
mod request;
mod response;
mod subscription;

pub use batch::{BatchRequest, BatchResponse};
pub use client::GraphQLClient;
pub use config::{GraphQLClientConfig, GraphQLClientConfigBuilder};
pub use error::{GraphQLError, Result};
pub use request::{MutationBuilder, QueryBuilder, SubscriptionBuilder};
pub use response::{GraphQLResponse, GraphQLResponseError};
pub use subscription::{Subscription, SubscriptionStream};

// Re-export common types
pub use serde_json::Value as JsonValue;
