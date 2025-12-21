//! # Armature SeaORM
//!
//! SeaORM database integration for the Armature framework.
//!
//! This crate provides seamless integration with SeaORM, a full-featured
//! async ORM for Rust with active record pattern support.
//!
//! ## Features
//!
//! - **Multiple Backends**: PostgreSQL, MySQL, and SQLite support
//! - **Connection Pooling**: Built-in connection pooling via SQLx
//! - **Transaction Management**: Easy-to-use transaction helpers
//! - **DI Integration**: Works with Armature's dependency injection
//! - **Migration Support**: Integrated migration system
//! - **Active Record**: Entity-based CRUD operations
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use armature_seaorm::{Database, DatabaseConfig};
//!
//! // Create configuration
//! let config = DatabaseConfig::new("postgres://user:pass@localhost/db")
//!     .max_connections(10)
//!     .connect_timeout(Duration::from_secs(5));
//!
//! // Connect to database
//! let db = Database::connect(config).await?;
//!
//! // Query entities
//! let users = User::find().all(&db).await?;
//! ```
//!
//! ## With Transactions
//!
//! ```rust,ignore
//! use armature_seaorm::TransactionExt;
//!
//! db.transaction(|txn| async move {
//!     let user = user::ActiveModel {
//!         name: Set("Alice".to_owned()),
//!         ..Default::default()
//!     };
//!     user.insert(&txn).await?;
//!     Ok(())
//! }).await?;
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

mod config;
mod database;
mod error;
mod pagination;
mod query;
mod transaction;

pub use config::*;
pub use database::*;
pub use error::*;
pub use pagination::*;
pub use query::*;
pub use transaction::*;

// Re-export sea-orm types for convenience
pub use sea_orm;
pub use sea_query;

/// Prelude module for commonly used types.
pub mod prelude {
    pub use super::{Database, DatabaseConfig, SeaOrmError, SeaOrmResult};
    pub use super::{Paginate, Paginated, PaginationOptions};
    pub use super::{QueryBuilder, QueryExt};
    pub use super::TransactionExt;
    pub use sea_orm::entity::prelude::*;
    pub use sea_orm::{
        ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, ModelTrait,
        PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set,
    };
}

