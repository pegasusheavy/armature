// GraphQL resolver traits and utilities

use async_graphql::{Context, Result};
use std::any::Any;

/// Trait for GraphQL resolvers with DI support
pub trait Resolver: Send + Sync {
    fn as_any(&self) -> &dyn Any;
}

/// Trait for query resolvers
pub trait QueryResolver: Resolver {
    fn type_name() -> &'static str
    where
        Self: Sized;
}

/// Trait for mutation resolvers
pub trait MutationResolver: Resolver {
    fn type_name() -> &'static str
    where
        Self: Sized;
}

/// Trait for subscription resolvers
pub trait SubscriptionResolver: Resolver {
    fn type_name() -> &'static str
    where
        Self: Sized;
}

/// Context extension for accessing DI services
pub trait ContextExt {
    /// Get a service from the context
    fn get_service<T: Send + Sync + 'static>(&self) -> Result<&T>;
}

impl<'a> ContextExt for Context<'a> {
    fn get_service<T: Send + Sync + 'static>(&self) -> Result<&T> {
        self.data::<T>()
    }
}
