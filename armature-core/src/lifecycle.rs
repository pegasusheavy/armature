//! Lifecycle hook system for Armature framework.
//!
//! Provides lifecycle hooks for modules, controllers, and services similar to NestJS.
//!
//! ## Available Hooks
//!
//! - `OnModuleInit` - Called after module initialization
//! - `OnModuleDestroy` - Called before module destruction
//! - `OnApplicationBootstrap` - Called after full application bootstrap
//! - `OnApplicationShutdown` - Called during graceful shutdown
//!
//! ## Examples
//!
//! ```
//! use armature_core::lifecycle::{OnModuleInit, OnModuleDestroy};
//! use armature_core::Provider;
//! use async_trait::async_trait;
//!
//! struct MyService {
//!     name: String,
//! }
//!
//! impl Provider for MyService {}
//!
//! #[async_trait]
//! impl OnModuleInit for MyService {
//!     async fn on_module_init(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//!         println!("Service {} initialized!", self.name);
//!         Ok(())
//!     }
//! }
//!
//! #[async_trait]
//! impl OnModuleDestroy for MyService {
//!     async fn on_module_destroy(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//!         println!("Service {} destroyed!", self.name);
//!         Ok(())
//!     }
//! }
//! ```

use async_trait::async_trait;
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Error type for lifecycle operations
pub type LifecycleResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

/// Hook called after module dependencies are resolved
#[async_trait]
pub trait OnModuleInit: Send + Sync {
    /// Called once the module has been initialized
    async fn on_module_init(&self) -> LifecycleResult;
}

/// Hook called before module is destroyed
#[async_trait]
pub trait OnModuleDestroy: Send + Sync {
    /// Called before the module is destroyed
    async fn on_module_destroy(&self) -> LifecycleResult;
}

/// Hook called after all modules have been initialized
#[async_trait]
pub trait OnApplicationBootstrap: Send + Sync {
    /// Called once the application has fully started
    async fn on_application_bootstrap(&self) -> LifecycleResult;
}

/// Hook called during application shutdown
#[async_trait]
pub trait OnApplicationShutdown: Send + Sync {
    /// Called when the application is shutting down
    async fn on_application_shutdown(&self, signal: Option<String>) -> LifecycleResult;
}

/// Hook called before module initialization
#[async_trait]
pub trait BeforeApplicationShutdown: Send + Sync {
    /// Called before application shutdown hooks
    async fn before_application_shutdown(&self, signal: Option<String>) -> LifecycleResult;
}

/// Manages lifecycle hooks for all registered components
pub struct LifecycleManager {
    init_hooks: Arc<RwLock<Vec<(String, Arc<dyn OnModuleInit>)>>>,
    destroy_hooks: Arc<RwLock<Vec<(String, Arc<dyn OnModuleDestroy>)>>>,
    bootstrap_hooks: Arc<RwLock<Vec<(String, Arc<dyn OnApplicationBootstrap>)>>>,
    shutdown_hooks: Arc<RwLock<Vec<(String, Arc<dyn OnApplicationShutdown>)>>>,
    before_shutdown_hooks: Arc<RwLock<Vec<(String, Arc<dyn BeforeApplicationShutdown>)>>>,
    hook_registry: Arc<RwLock<HashMap<TypeId, String>>>,
}

