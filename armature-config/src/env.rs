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

    #[test]
    fn test_env_loader() {
        unsafe {
            env::set_var("TEST_VAR", "test_value");
        }

        let loader = EnvLoader::new(Some("TEST".to_string()));
        let value = loader.load_var("VAR").unwrap();

        assert_eq!(value, "test_value");

        unsafe {
            env::remove_var("TEST_VAR");
        }
    }

    #[test]
    fn test_env_loader_with_default() {
        let loader = EnvLoader::new(None);
        let value = loader.load_var_or("NONEXISTENT_VAR", "default");

        assert_eq!(value, "default");
    }
}
