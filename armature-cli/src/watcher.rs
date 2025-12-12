//! File watcher utilities for development server.

use std::path::Path;

/// File watcher configuration.
#[allow(dead_code)]
pub struct WatchConfig {
    /// Directories to watch.
    pub paths: Vec<String>,
    /// File extensions to watch.
    pub extensions: Vec<String>,
    /// Debounce duration in milliseconds.
    pub debounce_ms: u64,
    /// Ignore patterns.
    pub ignore: Vec<String>,
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            paths: vec!["src".to_string()],
            extensions: vec!["rs".to_string()],
            debounce_ms: 500,
            ignore: vec![
                "target".to_string(),
                ".git".to_string(),
                "node_modules".to_string(),
            ],
        }
    }
}

/// Check if a path should be ignored based on patterns.
#[allow(dead_code)]
pub fn should_ignore(path: &Path, patterns: &[String]) -> bool {
    let path_str = path.to_string_lossy();
    patterns.iter().any(|p| path_str.contains(p))
}

/// Check if a file has a watched extension.
#[allow(dead_code)]
pub fn has_watched_extension(path: &Path, extensions: &[String]) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| extensions.iter().any(|e| e == ext))
        .unwrap_or(false)
}
