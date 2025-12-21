//! Transaction management for SeaORM.

use crate::{Database, SeaOrmError, SeaOrmResult};
use async_trait::async_trait;
use sea_orm::{DatabaseConnection, DatabaseTransaction, TransactionTrait};
use std::future::Future;

/// Extension trait for transaction management.
#[async_trait]
pub trait TransactionExt {
    /// Execute a closure within a transaction.
    ///
    /// If the closure returns an error, the transaction is rolled back.
    /// Otherwise, the transaction is committed.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// db.transaction(|txn| async move {
    ///     let user = user::ActiveModel {
    ///         name: Set("Alice".to_owned()),
    ///         ..Default::default()
    ///     };
    ///     user.insert(&txn).await?;
    ///     Ok(())
    /// }).await?;
    /// ```
    async fn transaction<F, Fut, T, E>(&self, f: F) -> Result<T, E>
    where
        F: FnOnce(DatabaseTransaction) -> Fut + Send,
        Fut: Future<Output = Result<T, E>> + Send,
        T: Send,
        E: From<sea_orm::DbErr> + Send;

    /// Execute a closure within a transaction with custom isolation level.
    async fn transaction_with_isolation<F, Fut, T, E>(
        &self,
        isolation: IsolationLevel,
        f: F,
    ) -> Result<T, E>
    where
        F: FnOnce(DatabaseTransaction) -> Fut + Send,
        Fut: Future<Output = Result<T, E>> + Send,
        T: Send,
        E: From<sea_orm::DbErr> + Send;
}

/// Transaction isolation levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsolationLevel {
    /// Read uncommitted - lowest isolation.
    ReadUncommitted,
    /// Read committed.
    ReadCommitted,
    /// Repeatable read.
    RepeatableRead,
    /// Serializable - highest isolation.
    Serializable,
}

impl From<IsolationLevel> for Option<sea_orm::IsolationLevel> {
    fn from(level: IsolationLevel) -> Self {
        Some(match level {
            IsolationLevel::ReadUncommitted => sea_orm::IsolationLevel::ReadUncommitted,
            IsolationLevel::ReadCommitted => sea_orm::IsolationLevel::ReadCommitted,
            IsolationLevel::RepeatableRead => sea_orm::IsolationLevel::RepeatableRead,
            IsolationLevel::Serializable => sea_orm::IsolationLevel::Serializable,
        })
    }
}

#[async_trait]
impl TransactionExt for Database {
    async fn transaction<F, Fut, T, E>(&self, f: F) -> Result<T, E>
    where
        F: FnOnce(DatabaseTransaction) -> Fut + Send,
        Fut: Future<Output = Result<T, E>> + Send,
        T: Send,
        E: From<sea_orm::DbErr> + Send,
    {
        armature_log::debug!("Starting database transaction");
        self.connection().transaction(f).await
    }

    async fn transaction_with_isolation<F, Fut, T, E>(
        &self,
        isolation: IsolationLevel,
        f: F,
    ) -> Result<T, E>
    where
        F: FnOnce(DatabaseTransaction) -> Fut + Send,
        Fut: Future<Output = Result<T, E>> + Send,
        T: Send,
        E: From<sea_orm::DbErr> + Send,
    {
        armature_log::debug!("Starting database transaction with isolation level {:?}", isolation);
        self.connection()
            .transaction_with_config(f, isolation.into(), None)
            .await
    }
}

#[async_trait]
impl TransactionExt for DatabaseConnection {
    async fn transaction<F, Fut, T, E>(&self, f: F) -> Result<T, E>
    where
        F: FnOnce(DatabaseTransaction) -> Fut + Send,
        Fut: Future<Output = Result<T, E>> + Send,
        T: Send,
        E: From<sea_orm::DbErr> + Send,
    {
        TransactionTrait::transaction(self, f).await
    }

    async fn transaction_with_isolation<F, Fut, T, E>(
        &self,
        isolation: IsolationLevel,
        f: F,
    ) -> Result<T, E>
    where
        F: FnOnce(DatabaseTransaction) -> Fut + Send,
        Fut: Future<Output = Result<T, E>> + Send,
        T: Send,
        E: From<sea_orm::DbErr> + Send,
    {
        TransactionTrait::transaction_with_config(self, f, isolation.into(), None).await
    }
}

/// Transaction options for advanced control.
#[derive(Debug, Clone)]
pub struct TransactionOptions {
    /// Isolation level.
    pub isolation: Option<IsolationLevel>,
    /// Read-only transaction.
    pub read_only: bool,
    /// Deferrable (PostgreSQL only).
    pub deferrable: bool,
}

impl Default for TransactionOptions {
    fn default() -> Self {
        Self {
            isolation: None,
            read_only: false,
            deferrable: false,
        }
    }
}

impl TransactionOptions {
    /// Create new transaction options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set isolation level.
    pub fn isolation(mut self, level: IsolationLevel) -> Self {
        self.isolation = Some(level);
        self
    }

    /// Set read-only mode.
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    /// Set deferrable (PostgreSQL only).
    pub fn deferrable(mut self, deferrable: bool) -> Self {
        self.deferrable = deferrable;
        self
    }
}

