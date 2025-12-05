// Svelte SSR Renderer

use crate::config::SvelteConfig;
use armature_core::Error;
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

/// Svelte Server-Side Renderer
#[derive(Clone)]
pub struct SvelteRenderer {
    config: SvelteConfig,
}

impl SvelteRenderer {
    /// Create a new Svelte renderer
    pub fn new(config: SvelteConfig) -> Self {
        Self { config }
    }

    /// Render a Svelte application for a given URL
    pub async fn render(
        &self,
        url: &str,
        props: Option<serde_json::Value>,
    ) -> Result<SvelteRenderResult, Error> {
        // Build the render command
        let server_path = self.config.build_dir.join(&self.config.server_entry);

        if !server_path.exists() {
            return Err(Error::Internal(format!(
                "Svelte server entry not found: {}",
                server_path.display()
            )));
        }

        // Prepare render request
        let render_request = serde_json::json!({
            "url": url,
            "props": props.unwrap_or(serde_json::json!({})),
            "hydration": self.config.hydration
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
            return Err(Error::Internal(format!("Svelte SSR failed: {}", stderr)));
        }

        let output_str = String::from_utf8(output.stdout)
            .map_err(|e| Error::Internal(format!("Invalid UTF-8 from Node.js: {}", e)))?;

        // Parse the SSR result (Svelte returns { html, css, head })
        let result: SvelteRenderResult = serde_json::from_str(&output_str)
            .map_err(|e| Error::Serialization(format!("Failed to parse SSR result: {}", e)))?;

        Ok(result)
    }

    /// Check if the renderer is ready
    pub async fn health_check(&self) -> Result<(), Error> {
        let server_path = self.config.build_dir.join(&self.config.server_entry);

        if !server_path.exists() {
            return Err(Error::Internal(format!(
                "Svelte server entry not found: {}",
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

/// Result from Svelte SSR rendering
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct SvelteRenderResult {
    /// Rendered HTML
    pub html: String,

    /// Component CSS (if any)
    pub css: Option<SvelteRenderCss>,

    /// Head elements (title, meta, links)
    pub head: Option<String>,
}

/// CSS result from Svelte SSR
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct SvelteRenderCss {
    /// CSS code
    pub code: String,

    /// CSS source map (if available)
    pub map: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_renderer_creation() {
        let config = SvelteConfig::new(PathBuf::from("build"));
        let _renderer = SvelteRenderer::new(config);
    }
}
