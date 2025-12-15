//! Azure Functions configuration.

/// Azure Functions configuration.
#[derive(Debug, Clone)]
pub struct FunctionConfig {
    /// Function app name.
    pub app_name: Option<String>,
    /// Function name.
    pub function_name: Option<String>,
    /// Environment (Development, Staging, Production).
    pub environment: String,
    /// Application Insights connection string.
    pub app_insights_connection: Option<String>,
    /// Storage connection string.
    pub storage_connection: Option<String>,
    /// Custom base path for routing.
    pub base_path: Option<String>,
    /// Maximum request body size in bytes.
    pub max_request_size: usize,
    /// Request timeout in seconds.
    pub timeout_seconds: u64,
}

impl Default for FunctionConfig {
    fn default() -> Self {
        Self {
            app_name: None,
            function_name: None,
            environment: "Development".to_string(),
            app_insights_connection: None,
            storage_connection: None,
            base_path: None,
            max_request_size: 100 * 1024 * 1024, // 100 MB
            timeout_seconds: 230, // Azure Functions default timeout
        }
    }
}

impl FunctionConfig {
    /// Create configuration from environment variables.
    pub fn from_env() -> Self {
        Self {
            app_name: std::env::var("WEBSITE_SITE_NAME").ok(),
            function_name: std::env::var("AZURE_FUNCTIONS_FUNCTION_NAME").ok(),
            environment: std::env::var("AZURE_FUNCTIONS_ENVIRONMENT")
                .unwrap_or_else(|_| "Development".to_string()),
            app_insights_connection: std::env::var("APPLICATIONINSIGHTS_CONNECTION_STRING").ok(),
            storage_connection: std::env::var("AzureWebJobsStorage").ok(),
            base_path: std::env::var("FUNCTION_BASE_PATH").ok(),
            max_request_size: std::env::var("MAX_REQUEST_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(100 * 1024 * 1024),
            timeout_seconds: std::env::var("FUNCTIONS_REQUEST_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(230),
        }
    }

    /// Set the base path.
    pub fn base_path(mut self, path: impl Into<String>) -> Self {
        self.base_path = Some(path.into());
        self
    }

    /// Set max request size.
    pub fn max_request_size(mut self, size: usize) -> Self {
        self.max_request_size = size;
        self
    }

    /// Set request timeout.
    pub fn timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self
    }

    /// Check if running in production.
    pub fn is_production(&self) -> bool {
        self.environment.eq_ignore_ascii_case("production")
    }

    /// Check if running in Azure Functions.
    pub fn is_azure_functions(&self) -> bool {
        self.app_name.is_some() || std::env::var("FUNCTIONS_WORKER_RUNTIME").is_ok()
    }
}

