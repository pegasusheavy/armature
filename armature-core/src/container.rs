// Dependency injection container

use crate::{Error, Provider};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// The dependency injection container
#[derive(Clone)]
pub struct Container {
    providers: Arc<RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>>,
}

impl Container {
    pub fn new() -> Self {
        Self {
            providers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a provider instance
    pub fn register<T: Provider>(&self, instance: T) {
        let type_id = TypeId::of::<T>();
        let mut providers = self.providers.write().unwrap();
        providers.insert(type_id, Arc::new(instance));
    }

    /// Register a provider from a boxed instance
    #[allow(clippy::boxed_local)]
    pub fn register_boxed<T: Provider>(&self, instance: Box<T>) {
        let type_id = TypeId::of::<T>();
        let mut providers = self.providers.write().unwrap();
        providers.insert(type_id, Arc::new(*instance));
    }

    /// Register a provider by TypeId and Arc (internal use)
    pub fn register_by_id(&self, type_id: TypeId, instance: Arc<dyn std::any::Any + Send + Sync>) {
        let mut providers = self.providers.write().unwrap();
        providers.insert(type_id, instance);
    }

    /// Register a provider using a factory function
    pub fn register_factory<T: Provider, F>(&self, factory: F)
    where
        F: FnOnce() -> T,
    {
        let instance = factory();
        self.register(instance);
    }

    /// Resolve a provider by type
    pub fn resolve<T: Provider>(&self) -> Result<Arc<T>, Error> {
        let type_id = TypeId::of::<T>();
        let providers = self.providers.read().unwrap();

        providers
            .get(&type_id)
            .and_then(|any| any.clone().downcast::<T>().ok())
            .ok_or_else(|| {
                Error::ProviderNotFound(format!(
                    "Provider not found: {}",
                    std::any::type_name::<T>()
                ))
            })
    }

    /// Check if a provider is registered
    pub fn has<T: Provider>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        let providers = self.providers.read().unwrap();
        providers.contains_key(&type_id)
    }

    /// Clear all providers
    pub fn clear(&self) {
        let mut providers = self.providers.write().unwrap();
        providers.clear();
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}
