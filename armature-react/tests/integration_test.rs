//! Integration tests for armature-react

use armature_react::*;

#[test]
fn test_react_config_creation() {
    let config = ReactConfig::new("/path/to/build");

    assert_eq!(config.build_dir, "/path/to/build");
    assert_eq!(config.server_entry, "server.js");
}

#[test]
fn test_react_config_builder() {
    let config = ReactConfig::new("/build")
        .with_server_entry("custom-server.js")
        .with_index_html("custom-index.html")
        .with_cache(true)
        .with_compression(false);

    assert_eq!(config.server_entry, "custom-server.js");
    assert_eq!(config.index_html, "custom-index.html");
    assert!(config.enable_cache);
    assert!(!config.enable_compression);
}

#[test]
fn test_react_error_display() {
    let err = ReactError::ConfigError("Invalid config".to_string());
    let display = format!("{}", err);
    assert!(display.contains("Invalid config"));
}

#[test]
fn test_react_config_static_dir() {
    let config = ReactConfig::new("/build");
    assert_eq!(config.static_dir(), "/build/static");
}

#[test]
fn test_react_config_server_path() {
    let config = ReactConfig::new("/build");
    let path = config.server_path();
    assert!(path.ends_with("server.js"));
}


