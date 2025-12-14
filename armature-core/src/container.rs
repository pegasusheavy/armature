//! Dependency injection container
//!
//! This module re-exports the DI container from `dependency-injector` and provides
//! framework-specific integration.

use crate::Error;
use std::any::{Any, TypeId};
use std::sync::Arc;

// Re-export the core DI types (excluding ProviderRegistration to avoid conflict with traits.rs)
pub use dependency_injector::{
    Container as DiContainer, DiError, Factory, Injectable, Lifetime, Provider, Scope,
    ScopeBuilder, ScopedContainer as DiScopedContainer,
};

/// The dependency injection container for Armature.
///
/// This is a thin wrapper around `dependency_injector::Container` that provides
/// error conversion to the framework's error type.
#[derive(Clone, Default)]
pub struct Container {
    inner: DiContainer,
}

impl Container {
    /// Create a new empty container.
    #[inline]
    pub fn new() -> Self {
        Self {
            inner: DiContainer::new(),
        }
    }

    /// Create with pre-allocated capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: DiContainer::with_capacity(capacity),
        }
    }

    /// Create a scoped child container.
    #[inline]
    pub fn create_scope(&self) -> Self {
        Self {
            inner: self.inner.scope(),
        }
    }

    /// Alias for create_scope.
    #[inline]
    pub fn scope(&self) -> Self {
        self.create_scope()
    }

    /// Register a singleton service.
    #[inline]
    pub fn register<T: Injectable>(&self, instance: T) {
        self.inner.singleton(instance);
    }

    /// Register a singleton service (explicit).
    #[inline]
    pub fn singleton<T: Injectable>(&self, instance: T) {
        self.inner.singleton(instance);
    }

    /// Register a lazy singleton.
    #[inline]
    pub fn lazy<T: Injectable, F>(&self, factory: F)
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        self.inner.lazy(factory);
    }

    /// Register a transient service.
    #[inline]
    pub fn transient<T: Injectable, F>(&self, factory: F)
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        self.inner.transient(factory);
    }

    /// Register a boxed service instance.
    #[inline]
    pub fn register_boxed<T: Injectable>(&self, instance: Box<T>) {
        self.inner.register_boxed(instance);
    }

    /// Register by TypeId directly.
    #[inline]
    pub fn register_by_id(&self, type_id: TypeId, instance: Arc<dyn Any + Send + Sync>) {
        self.inner.register_by_id(type_id, instance);
    }

    /// Register using a factory function.
    #[inline]
    pub fn register_factory<T: Injectable, F>(&self, factory: F)
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        self.inner.lazy(factory);
    }

    /// Resolve a service by type.
    #[inline]
    pub fn resolve<T: Injectable>(&self) -> Result<Arc<T>, Error> {
        self.inner
            .get::<T>()
            .map_err(|e| Error::ProviderNotFound(e.to_string()))
    }

    /// Alias for resolve.
    #[inline]
    pub fn get<T: Injectable>(&self) -> Result<Arc<T>, Error> {
        self.resolve::<T>()
    }

    /// Try to resolve, returning None if not found.
    #[inline]
    pub fn try_resolve<T: Injectable>(&self) -> Option<Arc<T>> {
        self.inner.try_get()
    }

    /// Try to get a service.
    #[inline]
    pub fn try_get<T: Injectable>(&self) -> Option<Arc<T>> {
        self.try_resolve::<T>()
    }

    /// Check if a service is registered.
    #[inline]
    pub fn has<T: Injectable>(&self) -> bool {
        self.inner.contains::<T>()
    }

    /// Alias for has.
    #[inline]
    pub fn contains<T: Injectable>(&self) -> bool {
        self.has::<T>()
    }

    /// Clear all services.
    #[inline]
    pub fn clear(&self) {
        self.inner.clear();
    }

    /// Get the number of registered services.
    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if the container is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Lock the container.
    #[inline]
    pub fn lock(&self) {
        self.inner.lock();
    }

    /// Check if the container is locked.
    #[inline]
    pub fn is_locked(&self) -> bool {
        self.inner.is_locked()
    }

    /// Get all registered type IDs.
    #[inline]
    pub fn registered_types(&self) -> Vec<TypeId> {
        self.inner.registered_types()
    }

    /// Get the scope depth.
    #[inline]
    pub fn depth(&self) -> u32 {
        self.inner.depth()
    }

    /// Get the inner DI container.
    #[inline]
    pub fn inner(&self) -> &DiContainer {
        &self.inner
    }
}

impl std::fmt::Debug for Container {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Container")
            .field("inner", &self.inner)
            .finish()
    }
}

impl From<DiContainer> for Container {
    fn from(inner: DiContainer) -> Self {
        Self { inner }
    }
}

impl AsRef<DiContainer> for Container {
    fn as_ref(&self) -> &DiContainer {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct TestService {
        value: String,
    }

    #[test]
    fn test_container_creation() {
        let container = Container::new();
        assert!(container.is_empty());
    }

    #[test]
    fn test_register_and_resolve() {
        let container = Container::new();
        container.register(TestService {
            value: "test".to_string(),
        });

        let resolved = container.resolve::<TestService>().unwrap();
        assert_eq!(resolved.value, "test");
    }

    #[test]
    fn test_scoped_container() {
        let parent = Container::new();
        parent.register(TestService {
            value: "parent".to_string(),
        });

        let child = parent.create_scope();

        // Child can resolve from parent
        assert!(child.has::<TestService>());
        let resolved = child.resolve::<TestService>().unwrap();
        assert_eq!(resolved.value, "parent");
    }

    #[test]
    fn test_lazy_singleton() {
        use std::sync::atomic::{AtomicBool, Ordering};

        static CREATED: AtomicBool = AtomicBool::new(false);

        #[derive(Clone)]
        struct LazyService;

        let container = Container::new();
        container.lazy(|| {
            CREATED.store(true, Ordering::SeqCst);
            LazyService
        });

        assert!(!CREATED.load(Ordering::SeqCst));

        let _ = container.get::<LazyService>().unwrap();
        assert!(CREATED.load(Ordering::SeqCst));
    }

    #[test]
    fn test_transient() {
        use std::sync::atomic::{AtomicU32, Ordering};

        static COUNTER: AtomicU32 = AtomicU32::new(0);

        #[derive(Clone)]
        struct Counter(u32);

        let container = Container::new();
        container.transient(|| Counter(COUNTER.fetch_add(1, Ordering::SeqCst)));

        let c1 = container.get::<Counter>().unwrap();
        let c2 = container.get::<Counter>().unwrap();

        assert_ne!(c1.0, c2.0);
    }
}
