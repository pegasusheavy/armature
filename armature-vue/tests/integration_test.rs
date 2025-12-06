//! Integration tests for armature-vue

use armature_vue::*;

#[test]
fn test_vue_config_creation() {
    let config = VueConfig::new("/path/to/dist");

    assert_eq!(config.build_dir, "/path/to/dist");
    assert_eq!(config.server_entry, "entry-server.js");
}

#[test]
fn test_vue_config_builder() {
    let config = VueConfig::new("/dist")
        .with_server_entry("custom-server.js")
        .with_client_manifest("custom-manifest.json")
        .with_template("custom-template.html")
        .with_cache(true)
        .with_compression(false);

    assert_eq!(config.server_entry, "custom-server.js");
    assert_eq!(config.client_manifest, "custom-manifest.json");
    assert_eq!(config.template, "custom-template.html");
    assert!(config.enable_cache);
    assert!(!config.enable_compression);
}

#[test]
fn test_vue_error_display() {
    let err = VueError::ConfigError("Invalid config".to_string());
    let display = format!("{}", err);
    assert!(display.contains("Invalid config"));
}

#[test]
fn test_vue_config_static_dir() {
    let config = VueConfig::new("/dist");
    assert_eq!(config.static_dir(), "/dist/client");
}

#[test]
fn test_vue_config_server_path() {
    let config = VueConfig::new("/dist");
    let path = config.server_path();
    assert!(path.ends_with("entry-server.js"));
}


