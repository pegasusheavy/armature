// Static file serving for Angular applications

use crate::{AngularError, Result};
use std::path::{Path, PathBuf};
use tokio::fs;

/// Service for serving static files
#[derive(Clone)]
pub struct StaticFileService {
    root_dir: PathBuf,
    index_html: PathBuf,
}

impl StaticFileService {
    /// Create a new static file service
    pub fn new(root_dir: PathBuf, index_html: PathBuf) -> Result<Self> {
        if !root_dir.exists() {
            return Err(AngularError::ConfigError(format!(
                "Static files directory not found: {:?}",
                root_dir
            )));
        }

        Ok(Self {
            root_dir,
            index_html,
        })
    }

    /// Serve a static file
    pub async fn serve(&self, path: &str) -> Result<Vec<u8>> {
        let safe_path = self.sanitize_path(path)?;
        let full_path = self.root_dir.join(&safe_path);

        // Check if file exists
        if !full_path.exists() {
            return Err(AngularError::FileNotFound(path.to_string()));
        }

        // Read and return the file
        let content = fs::read(&full_path).await.map_err(AngularError::Io)?;

        Ok(content)
    }

    /// Get the content type for a file
    pub fn get_content_type(&self, path: &str) -> String {
        mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string()
    }

    /// Serve the index.html file
    pub async fn serve_index(&self) -> Result<Vec<u8>> {
        fs::read(&self.index_html).await.map_err(AngularError::Io)
    }

    /// Sanitize a path to prevent directory traversal
    fn sanitize_path(&self, path: &str) -> Result<PathBuf> {
        let path = path.trim_start_matches('/');
        let decoded = percent_encoding::percent_decode_str(path)
            .decode_utf8()
            .map_err(|_| AngularError::InvalidPath("Invalid UTF-8 in path".to_string()))?;

        let path = Path::new(decoded.as_ref());

        // Check for directory traversal
        for component in path.components() {
            if matches!(component, std::path::Component::ParentDir) {
                return Err(AngularError::InvalidPath(
                    "Directory traversal not allowed".to_string(),
                ));
            }
        }

        Ok(path.to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_path() {
        let service = StaticFileService {
            root_dir: PathBuf::from("dist/browser"),
            index_html: PathBuf::from("dist/browser/index.html"),
        };

        // Valid paths
        assert!(service.sanitize_path("/assets/logo.png").is_ok());
        assert!(service.sanitize_path("main.js").is_ok());

        // Invalid paths (directory traversal)
        assert!(service.sanitize_path("../../../etc/passwd").is_err());
        assert!(service.sanitize_path("/assets/../../../secret").is_err());
    }

    #[test]
    fn test_get_content_type() {
        let service = StaticFileService {
            root_dir: PathBuf::from("dist/browser"),
            index_html: PathBuf::from("dist/browser/index.html"),
        };

        assert_eq!(service.get_content_type("test.js"), "text/javascript");
        assert_eq!(service.get_content_type("test.css"), "text/css");
        assert_eq!(service.get_content_type("test.html"), "text/html");
        assert_eq!(service.get_content_type("test.png"), "image/png");
    }
}
