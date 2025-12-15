//! HTTP client configuration.

use std::time::Duration;
use crate::retry::RetryConfig;
use crate::circuit_breaker::CircuitBreakerConfig;

/// HTTP client configuration.
#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    /// Base URL for all requests.
    pub base_url: Option<String>,
    /// Default request timeout.
    pub timeout: Duration,
    /// Connection timeout.
    pub connect_timeout: Duration,
    /// Retry configuration.
    pub retry: Option<RetryConfig>,
    /// Circuit breaker configuration.
    pub circuit_breaker: Option<CircuitBreakerConfig>,
    /// Maximum number of idle connections per host.
    pub pool_idle_timeout: Duration,
    /// Maximum idle connections.
    pub pool_max_idle_per_host: usize,
    /// Default headers for all requests.
    pub default_headers: Vec<(String, String)>,
    /// User agent string.
    pub user_agent: String,
    /// Enable gzip compression.
    pub gzip: bool,
    /// Enable brotli compression.
    pub brotli: bool,
    /// Follow redirects.
    pub follow_redirects: bool,
    /// Maximum redirects to follow.
    pub max_redirects: usize,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            base_url: None,
            timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            retry: None,
            circuit_breaker: None,
            pool_idle_timeout: Duration::from_secs(90),
            pool_max_idle_per_host: 32,
            default_headers: Vec::new(),
            user_agent: format!("armature-http-client/{}", env!("CARGO_PKG_VERSION")),
            gzip: true,
            brotli: true,
            follow_redirects: true,
            max_redirects: 10,
        }
    }
}

impl HttpClientConfig {
    /// Create a new configuration builder.
    pub fn builder() -> HttpClientConfigBuilder {
        HttpClientConfigBuilder::default()
    }
}

/// Builder for HTTP client configuration.
#[derive(Debug, Default)]
pub struct HttpClientConfigBuilder {
    config: HttpClientConfig,
}

impl HttpClientConfigBuilder {
    /// Set the base URL for all requests.
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.config.base_url = Some(url.into());
        self
    }

    /// Set the default request timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }

    /// Set the connection timeout.
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.config.connect_timeout = timeout;
        self
    }

    /// Set retry configuration.
    pub fn retry(mut self, config: RetryConfig) -> Self {
        self.config.retry = Some(config);
        self
    }

    /// Set circuit breaker configuration.
    pub fn circuit_breaker(mut self, config: CircuitBreakerConfig) -> Self {
        self.config.circuit_breaker = Some(config);
        self
    }

    /// Set the connection pool idle timeout.
    pub fn pool_idle_timeout(mut self, timeout: Duration) -> Self {
        self.config.pool_idle_timeout = timeout;
        self
    }

    /// Set the maximum idle connections per host.
    pub fn pool_max_idle_per_host(mut self, max: usize) -> Self {
        self.config.pool_max_idle_per_host = max;
        self
    }

    /// Add a default header for all requests.
    pub fn default_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.default_headers.push((name.into(), value.into()));
        self
    }

    /// Set the user agent string.
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.config.user_agent = user_agent.into();
        self
    }

    /// Enable or disable gzip compression.
    pub fn gzip(mut self, enable: bool) -> Self {
        self.config.gzip = enable;
        self
    }

    /// Enable or disable brotli compression.
    pub fn brotli(mut self, enable: bool) -> Self {
        self.config.brotli = enable;
        self
    }

    /// Enable or disable following redirects.
    pub fn follow_redirects(mut self, enable: bool) -> Self {
        self.config.follow_redirects = enable;
        self
    }

    /// Set the maximum number of redirects to follow.
    pub fn max_redirects(mut self, max: usize) -> Self {
        self.config.max_redirects = max;
        self
    }

    /// Build the configuration.
    pub fn build(self) -> HttpClientConfig {
        self.config
    }
}