impl LifecycleManager {
    /// Create a new lifecycle manager
    pub fn new() -> Self {
        Self {
            init_hooks: Arc::new(RwLock::new(Vec::new())),
            destroy_hooks: Arc::new(RwLock::new(Vec::new())),
            bootstrap_hooks: Arc::new(RwLock::new(Vec::new())),
            shutdown_hooks: Arc::new(RwLock::new(Vec::new())),
            before_shutdown_hooks: Arc::new(RwLock::new(Vec::new())),
            hook_registry: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a type name for tracking purposes
    pub async fn register_type(&self, type_id: TypeId, name: String) {
        let mut registry = self.hook_registry.write().await;
        registry.insert(type_id, name);
    }

    /// Get the registered name for a type
    pub async fn get_type_name(&self, type_id: TypeId) -> Option<String> {
        let registry = self.hook_registry.read().await;
        registry.get(&type_id).cloned()
    }

    /// Register an OnModuleInit hook
    pub async fn register_on_init(&self, name: String, hook: Arc<dyn OnModuleInit>) {
        let mut hooks = self.init_hooks.write().await;
        hooks.push((name, hook));
    }

    /// Register an OnModuleDestroy hook
    pub async fn register_on_destroy(&self, name: String, hook: Arc<dyn OnModuleDestroy>) {
        let mut hooks = self.destroy_hooks.write().await;
        hooks.push((name, hook));
    }

    /// Register an OnApplicationBootstrap hook
    pub async fn register_on_bootstrap(&self, name: String, hook: Arc<dyn OnApplicationBootstrap>) {
        let mut hooks = self.bootstrap_hooks.write().await;
        hooks.push((name, hook));
    }

    /// Register an OnApplicationShutdown hook
    pub async fn register_on_shutdown(&self, name: String, hook: Arc<dyn OnApplicationShutdown>) {
        let mut hooks = self.shutdown_hooks.write().await;
        hooks.push((name, hook));
    }

    /// Register a BeforeApplicationShutdown hook
    pub async fn register_before_shutdown(
        &self,
        name: String,
        hook: Arc<dyn BeforeApplicationShutdown>,
    ) {
        let mut hooks = self.before_shutdown_hooks.write().await;
        hooks.push((name, hook));
    }

    /// Execute all OnModuleInit hooks
    pub async fn call_module_init_hooks(&self) -> Result<(), Vec<(String, Box<dyn std::error::Error + Send + Sync>)>> {
        println!("🔄 Calling module initialization hooks...");
        let hooks = self.init_hooks.read().await;
        let mut errors = Vec::new();

        for (name, hook) in hooks.iter() {
            match hook.on_module_init().await {
                Ok(_) => {
                    println!("  ✓ {}: onModuleInit() completed", name);
                }
                Err(e) => {
                    eprintln!("  ✗ {}: onModuleInit() failed: {}", name, e);
                    errors.push((name.clone(), e));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Execute all OnModuleDestroy hooks
    pub async fn call_module_destroy_hooks(&self) -> Result<(), Vec<(String, Box<dyn std::error::Error + Send + Sync>)>> {
        println!("🔄 Calling module destruction hooks...");
        let hooks = self.destroy_hooks.read().await;
        let mut errors = Vec::new();

        // Call in reverse order (LIFO)
        for (name, hook) in hooks.iter().rev() {
            match hook.on_module_destroy().await {
                Ok(_) => {
                    println!("  ✓ {}: onModuleDestroy() completed", name);
                }
                Err(e) => {
                    eprintln!("  ✗ {}: onModuleDestroy() failed: {}", name, e);
                    errors.push((name.clone(), e));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Execute all OnApplicationBootstrap hooks
    pub async fn call_bootstrap_hooks(&self) -> Result<(), Vec<(String, Box<dyn std::error::Error + Send + Sync>)>> {
        println!("🚀 Calling application bootstrap hooks...");
        let hooks = self.bootstrap_hooks.read().await;
        let mut errors = Vec::new();

        for (name, hook) in hooks.iter() {
            match hook.on_application_bootstrap().await {
                Ok(_) => {
                    println!("  ✓ {}: onApplicationBootstrap() completed", name);
                }
                Err(e) => {
                    eprintln!("  ✗ {}: onApplicationBootstrap() failed: {}", name, e);
                    errors.push((name.clone(), e));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Execute BeforeApplicationShutdown hooks
    pub async fn call_before_shutdown_hooks(
        &self,
        signal: Option<String>,
    ) -> Result<(), Vec<(String, Box<dyn std::error::Error + Send + Sync>)>> {
        println!("⚠️  Calling before shutdown hooks...");
        let hooks = self.before_shutdown_hooks.read().await;
        let mut errors = Vec::new();

        for (name, hook) in hooks.iter() {
            match hook.before_application_shutdown(signal.clone()).await {
                Ok(_) => {
                    println!("  ✓ {}: beforeApplicationShutdown() completed", name);
                }
                Err(e) => {
                    eprintln!("  ✗ {}: beforeApplicationShutdown() failed: {}", name, e);
                    errors.push((name.clone(), e));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Execute all OnApplicationShutdown hooks
    pub async fn call_shutdown_hooks(
        &self,
        signal: Option<String>,
    ) -> Result<(), Vec<(String, Box<dyn std::error::Error + Send + Sync>)>> {
        println!("🛑 Calling application shutdown hooks...");
        let hooks = self.shutdown_hooks.read().await;
        let mut errors = Vec::new();

        // Call in reverse order (LIFO)
        for (name, hook) in hooks.iter().rev() {
            match hook.on_application_shutdown(signal.clone()).await {
                Ok(_) => {
                    println!("  ✓ {}: onApplicationShutdown() completed", name);
                }
                Err(e) => {
                    eprintln!("  ✗ {}: onApplicationShutdown() failed: {}", name, e);
                    errors.push((name.clone(), e));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Get the number of registered hooks of each type
    pub async fn hook_counts(&self) -> LifecycleHookCounts {
        LifecycleHookCounts {
            init: self.init_hooks.read().await.len(),
            destroy: self.destroy_hooks.read().await.len(),
            bootstrap: self.bootstrap_hooks.read().await.len(),
            shutdown: self.shutdown_hooks.read().await.len(),
            before_shutdown: self.before_shutdown_hooks.read().await.len(),
        }
    }

    /// Clear all registered hooks
    pub async fn clear(&self) {
        self.init_hooks.write().await.clear();
        self.destroy_hooks.write().await.clear();
        self.bootstrap_hooks.write().await.clear();
        self.shutdown_hooks.write().await.clear();
        self.before_shutdown_hooks.write().await.clear();
        self.hook_registry.write().await.clear();
    }
}

impl Default for LifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about registered lifecycle hooks
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LifecycleHookCounts {
    pub init: usize,
    pub destroy: usize,
    pub bootstrap: usize,
    pub shutdown: usize,
    pub before_shutdown: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestService {
        name: String,
        init_called: Arc<RwLock<bool>>,
        destroy_called: Arc<RwLock<bool>>,
    }

    #[async_trait]
    impl OnModuleInit for TestService {
        async fn on_module_init(&self) -> LifecycleResult {
            *self.init_called.write().await = true;
            Ok(())
        }
    }

    #[async_trait]
    impl OnModuleDestroy for TestService {
        async fn on_module_destroy(&self) -> LifecycleResult {
            *self.destroy_called.write().await = true;
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_lifecycle_manager_registration() {
        let manager = LifecycleManager::new();
        let init_called = Arc::new(RwLock::new(false));
        let destroy_called = Arc::new(RwLock::new(false));

        let service = Arc::new(TestService {
            name: "TestService".to_string(),
            init_called: init_called.clone(),
            destroy_called: destroy_called.clone(),
        });

        manager
            .register_on_init("TestService".to_string(), service.clone())
            .await;
        manager
            .register_on_destroy("TestService".to_string(), service.clone())
            .await;

        let counts = manager.hook_counts().await;
        assert_eq!(counts.init, 1);
        assert_eq!(counts.destroy, 1);
    }

    #[tokio::test]
    async fn test_lifecycle_hooks_execution() {
        let manager = LifecycleManager::new();
        let init_called = Arc::new(RwLock::new(false));
        let destroy_called = Arc::new(RwLock::new(false));

        let service = Arc::new(TestService {
            name: "TestService".to_string(),
            init_called: init_called.clone(),
            destroy_called: destroy_called.clone(),
        });

        manager
            .register_on_init("TestService".to_string(), service.clone())
            .await;
        manager
            .register_on_destroy("TestService".to_string(), service.clone())
            .await;

        // Execute init hooks
        manager.call_module_init_hooks().await.unwrap();
        assert!(*init_called.read().await);

        // Execute destroy hooks
        manager.call_module_destroy_hooks().await.unwrap();
        assert!(*destroy_called.read().await);
    }

    #[tokio::test]
    async fn test_lifecycle_hook_order() {
        let manager = LifecycleManager::new();
        let order = Arc::new(RwLock::new(Vec::new()));

        struct OrderService {
            id: usize,
            order: Arc<RwLock<Vec<usize>>>,
        }

        #[async_trait]
        impl OnModuleInit for OrderService {
            async fn on_module_init(&self) -> LifecycleResult {
                self.order.write().await.push(self.id);
                Ok(())
            }
        }

        for i in 1..=3 {
            let service = Arc::new(OrderService {
                id: i,
                order: order.clone(),
            });
            manager
                .register_on_init(format!("Service{}", i), service)
                .await;
        }

        manager.call_module_init_hooks().await.unwrap();

        let execution_order = order.read().await.clone();
        assert_eq!(execution_order, vec![1, 2, 3]);
    }
}

