//! Integration tests for armature-svelte

use armature_svelte::*;

#[test]
fn test_svelte_config_creation() {
    let config = SvelteConfig::new("/path/to/build");

    assert_eq!(config.build_dir, "/path/to/build");
    assert_eq!(config.server_entry, "index.js");
}

#[test]
fn test_svelte_config_builder() {
    let config = SvelteConfig::new("/build")
        .with_server_entry("custom-server.js")
        .with_hydration(true)
        .with_prerender(true)
        .with_cache(true)
        .with_compression(false);

    assert_eq!(config.server_entry, "custom-server.js");
    assert!(config.enable_hydration);
    assert!(config.enable_prerender);
    assert!(config.enable_cache);
    assert!(!config.enable_compression);
}

#[test]
fn test_svelte_error_display() {
    let err = SvelteError::ConfigError("Invalid config".to_string());
    let display = format!("{}", err);
    assert!(display.contains("Invalid config"));
}

#[test]
fn test_svelte_config_static_dir() {
    let config = SvelteConfig::new("/build");
    assert_eq!(config.static_dir(), "/build/client");
}

#[test]
fn test_svelte_config_server_path() {
    let config = SvelteConfig::new("/build");
    let path = config.server_path();
    assert!(path.ends_with("index.js"));
}

