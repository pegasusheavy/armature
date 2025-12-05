// Vue.js SSR Service

use crate::{VueConfig, VueRenderer};
use armature_core::{Error, HttpRequest, HttpResponse};
use std::collections::HashMap;
use std::path::Path;

/// Vue.js SSR Service - Injectable service for Vue server-side rendering
#[derive(Clone)]
pub struct VueService {
    renderer: VueRenderer,
    config: VueConfig,
}

impl VueService {
    /// Create a new Vue service
    pub fn new(config: VueConfig) -> Self {
        let renderer = VueRenderer::new(config.clone());
        Self { renderer, config }
    }

    /// Render a Vue page for the given request
    pub async fn render(&self, req: &HttpRequest) -> Result<HttpResponse, Error> {
        // Extract URL from request
        let url = &req.path;

        // Extract context from query params or body
        let context = self.extract_context(req)?;

        // Render the page
        let html = self.renderer.render(url, context).await?;

        // Build response
        let mut headers = HashMap::new();
        headers.insert(
            "Content-Type".to_string(),
            "text/html; charset=utf-8".to_string(),
        );

        if self.config.compression {
            headers.insert("X-Compression-Enabled".to_string(), "true".to_string());
        }

        Ok(HttpResponse {
            status: 200,
            headers,
            body: html.into_bytes(),
        })
    }

    /// Render with pre-fetched data
    pub async fn render_with_data(
        &self,
        url: &str,
        data: serde_json::Value,
    ) -> Result<HttpResponse, Error> {
        let html = self.renderer.render_with_data(url, data).await?;

        let mut headers = HashMap::new();
        headers.insert(
            "Content-Type".to_string(),
            "text/html; charset=utf-8".to_string(),
        );

        Ok(HttpResponse {
            status: 200,
            headers,
            body: html.into_bytes(),
        })
    }

    /// Serve static files
    pub async fn serve_static(&self, path: &str) -> Result<HttpResponse, Error> {
        // Remove leading slash
        let path = path.trim_start_matches('/');

        // Build full path
        let file_path = self.config.static_dir.join(path);

        // Security check: ensure path is within static directory
        let canonical_static = self
            .config
            .static_dir
            .canonicalize()
            .map_err(|_| Error::NotFound("Static directory not found".to_string()))?;

        let canonical_file = file_path
            .canonicalize()
            .map_err(|_| Error::NotFound("File not found".to_string()))?;

        if !canonical_file.starts_with(&canonical_static) {
            return Err(Error::Forbidden("Access denied".to_string()));
        }

        // Read file
        let content = tokio::fs::read(&file_path)
            .await
            .map_err(|_| Error::NotFound(format!("File not found: {}", path)))?;

        // Determine content type
        let content_type = self.guess_content_type(&file_path);

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), content_type);
        headers.insert(
            "Cache-Control".to_string(),
            "public, max-age=31536000, immutable".to_string(),
        );

        Ok(HttpResponse {
            status: 200,
            headers,
            body: content,
        })
    }

    /// Check service health
    pub async fn health_check(&self) -> Result<(), Error> {
        self.renderer.health_check().await
    }

    /// Extract context from request
    fn extract_context(&self, req: &HttpRequest) -> Result<Option<serde_json::Value>, Error> {
        // Try to parse body as JSON
        if !req.body.is_empty() {
            if let Ok(context) = serde_json::from_slice(&req.body) {
                return Ok(Some(context));
            }
        }

        // Could also extract from query params
        if !req.query_params.is_empty() {
            let context = serde_json::to_value(&req.query_params)
                .map_err(|e| Error::Serialization(e.to_string()))?;
            return Ok(Some(context));
        }

        Ok(None)
    }

    /// Guess content type from file extension
    fn guess_content_type(&self, path: &Path) -> String {
        match path.extension().and_then(|e| e.to_str()) {
            Some("html") => "text/html; charset=utf-8",
            Some("css") => "text/css; charset=utf-8",
            Some("js") | Some("mjs") => "application/javascript; charset=utf-8",
            Some("json") => "application/json",
            Some("png") => "image/png",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("gif") => "image/gif",
            Some("svg") => "image/svg+xml",
            Some("ico") => "image/x-icon",
            Some("woff") => "font/woff",
            Some("woff2") => "font/woff2",
            Some("ttf") => "font/ttf",
            Some("eot") => "application/vnd.ms-fontobject",
            Some("webp") => "image/webp",
            Some("mp4") => "video/mp4",
            Some("webm") => "video/webm",
            Some("map") => "application/json",
            Some("vue") => "text/plain; charset=utf-8",
            _ => "application/octet-stream",
        }
        .to_string()
    }
}

impl Default for VueService {
    fn default() -> Self {
        Self::new(VueConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_service_creation() {
        let config = VueConfig::new(PathBuf::from("dist"));
        let _service = VueService::new(config);
    }

    #[test]
    fn test_content_type_detection() {
        let config = VueConfig::new(PathBuf::from("dist"));
        let service = VueService::new(config);

        assert_eq!(
            service.guess_content_type(Path::new("app.js")),
            "application/javascript; charset=utf-8"
        );
        assert_eq!(
            service.guess_content_type(Path::new("style.css")),
            "text/css; charset=utf-8"
        );
        assert_eq!(
            service.guess_content_type(Path::new("logo.png")),
            "image/png"
        );
        assert_eq!(
            service.guess_content_type(Path::new("component.vue")),
            "text/plain; charset=utf-8"
        );
    }
}
