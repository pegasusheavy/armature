// Vue.js SSR Renderer

use crate::config::VueConfig;
use armature_core::Error;
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

/// Vue.js Server-Side Renderer
#[derive(Clone)]
pub struct VueRenderer {
    config: VueConfig,
}

impl VueRenderer {
    /// Create a new Vue renderer
    pub fn new(config: VueConfig) -> Self {
        Self { config }
    }

    /// Render a Vue application for a given URL
    pub async fn render(
        &self,
        url: &str,
        context: Option<serde_json::Value>,
    ) -> Result<String, Error> {
        // Build the render command
        let server_path = self.config.build_dir.join(&self.config.server_entry);

        if !server_path.exists() {
            return Err(Error::Internal(format!(
                "Vue server entry not found: {}",
                server_path.display()
            )));
        }

        // Prepare render request
        let render_request = serde_json::json!({
            "url": url,
            "context": context.unwrap_or(serde_json::json!({})),
            "template": self.read_template().await.ok()
        });

        // Execute Node.js process to render
        let mut child = Command::new(&self.config.node_path)
            .arg(&server_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| Error::Internal(format!("Failed to spawn Node.js: {}", e)))?;

        // Write request to stdin
        if let Some(mut stdin) = child.stdin.take() {
            let request_json = serde_json::to_string(&render_request)
                .map_err(|e| Error::Serialization(e.to_string()))?;

            stdin
                .write_all(request_json.as_bytes())
                .await
                .map_err(|e| Error::Internal(format!("Failed to write to Node.js: {}", e)))?;

            stdin
                .write_all(b"\n")
                .await
                .map_err(|e| Error::Internal(format!("Failed to write to Node.js: {}", e)))?;
        }

        // Wait for output
        let output = child
            .wait_with_output()
            .await
            .map_err(|e| Error::Internal(format!("Failed to wait for Node.js: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Internal(format!("Vue SSR failed: {}", stderr)));
        }

        let html = String::from_utf8(output.stdout)
            .map_err(|e| Error::Internal(format!("Invalid UTF-8 from Node.js: {}", e)))?;

        Ok(html)
    }

    /// Read the HTML template
    async fn read_template(&self) -> Result<String, Error> {
        tokio::fs::read_to_string(&self.config.template_path)
            .await
            .map_err(|e| {
                Error::Internal(format!(
                    "Failed to read template {}: {}",
                    self.config.template_path.display(),
                    e
                ))
            })
    }

    /// Render with pre-fetched data
    pub async fn render_with_data(
        &self,
        url: &str,
        initial_state: serde_json::Value,
    ) -> Result<String, Error> {
        let context = serde_json::json!({
            "state": initial_state,
            "url": url
        });

        self.render(url, Some(context)).await
    }

    /// Check if the renderer is ready
    pub async fn health_check(&self) -> Result<(), Error> {
        let server_path = self.config.build_dir.join(&self.config.server_entry);

        if !server_path.exists() {
            return Err(Error::Internal(format!(
                "Vue server entry not found: {}",
                server_path.display()
            )));
        }

        // Try to execute Node.js
        let output = Command::new(&self.config.node_path)
            .arg("--version")
            .output()
            .await
            .map_err(|e| Error::Internal(format!("Node.js not found: {}", e)))?;

        if !output.status.success() {
            return Err(Error::Internal("Node.js is not working".to_string()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_renderer_creation() {
        let config = VueConfig::new(PathBuf::from("dist"));
        let _renderer = VueRenderer::new(config);
    }
}
