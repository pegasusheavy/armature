// Configuration management for Armature framework

pub mod config_service;
pub mod env;
pub mod error;
pub mod loader;
pub mod validation;

pub use config_service::ConfigService;
pub use env::EnvLoader;
pub use error::{ConfigError, Result};
pub use loader::{ConfigLoader, FileFormat};
pub use validation::{ConfigValidator, Validate};

use armature_core::Provider;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Main configuration manager
#[derive(Clone)]
pub struct ConfigManager {
    config: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    env_prefix: Option<String>,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(HashMap::new())),
            env_prefix: None,
        }
    }

    /// Create with environment variable prefix
    pub fn with_prefix(prefix: String) -> Self {
        Self {
            config: Arc::new(RwLock::new(HashMap::new())),
            env_prefix: Some(prefix),
        }
    }

    /// Load configuration from environment variables
    pub fn load_env(&self) -> Result<()> {
        let loader = EnvLoader::new(self.env_prefix.clone());
        let env_vars = loader.load()?;

        let mut config = self.config.write().unwrap();
        for (key, value) in env_vars {
            config.insert(key, serde_json::Value::String(value));
        }

        Ok(())
    }

    /// Load configuration from .env file
    pub fn load_dotenv(&self, path: Option<&str>) -> Result<()> {
        if let Some(path) = path {
            dotenvy::from_path(path).map_err(|e| ConfigError::LoadError(e.to_string()))?;
        } else {
            dotenvy::dotenv().ok(); // Ignore if .env doesn't exist
        }
        self.load_env()
    }

    /// Load configuration from file
    pub fn load_file(&self, path: &str, format: FileFormat) -> Result<()> {
        let loader = ConfigLoader::new(format);
        let data = loader.load_file(path)?;

        let mut config = self.config.write().unwrap();
        if let serde_json::Value::Object(map) = data {
            for (key, value) in map {
                config.insert(key, value);
            }
        }

        Ok(())
    }

    /// Set a configuration value
    pub fn set<T: serde::Serialize>(&self, key: &str, value: T) -> Result<()> {
        let json_value = serde_json::to_value(value)
            .map_err(|e| ConfigError::SerializationError(e.to_string()))?;

        let mut config = self.config.write().unwrap();
        config.insert(key.to_string(), json_value);

        Ok(())
    }

    /// Get a configuration value
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Result<T> {
        let config = self.config.read().unwrap();

        let value = config
            .get(key)
            .ok_or_else(|| ConfigError::KeyNotFound(key.to_string()))?;

        serde_json::from_value(value.clone())
            .map_err(|e| ConfigError::DeserializationError(e.to_string()))
    }

    /// Get a configuration value with default
    pub fn get_or<T: DeserializeOwned>(&self, key: &str, default: T) -> T {
        self.get(key).unwrap_or(default)
    }

    /// Get a string value
    pub fn get_string(&self, key: &str) -> Result<String> {
        self.get(key)
    }

    /// Get an integer value
    pub fn get_int(&self, key: &str) -> Result<i64> {
        self.get(key)
    }

    /// Get a boolean value
    pub fn get_bool(&self, key: &str) -> Result<bool> {
        self.get(key)
    }

    /// Get a float value
    pub fn get_float(&self, key: &str) -> Result<f64> {
        self.get(key)
    }

    /// Check if a key exists
    pub fn has(&self, key: &str) -> bool {
        let config = self.config.read().unwrap();
        config.contains_key(key)
    }

    /// Get all configuration keys
    pub fn keys(&self) -> Vec<String> {
        let config = self.config.read().unwrap();
        config.keys().cloned().collect()
    }

    /// Merge configuration from another manager
    pub fn merge(&self, other: &ConfigManager) -> Result<()> {
        let other_config = other.config.read().unwrap();
        let mut config = self.config.write().unwrap();

        for (key, value) in other_config.iter() {
            config.insert(key.clone(), value.clone());
        }

        Ok(())
    }

    /// Load and validate configuration
    pub fn load_validated<T: DeserializeOwned + Validate>(&self) -> Result<T> {
        let config = self.config.read().unwrap();
        let json_value =
            serde_json::Value::Object(config.iter().map(|(k, v)| (k.clone(), v.clone())).collect());

        let validated: T = serde_json::from_value(json_value)
            .map_err(|e| ConfigError::DeserializationError(e.to_string()))?;

        validated.validate()?;

        Ok(validated)
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for ConfigManager {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_and_get() {
        let manager = ConfigManager::new();
        manager.set("test_key", "test_value").unwrap();

        let value: String = manager.get("test_key").unwrap();
        assert_eq!(value, "test_value");
    }

    #[test]
    fn test_get_or_default() {
        let manager = ConfigManager::new();

        let value: String = manager.get_or("missing_key", "default_value".to_string());
        assert_eq!(value, "default_value");
    }

    #[test]
    fn test_has_key() {
        let manager = ConfigManager::new();
        manager.set("existing_key", "value").unwrap();

        assert!(manager.has("existing_key"));
        assert!(!manager.has("missing_key"));
    }

    #[test]
    fn test_type_conversions() {
        let manager = ConfigManager::new();

        manager.set("string_key", "hello").unwrap();
        manager.set("int_key", 42i64).unwrap();
        manager.set("bool_key", true).unwrap();
        manager.set("float_key", 3.14).unwrap();

        assert_eq!(manager.get_string("string_key").unwrap(), "hello");
        assert_eq!(manager.get_int("int_key").unwrap(), 42);
        assert_eq!(manager.get_bool("bool_key").unwrap(), true);
        assert_eq!(manager.get_float("float_key").unwrap(), 3.14);
    }
}
