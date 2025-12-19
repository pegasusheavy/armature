//! API Playground - Interactive API Testing UI
//!
//! Provides an interactive web UI for testing APIs, similar to Swagger UI.

use armature_core::{Error, HttpRequest, HttpResponse};

/// API Playground configuration
#[derive(Debug, Clone)]
pub struct PlaygroundConfig {
    /// OpenAPI spec URL
    pub spec_url: String,

    /// Page title
    pub title: String,

    /// API base URL
    pub base_url: Option<String>,

    /// Enable authentication UI
    pub enable_auth: bool,

    /// Custom CSS
    pub custom_css: Option<String>,
}

impl PlaygroundConfig {
    /// Create new playground config
    ///
    /// # Examples
    ///
    /// ```
    /// use armature_openapi::PlaygroundConfig;
    ///
    /// let config = PlaygroundConfig::new("/api/openapi.json");
    /// ```
    pub fn new(spec_url: impl Into<String>) -> Self {
        Self {
            spec_url: spec_url.into(),
            title: "API Playground".to_string(),
            base_url: None,
            enable_auth: true,
            custom_css: None,
        }
    }

    /// Set page title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set API base URL
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Enable/disable authentication UI
    pub fn with_auth(mut self, enable: bool) -> Self {
        self.enable_auth = enable;
        self
    }

    /// Add custom CSS
    pub fn with_custom_css(mut self, css: impl Into<String>) -> Self {
        self.custom_css = Some(css.into());
        self
    }
}

impl Default for PlaygroundConfig {
    fn default() -> Self {
        Self::new("/api/openapi.json")
    }
}

