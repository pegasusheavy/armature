// Angular Universal SSR renderer

use crate::{AngularError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

/// Options for rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderOptions {
    /// URL to render
    pub url: String,

    /// Request headers
    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,

    /// Cookies
    #[serde(default)]
    pub cookies: std::collections::HashMap<String, String>,

    /// User agent
    #[serde(default)]
    pub user_agent: Option<String>,
}

impl RenderOptions {
    pub fn new(url: String) -> Self {
        Self {
            url,
            headers: std::collections::HashMap::new(),
            cookies: std::collections::HashMap::new(),
            user_agent: None,
        }
    }
}

/// Angular Universal renderer
#[derive(Clone)]
pub struct AngularRenderer {
    node_path: PathBuf,
    server_bundle: PathBuf,
}

impl AngularRenderer {
    /// Create a new renderer
    pub fn new(node_path: PathBuf, server_bundle: PathBuf) -> Result<Self> {
        if !server_bundle.exists() {
            return Err(AngularError::ConfigError(format!(
                "Server bundle not found: {:?}",
                server_bundle
            )));
        }

        Ok(Self {
            node_path,
            server_bundle,
        })
    }

    /// Render a URL server-side
    pub async fn render(&self, url: &str, options: RenderOptions) -> Result<String> {
        // Create a simple Node.js script to render the page
        let render_script = self.create_render_script(url, &options)?;

        // Execute Node.js with the render script
        let child = Command::new(&self.node_path)
            .arg("-e")
            .arg(&render_script)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| AngularError::NodeError(format!("Failed to spawn Node.js: {}", e)))?;

        // Wait for the process to complete
        let output = child
            .wait_with_output()
            .await
            .map_err(|e| AngularError::NodeError(format!("Failed to wait for Node.js: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AngularError::RenderError(format!(
                "Node.js rendering failed: {}",
                stderr
            )));
        }

        let html = String::from_utf8(output.stdout)
            .map_err(|e| AngularError::RenderError(format!("Invalid UTF-8 in output: {}", e)))?;

        Ok(html)
    }

    /// Create a Node.js script for rendering
    fn create_render_script(&self, url: &str, _options: &RenderOptions) -> Result<String> {
        let bundle_path = self
            .server_bundle
            .to_str()
            .ok_or_else(|| AngularError::InvalidPath("Invalid server bundle path".to_string()))?;

        // This is a simplified example. In production, you'd need to properly
        // integrate with Angular Universal's renderModule or renderApplication
        let script = format!(
            r#"
            (async () => {{
                try {{
                    const {{ renderApplication }} = require('@angular/platform-server');
                    const {{ AppServerModule }} = require('{}');

                    const html = await renderApplication(AppServerModule, {{
                        document: '<app-root></app-root>',
                        url: '{}',
                    }});

                    console.log(html);
                }} catch (error) {{
                    console.error('Rendering error:', error);
                    process.exit(1);
                }}
            }})();
            "#,
            bundle_path, url
        );

        Ok(script)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_options() {
        let options = RenderOptions::new("/test".to_string());
        assert_eq!(options.url, "/test");
        assert!(options.headers.is_empty());
    }

    #[test]
    fn test_create_render_script() {
        let renderer = AngularRenderer {
            node_path: PathBuf::from("node"),
            server_bundle: PathBuf::from("dist/server/main.js"),
        };

        let script = renderer
            .create_render_script("/test", &RenderOptions::new("/test".to_string()))
            .unwrap();

        assert!(script.contains("renderApplication"));
        assert!(script.contains("/test"));
    }
}
