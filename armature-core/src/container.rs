// Dependency injection container

use crate::{Error, Provider};
use crate::logging::{debug, trace};
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
        debug!("Creating new DI container");
        Self {
            providers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a provider instance
    pub fn register<T: Provider>(&self, instance: T) {
        let type_id = TypeId::of::<T>();
        let type_name = std::any::type_name::<T>();
        
        trace!(provider = type_name, "Acquiring write lock for registration");
        let mut providers = self.providers.write().unwrap();
        providers.insert(type_id, Arc::new(instance));
        
        debug!(provider = type_name, "Provider registered in DI container");
    }

    /// Register a provider from a boxed instance
    #[allow(clippy::boxed_local)]
    pub fn register_boxed<T: Provider>(&self, instance: Box<T>) {
        let type_id = TypeId::of::<T>();
        let type_name = std::any::type_name::<T>();
        
        trace!(provider = type_name, "Registering boxed provider");
        let mut providers = self.providers.write().unwrap();
        providers.insert(type_id, Arc::new(*instance));
        
        debug!(provider = type_name, "Boxed provider registered");
    }

    /// Register a provider by TypeId and Arc (internal use)
    pub fn register_by_id(&self, type_id: TypeId, instance: Arc<dyn std::any::Any + Send + Sync>) {
        trace!(type_id = ?type_id, "Registering provider by TypeId");
        let mut providers = self.providers.write().unwrap();
        providers.insert(type_id, instance);
        
        debug!(type_id = ?type_id, "Provider registered by TypeId");
    }

    /// Register a provider using a factory function
    pub fn register_factory<T: Provider, F>(&self, factory: F)
    where
        F: FnOnce() -> T,
    {
        let type_name = std::any::type_name::<T>();
        debug!(provider = type_name, "Creating provider from factory");
        
        let instance = factory();
        self.register(instance);
    }

    /// Resolve a provider by type
    pub fn resolve<T: Provider>(&self) -> Result<Arc<T>, Error> {
        let type_id = TypeId::of::<T>();
        let type_name = std::any::type_name::<T>();
        
        trace!(provider = type_name, "Attempting to resolve provider");
        let providers = self.providers.read().unwrap();

        let result = providers
            .get(&type_id)
            .and_then(|any| any.clone().downcast::<T>().ok())
            .ok_or_else(|| {
                Error::ProviderNotFound(format!(
                    "Provider not found: {}",
                    type_name
                ))
            });
        
        match &result {
            Ok(_) => debug!(provider = type_name, "Provider resolved successfully"),
            Err(_) => debug!(provider = type_name, "Provider not found in container"),
        }
        
        result
    }

    /// Check if a provider is registered
    pub fn has<T: Provider>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        let type_name = std::any::type_name::<T>();
        
        let providers = self.providers.read().unwrap();
        let exists = providers.contains_key(&type_id);
        
        trace!(provider = type_name, exists = exists, "Checked provider existence");
        exists
    }

    /// Clear all providers
    pub fn clear(&self) {
        let mut providers = self.providers.write().unwrap();
        let count = providers.len();
        providers.clear();
        
        debug!(provider_count = count, "Cleared all providers from container");
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}