/// Generate playground HTML
pub fn generate_html(config: &PlaygroundConfig) -> String {
    let custom_css = config.custom_css.as_deref().unwrap_or("");
    let base_url = config.base_url.as_deref().unwrap_or("");
    let auth_section = if config.enable_auth {
        r#"
        <div class="auth-section">
            <h3>Auth</h3>
            <input type="text" id="authToken" placeholder="Bearer token (optional)">
            <button onclick="setAuth()">Set Token</button>
        </div>
        "#
    } else {
        ""
    };
    let auth_js = if config.enable_auth {
        r#"
        // Set auth token
        function setAuth() {
            authToken = document.getElementById('authToken').value;
            alert('Token set!');
        }
        "#
    } else {
        ""
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}

        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
            background: #fafafa;
            color: #333;
            line-height: 1.6;
        }}

        .container {{
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
        }}

        header {{
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 30px 0;
            margin-bottom: 30px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }}

        h1 {{
            font-size: 2.5em;
            margin-bottom: 10px;
        }}

        .subtitle {{
            opacity: 0.9;
            font-size: 1.1em;
        }}

        .auth-section {{
            background: white;
            padding: 20px;
            border-radius: 8px;
            margin-bottom: 30px;
            box-shadow: 0 2px 5px rgba(0,0,0,0.05);
        }}

        .auth-section h3 {{
            margin-bottom: 15px;
            color: #667eea;
        }}

        input[type="text"] {{
            width: 70%;
            padding: 10px;
            border: 2px solid #e0e0e0;
            border-radius: 4px;
            font-size: 14px;
            margin-right: 10px;
        }}

        button {{
            padding: 10px 20px;
            background: #667eea;
            color: white;
            border: none;
            border-radius: 4px;
            cursor: pointer;
            font-size: 14px;
            font-weight: 600;
            transition: background 0.3s;
        }}

        button:hover {{
            background: #5568d3;
        }}

        .endpoint {{
            background: white;
            border-radius: 8px;
            margin-bottom: 20px;
            overflow: hidden;
            box-shadow: 0 2px 5px rgba(0,0,0,0.05);
        }}

        .endpoint-header {{
            padding: 20px;
            cursor: pointer;
            display: flex;
            align-items: center;
            gap: 15px;
            transition: background 0.2s;
        }}

        .endpoint-header:hover {{
            background: #f5f5f5;
        }}

        .method {{
            padding: 6px 12px;
            border-radius: 4px;
            font-weight: bold;
            font-size: 12px;
            min-width: 60px;
            text-align: center;
        }}

        .method.get {{ background: #61affe; color: white; }}
        .method.post {{ background: #49cc90; color: white; }}
        .method.put {{ background: #fca130; color: white; }}
        .method.delete {{ background: #f93e3e; color: white; }}
        .method.patch {{ background: #50e3c2; color: white; }}

        .path {{
            font-family: 'Monaco', 'Courier New', monospace;
            font-size: 14px;
            flex: 1;
        }}

        .endpoint-body {{
            padding: 20px;
            border-top: 1px solid #e0e0e0;
            display: none;
            background: #f9f9f9;
        }}

        .endpoint-body.open {{
            display: block;
        }}

        .description {{
            margin-bottom: 20px;
            color: #666;
        }}

        .param-section {{
            margin-bottom: 20px;
        }}

        .param-section h4 {{
            margin-bottom: 10px;
            color: #667eea;
        }}

        .param {{
            margin-bottom: 10px;
        }}

        .param label {{
            display: block;
            margin-bottom: 5px;
            font-weight: 600;
        }}

        .param input, .param textarea {{
            width: 100%;
            padding: 8px;
            border: 2px solid #e0e0e0;
            border-radius: 4px;
            font-family: 'Monaco', 'Courier New', monospace;
            font-size: 13px;
        }}

        .param textarea {{
            min-height: 100px;
            resize: vertical;
        }}

        .try-button {{
            background: #49cc90;
            margin-top: 15px;
        }}

        .try-button:hover {{
            background: #3fb87f;
        }}

        .response {{
            margin-top: 20px;
            padding: 15px;
            background: #2d2d2d;
            color: #f8f8f8;
            border-radius: 4px;
            font-family: 'Monaco', 'Courier New', monospace;
            font-size: 13px;
            overflow-x: auto;
            display: none;
        }}

        .response.show {{
            display: block;
        }}

        .response-status {{
            margin-bottom: 10px;
            padding-bottom: 10px;
            border-bottom: 1px solid #444;
            font-weight: bold;
        }}

        .response-status.success {{ color: #49cc90; }}
        .response-status.error {{ color: #f93e3e; }}

        .loading {{
            display: none;
            text-align: center;
            padding: 20px;
            color: #667eea;
        }}

        .loading.show {{
            display: block;
        }}

        {custom_css}
    </style>
</head>
<body>
    <header>
        <div class="container">
            <h1>🚀 {title}</h1>
            <div class="subtitle">Interactive API Testing</div>
        </div>
    </header>

    <div class="container">
        {auth_section}

        <div id="endpoints"></div>
    </div>

    <script>
        const specUrl = '{spec_url}';
        const baseUrl = '{base_url}';
        let authToken = '';
        let spec = null;

        {auth_js}

        // Load OpenAPI spec
        async function loadSpec() {{
            try {{
                const response = await fetch(specUrl);
                spec = await response.json();
                renderEndpoints();
            }} catch (error) {{
                console.error('Failed to load OpenAPI spec:', error);
                document.getElementById('endpoints').innerHTML =
                    '<div class="endpoint"><div class="endpoint-header">❌ Failed to load API specification</div></div>';
            }}
        }}

        // Render endpoints
        function renderEndpoints() {{
            const container = document.getElementById('endpoints');
            container.innerHTML = '';

            if (!spec || !spec.paths) {{
                container.innerHTML = '<div class="endpoint"><div class="endpoint-header">No endpoints found</div></div>';
                return;
            }}

            Object.keys(spec.paths).forEach(path => {{
                const pathItem = spec.paths[path];
                Object.keys(pathItem).forEach(method => {{
                    if (['get', 'post', 'put', 'delete', 'patch'].includes(method)) {{
                        const operation = pathItem[method];
                        const endpointDiv = createEndpoint(method, path, operation);
                        container.appendChild(endpointDiv);
                    }}
                }});
            }});
        }}

        // Create endpoint element
        function createEndpoint(method, path, operation) {{
            const div = document.createElement('div');
            div.className = 'endpoint';

            const header = document.createElement('div');
            header.className = 'endpoint-header';
            header.innerHTML = `
                <span class="method ${{method}}">${{method.toUpperCase()}}</span>
                <span class="path">${{path}}</span>
            `;
            header.onclick = () => toggleEndpoint(div);

            const body = document.createElement('div');
            body.className = 'endpoint-body';
            body.innerHTML = createEndpointBody(method, path, operation);

            div.appendChild(header);
            div.appendChild(body);

            return div;
        }}

        // Create endpoint body
        function createEndpointBody(method, path, operation) {{
            let html = '';

            if (operation.description) {{
                html += `<div class="description">${{operation.description}}</div>`;
            }}

            // Parameters
            if (operation.parameters && operation.parameters.length > 0) {{
                html += '<div class="param-section"><h4>Parameters</h4>';
                operation.parameters.forEach(param => {{
                    const required = param.required ? ' (required)' : '';
                    html += `
                        <div class="param">
                            <label>${{param.name}}${{required}}</label>
                            <input type="text" id="param-${{param.name}}" placeholder="${{param.description || ''}}">
                        </div>
                    `;
                }});
                html += '</div>';
            }}

            // Request body
            if (operation.requestBody) {{
                html += `
                    <div class="param-section">
                        <h4>Request Body</h4>
                        <textarea id="body-${{method}}-${{path.replace(/\//g, '-')}}" placeholder='${{JSON.stringify({{ example: "data" }}, null, 2)}}'></textarea>
                    </div>
                `;
            }}

            html += `
                <button class="try-button" onclick="tryEndpoint('${{method}}', '${{path}}')">
                    Try it out
                </button>
                <div class="loading" id="loading-${{method}}-${{path.replace(/\//g, '-')}}">
                    Loading...
                </div>
                <div class="response" id="response-${{method}}-${{path.replace(/\//g, '-')}}"></div>
            `;

            return html;
        }}

        // Toggle endpoint visibility
        function toggleEndpoint(div) {{
            const body = div.querySelector('.endpoint-body');
            body.classList.toggle('open');
        }}

        // Try endpoint
        async function tryEndpoint(method, path) {{
            const responseId = `response-${{method}}-${{path.replace(/\//g, '-')}}`;
            const loadingId = `loading-${{method}}-${{path.replace(/\//g, '-')}}`;
            const responseDiv = document.getElementById(responseId);
            const loadingDiv = document.getElementById(loadingId);

            loadingDiv.classList.add('show');
            responseDiv.classList.remove('show');

            try {{
                // Build URL
                let url = (baseUrl || '') + path;

                // Add query parameters
                const params = document.querySelectorAll(`[id^="param-"]`);
                const queryParams = [];
                params.forEach(param => {{
                    if (param.value) {{
                        const name = param.id.replace('param-', '');
                        queryParams.push(`${{name}}=${{encodeURIComponent(param.value)}}`);
                    }}
                }});
                if (queryParams.length > 0) {{
                    url += '?' + queryParams.join('&');
                }}

                // Build request
                const options = {{
                    method: method.toUpperCase(),
                    headers: {{
                        'Content-Type': 'application/json'
                    }}
                }};

                if (authToken) {{
                    options.headers['Authorization'] = `Bearer ${{authToken}}`;
                }}

                // Add body for POST/PUT/PATCH
                if (['post', 'put', 'patch'].includes(method)) {{
                    const bodyField = document.getElementById(`body-${{method}}-${{path.replace(/\//g, '-')}}`);
                    if (bodyField && bodyField.value) {{
                        options.body = bodyField.value;
                    }}
                }}

                // Make request
                const response = await fetch(url, options);
                const data = await response.text();

                // Display response
                let jsonData;
                try {{
                    jsonData = JSON.parse(data);
                }} catch (e) {{
                    jsonData = data;
                }}

                const statusClass = response.ok ? 'success' : 'error';
                responseDiv.innerHTML = `
                    <div class="response-status ${{statusClass}}">
                        Status: ${{response.status}} ${{response.statusText}}
                    </div>
                    <pre>${{JSON.stringify(jsonData, null, 2)}}</pre>
                `;
                responseDiv.classList.add('show');
            }} catch (error) {{
                responseDiv.innerHTML = `
                    <div class="response-status error">Error</div>
                    <pre>${{error.message}}</pre>
                `;
                responseDiv.classList.add('show');
            }} finally {{
                loadingDiv.classList.remove('show');
            }}
        }}

        // Load spec on page load
        loadSpec();
    </script>
</body>
</html>"#,
        title = config.title,
        spec_url = config.spec_url,
        base_url = base_url,
        auth_section = auth_section,
        auth_js = auth_js,
        custom_css = custom_css,
    )
}

/// Serve playground HTML
pub fn serve_playground(
    config: &PlaygroundConfig,
) -> impl Fn(HttpRequest) -> Result<HttpResponse, Error> {
    let html = generate_html(config);

    move |_req: HttpRequest| {
        Ok(HttpResponse::ok()
            .with_header(
                "Content-Type".to_string(),
                "text/html; charset=utf-8".to_string(),
            )
            .with_body(html.clone().into_bytes()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playground_config() {
        let config = PlaygroundConfig::new("/api/openapi.json")
            .with_title("My API")
            .with_base_url("https://api.example.com")
            .with_auth(false);

        assert_eq!(config.spec_url, "/api/openapi.json");
        assert_eq!(config.title, "My API");
        assert_eq!(config.base_url, Some("https://api.example.com".to_string()));
        assert!(!config.enable_auth);
    }

    #[test]
    fn test_generate_html() {
        let config = PlaygroundConfig::default();
        let html = generate_html(&config);

        assert!(html.contains("API Playground"));
        assert!(html.contains("/api/openapi.json"));
        assert!(html.contains("<h3>Auth</h3>"));
    }

    #[test]
    fn test_generate_html_no_auth() {
        let config = PlaygroundConfig::new("/api/spec.json").with_auth(false);
        let html = generate_html(&config);

        assert!(!html.contains("<h3>Auth</h3>"));
        assert!(!html.contains("setAuth"));
    }
}
