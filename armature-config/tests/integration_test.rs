//! Integration tests for armature-config
//!
//! Note: As of Rust 1.78, std::env::set_var is unsafe because it's not thread-safe.
//! These tests are designed to work without setting environment variables.

use armature_config::*;

#[test]
fn test_config_manager_creation() {
    let _manager = ConfigManager::new();
    // ConfigManager created successfully
}

#[test]
fn test_env_loader_existing_var() {
    let loader = EnvLoader::new(None);

    // Test with an environment variable that should exist on most systems
    // PATH on Unix/Windows, or HOME on Unix
    #[cfg(unix)]
    {
        if std::env::var("HOME").is_ok() {
            let result = loader.load_var("HOME");
            assert!(result.is_ok());
        }
    }

    #[cfg(windows)]
    {
        if std::env::var("USERPROFILE").is_ok() {
            let result = loader.load_var("USERPROFILE");
            assert!(result.is_ok());
        }
    }
}

#[test]
fn test_env_loader_with_prefix_missing() {
    let loader = EnvLoader::new(Some("MYAPP_INTEGRATION_TEST".to_string()));

    // Test that missing prefixed vars return error
    let result = loader.load_var("NONEXISTENT_VAR");
    assert!(result.is_err());
}

#[test]
fn test_env_loader_missing_var() {
    let loader = EnvLoader::new(None);

    let result = loader.load_var("NONEXISTENT_VAR_123456_INTEGRATION_TEST");
    assert!(result.is_err());
}

#[test]
fn test_env_loader_default_value() {
    let loader = EnvLoader::new(None);

    let result = loader.load_var_or("MISSING_VAR_INTEGRATION", "default_value");
    assert_eq!(result, "default_value");
}

#[test]
fn test_config_error_display() {
    let err = ConfigError::ParseError("test_key".to_string());
    let display = format!("{}", err);
    assert!(display.contains("test_key"));
}

#[test]
fn test_config_manager_set_get() {
    let manager = ConfigManager::new();

    // Set a value
    manager.set("test_key", "test_value").unwrap();

    // Get the value
    let result = manager.get_string("test_key");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "test_value");
}

#[test]
fn test_config_manager_has_key() {
    let manager = ConfigManager::new();

    assert!(!manager.has("missing_key"));

    manager.set("present_key", "value").unwrap();
    assert!(manager.has("present_key"));
}

#[test]
fn test_config_manager_remove_key() {
    let manager = ConfigManager::new();

    manager.set("removable_key", "value").unwrap();
    assert!(manager.has("removable_key"));

    manager.remove("removable_key");
    assert!(!manager.has("removable_key"));
}

#[test]
fn test_config_manager_clear() {
    let manager = ConfigManager::new();

    manager.set("key1", "value1").unwrap();
    manager.set("key2", "value2").unwrap();

    assert!(manager.has("key1"));
    assert!(manager.has("key2"));

    manager.clear();

    assert!(!manager.has("key1"));
    assert!(!manager.has("key2"));
}
