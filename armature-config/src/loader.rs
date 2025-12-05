// Configuration file loaders

use crate::{ConfigError, Result};
use serde_json::Value;
use std::fs;
use std::path::Path;

/// Supported configuration file formats
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileFormat {
    Json,
    Toml,
    Env,
}

impl FileFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "json" => Some(FileFormat::Json),
            "toml" => Some(FileFormat::Toml),
            "env" => Some(FileFormat::Env),
            _ => None,
        }
    }
}

/// Configuration file loader
pub struct ConfigLoader {
    format: FileFormat,
}

impl ConfigLoader {
    pub fn new(format: FileFormat) -> Self {
        Self { format }
    }

    /// Auto-detect format from file extension
    pub fn auto(path: &str) -> Result<Self> {
        let path_obj = Path::new(path);
        let ext = path_obj
            .extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| ConfigError::LoadError("No file extension found".to_string()))?;

        let format = FileFormat::from_extension(ext)
            .ok_or_else(|| ConfigError::LoadError(format!("Unsupported format: {}", ext)))?;

        Ok(Self::new(format))
    }

    /// Load configuration from file
    pub fn load_file(&self, path: &str) -> Result<Value> {
        let content = fs::read_to_string(path)
            .map_err(|e| ConfigError::LoadError(format!("Failed to read file: {}", e)))?;

        self.parse(&content)
    }

    /// Parse configuration from string
    pub fn parse(&self, content: &str) -> Result<Value> {
        match self.format {
            FileFormat::Json => self.parse_json(content),
            FileFormat::Toml => self.parse_toml(content),
            FileFormat::Env => self.parse_env(content),
        }
    }

    fn parse_json(&self, content: &str) -> Result<Value> {
        serde_json::from_str(content)
            .map_err(|e| ConfigError::ParseError(format!("JSON parse error: {}", e)))
    }

    fn parse_toml(&self, content: &str) -> Result<Value> {
        let toml_value: toml::Value = toml::from_str(content)
            .map_err(|e| ConfigError::ParseError(format!("TOML parse error: {}", e)))?;

        // Convert TOML value to JSON value
        let json_str = serde_json::to_string(&toml_value)
            .map_err(|e| ConfigError::SerializationError(e.to_string()))?;

        serde_json::from_str(&json_str)
            .map_err(|e| ConfigError::ParseError(format!("TOML to JSON conversion error: {}", e)))
    }

    fn parse_env(&self, content: &str) -> Result<Value> {
        let mut map = serde_json::Map::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim().trim_matches('"').trim_matches('\'');
                map.insert(key.to_string(), Value::String(value.to_string()));
            }
        }

        Ok(Value::Object(map))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json() {
        let loader = ConfigLoader::new(FileFormat::Json);
        let json = r#"{"key": "value", "number": 42}"#;

        let result = loader.parse(json).unwrap();
        assert!(result.is_object());
    }

    #[test]
    fn test_parse_toml() {
        let loader = ConfigLoader::new(FileFormat::Toml);
        let toml = r#"
            key = "value"
            number = 42
        "#;

        let result = loader.parse(toml).unwrap();
        assert!(result.is_object());
    }

    #[test]
    fn test_parse_env() {
        let loader = ConfigLoader::new(FileFormat::Env);
        let env = r#"
            KEY=value
            NUMBER=42
            # Comment
            QUOTED="quoted value"
        "#;

        let result = loader.parse(env).unwrap();
        assert!(result.is_object());
    }

    #[test]
    fn test_format_detection() {
        assert_eq!(FileFormat::from_extension("json"), Some(FileFormat::Json));
        assert_eq!(FileFormat::from_extension("toml"), Some(FileFormat::Toml));
        assert_eq!(FileFormat::from_extension("env"), Some(FileFormat::Env));
        assert_eq!(FileFormat::from_extension("unknown"), None);
    }
}
