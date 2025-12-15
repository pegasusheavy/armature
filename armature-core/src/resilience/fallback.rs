//! Fallback pattern for graceful degradation.
//!
//! The fallback pattern provides alternative behavior when
//! primary operations fail.
//!
//! ## Example
//!
//! ```rust,ignore
//! use armature::resilience::{Fallback, FallbackConfig};
//!
//! let fallback = Fallback::new(|| async {
//!     // Return cached/default value
//!     Ok(CachedData::default())
//! });
//!
//! let result = fallback.call(
//!     || async { external_service.fetch().await },
//! ).await;
//! ```

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tracing::debug;

/// Type alias for an async fallback function.
pub type FallbackFn<T, E> = Arc<dyn Fn() -> Pin<Box<dyn Future<Output = Result<T, E>> + Send>> + Send + Sync>;

/// Fallback handler that provides alternative behavior.
pub struct Fallback<T, E> {
    fallback: FallbackFn<T, E>,
    name: String,
}

impl<T, E> Fallback<T, E>
where
    T: Send + 'static,
    E: Send + 'static,
{
    /// Create a new fallback handler.
    pub fn new<F, Fut>(fallback: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<T, E>> + Send + 'static,
    {
        Self {
            fallback: Arc::new(move || Box::pin(fallback())),
            name: "default".to_string(),
        }
    }

    /// Set the fallback name for logging.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Execute the primary operation, falling back on failure.
    pub async fn call<F, Fut>(&self, primary: F) -> Result<T, E>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>>,
    {
        match primary().await {
            Ok(result) => Ok(result),
            Err(_) => {
                debug!(name = %self.name, "Primary operation failed, using fallback");
                (self.fallback)().await
            }
        }
    }

    /// Execute with fallback only for specific error types.
    pub async fn call_if<F, Fut, P>(&self, primary: F, should_fallback: P) -> Result<T, E>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>>,
        P: FnOnce(&E) -> bool,
    {
        match primary().await {
            Ok(result) => Ok(result),
            Err(e) => {
                if should_fallback(&e) {
                    debug!(name = %self.name, "Primary operation failed, using fallback");
                    (self.fallback)().await
                } else {
                    Err(e)
                }
            }
        }
    }
}

impl<T, E> Clone for Fallback<T, E> {
    fn clone(&self) -> Self {
        Self {
            fallback: Arc::clone(&self.fallback),
            name: self.name.clone(),
        }
    }
}

/// Create a fallback that returns a constant value.
pub fn fallback_value<T, E>(value: T) -> Fallback<T, E>
where
    T: Clone + Send + Sync + 'static,
    E: Send + 'static,
{
    Fallback::new(move || {
        let v = value.clone();
        async move { Ok(v) }
    })
}

/// Create a fallback that returns a default value.
pub fn fallback_default<T, E>() -> Fallback<T, E>
where
    T: Default + Send + 'static,
    E: Send + 'static,
{
    Fallback::new(|| async { Ok(T::default()) })
}

/// Execute with fallback using a builder pattern.
pub struct FallbackBuilder<T, E> {
    name: String,
    _marker: std::marker::PhantomData<(T, E)>,
}

impl<T, E> FallbackBuilder<T, E>
where
    T: Send + 'static,
    E: Send + 'static,
{
    /// Create a new builder.
    pub fn new() -> Self {
        Self {
            name: "default".to_string(),
            _marker: std::marker::PhantomData,
        }
    }

    /// Set the name for logging.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Build with a fallback function.
    pub fn fallback<F, Fut>(self, f: F) -> Fallback<T, E>
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<T, E>> + Send + 'static,
    {
        Fallback::new(f).with_name(self.name)
    }

    /// Build with a constant fallback value.
    pub fn value(self, value: T) -> Fallback<T, E>
    where
        T: Clone + Send + Sync + 'static,
    {
        fallback_value(value).with_name(self.name)
    }

    /// Build with a default fallback value.
    pub fn default_value(self) -> Fallback<T, E>
    where
        T: Default,
    {
        fallback_default().with_name(self.name)
    }
}

impl<T, E> Default for FallbackBuilder<T, E>
where
    T: Send + 'static,
    E: Send + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

/// Combined resilience pattern that tries multiple strategies.
pub struct FallbackChain<T, E> {
    handlers: Vec<FallbackFn<T, E>>,
}

impl<T, E> FallbackChain<T, E>
where
    T: Send + 'static,
    E: std::fmt::Display + Send + 'static,
{
    /// Create a new fallback chain.
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    /// Add a handler to the chain.
    pub fn with_handler<F, Fut>(mut self, handler: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<T, E>> + Send + 'static,
    {
        self.handlers.push(Arc::new(move || Box::pin(handler())));
        self
    }

    /// Execute the chain, trying each handler until one succeeds.
    pub async fn call(&self) -> Result<T, E> {
        let mut last_error: Option<E> = None;

        for (i, handler) in self.handlers.iter().enumerate() {
            match handler().await {
                Ok(result) => {
                    if i > 0 {
                        debug!(handler = i, "Fallback chain succeeded on handler");
                    }
                    return Ok(result);
                }
                Err(e) => {
                    debug!(handler = i, error = %e, "Fallback chain handler failed");
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.expect("Fallback chain must have at least one handler"))
    }
}

impl<T, E> Default for FallbackChain<T, E>
where
    T: Send + 'static,
    E: std::fmt::Display + Send + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fallback_success() {
        let fallback = fallback_value::<i32, &str>(0);

        let result = fallback.call(|| async {
            Ok::<i32, &str>(42)
        }).await;

        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_fallback_on_failure() {
        let fallback = fallback_value::<i32, &str>(99);

        let result = fallback.call(|| async {
            Err::<i32, &str>("error")
        }).await;

        assert_eq!(result.unwrap(), 99);
    }

    #[tokio::test]
    async fn test_fallback_chain() {
        let chain = FallbackChain::<i32, &str>::new()
            .with_handler(|| async { Err("first fails") })
            .with_handler(|| async { Err("second fails") })
            .with_handler(|| async { Ok(42) });

        let result = chain.call().await;
        assert_eq!(result.unwrap(), 42);
    }
}

