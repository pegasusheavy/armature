//! Integration tests for armature-angular

use armature_angular::*;

#[test]
fn test_angular_config_creation() {
    let config = AngularConfig::new("/path/to/dist");

    assert_eq!(config.dist_dir, "/path/to/dist");
    assert_eq!(config.server_bundle, "main.server.mjs");
}

#[test]
fn test_angular_config_builder() {
    let config = AngularConfig::new("/dist")
        .with_server_bundle("custom.mjs")
        .with_index_html("custom-index.html")
        .with_cache(true)
        .with_compression(false)
        .exclude_route("/api/*");

    assert_eq!(config.server_bundle, "custom.mjs");
    assert_eq!(config.index_html, "custom-index.html");
    assert!(config.enable_cache);
    assert!(!config.enable_compression);
    assert_eq!(config.exclude_routes.len(), 1);
}

#[test]
fn test_angular_error_display() {
    let err = AngularError::ConfigError("Invalid config".to_string());
    let display = format!("{}", err);
    assert!(display.contains("Invalid config"));
}

#[test]
fn test_angular_config_static_dir() {
    let config = AngularConfig::new("/dist");
    assert_eq!(config.static_dir(), "/dist/browser");
}

#[test]
fn test_angular_config_server_bundle_path() {
    let config = AngularConfig::new("/dist");
    let path = config.server_bundle_path();
    assert!(path.ends_with("main.server.mjs"));
}

