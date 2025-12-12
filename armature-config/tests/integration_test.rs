//! Integration tests for armature-config

use armature_config::*;
use std::env;

#[test]
fn test_config_manager_creation() {
    let _manager = ConfigManager::new();
    // ConfigManager created successfully
}

#[test]
fn test_env_loader() {
    let loader = EnvLoader::new(None);

    // Set a test environment variable
    unsafe {
        env::set_var("TEST_INTEGRATION_VAR", "integration_value");
    }

    let result = loader.load_var("TEST_INTEGRATION_VAR");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "integration_value");

    // Cleanup
    unsafe {
        env::remove_var("TEST_INTEGRATION_VAR");
    }
}

#[test]
fn test_env_loader_with_prefix() {
    let loader = EnvLoader::new(Some("MYAPP".to_string()));

    unsafe {
        env::set_var("MYAPP_DATABASE_URL", "postgres://localhost");
    }

    let result = loader.load_var("DATABASE_URL");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "postgres://localhost");

    // Cleanup
    unsafe {
        env::remove_var("MYAPP_DATABASE_URL");
    }
}

#[test]
fn test_env_loader_missing_var() {
    let loader = EnvLoader::new(None);

    let result = loader.load_var("NONEXISTENT_VAR_123456");
    assert!(result.is_err());
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
