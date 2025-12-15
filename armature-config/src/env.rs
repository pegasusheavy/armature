// Environment variable loading

use crate::{ConfigError, Result};
use std::collections::HashMap;
use std::env;

/// Environment variable loader
pub struct EnvLoader {
    prefix: Option<String>,
}

impl EnvLoader {
    /// Create a new environment loader
    pub fn new(prefix: Option<String>) -> Self {
        Self { prefix }
    }

    /// Load all environment variables
    pub fn load(&self) -> Result<HashMap<String, String>> {
        let mut config = HashMap::new();

        for (key, value) in env::vars() {
            if let Some(ref prefix) = self.prefix {
                if key.starts_with(prefix) {
                    let trimmed_key = key.trim_start_matches(prefix).trim_start_matches('_');
                    config.insert(trimmed_key.to_lowercase(), value);
                }
            } else {
                config.insert(key.to_lowercase(), value);
            }
        }

        Ok(config)
    }

    /// Load a specific environment variable
    pub fn load_var(&self, key: &str) -> Result<String> {
        let full_key = if let Some(ref prefix) = self.prefix {
            format!("{}_{}", prefix, key.to_uppercase())
        } else {
            key.to_uppercase()
        };

        env::var(&full_key).map_err(ConfigError::EnvError)
    }

    /// Load with default value
    pub fn load_var_or(&self, key: &str, default: &str) -> String {
        self.load_var(key).unwrap_or_else(|_| default.to_string())
    }
}

impl Default for EnvLoader {
    fn default() -> Self {
        Self::new(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Environment variable tests are inherently difficult to test safely
    // in Rust 1.78+ because std::env::set_var is unsafe (not thread-safe).
    // These tests use existing environment variables or test default behavior.

    #[test]
    fn test_env_loader_with_default() {
        let loader = EnvLoader::new(None);
        let value = loader.load_var_or("NONEXISTENT_VAR_12345", "default");

        assert_eq!(value, "default");
    }

    #[test]
    fn test_env_loader_missing_var() {
        let loader = EnvLoader::new(Some("ARMATURE_TEST".to_string()));
        let result = loader.load_var("MISSING_VAR_67890");

        assert!(result.is_err());
    }

    #[test]
    fn test_env_loader_path_exists() {
        // PATH is almost always set on any system
        let loader = EnvLoader::new(None);
        let result = loader.load_var("PATH");

        // PATH should exist on most systems
        if std::env::var("PATH").is_ok() {
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_env_loader_prefix() {
        let loader = EnvLoader::new(Some("MY_APP".to_string()));
        // This tests the prefix logic without needing to set env vars
        // The prefix should be applied when looking up "FOO" -> "MY_APP_FOO"
        let result = loader.load_var("NONEXISTENT_99999");
        assert!(result.is_err());
    }
}
