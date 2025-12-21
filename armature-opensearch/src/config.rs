//! OpenSearch client configuration.

use std::time::Duration;

/// OpenSearch client configuration.
#[derive(Debug, Clone)]
pub struct OpenSearchConfig {
    /// OpenSearch URL(s).
    pub urls: Vec<String>,
    /// Basic auth username.
    pub username: Option<String>,
    /// Basic auth password.
    pub password: Option<String>,
    /// Connection timeout.
    pub connect_timeout: Duration,
    /// Request timeout.
    pub request_timeout: Duration,
    /// TLS configuration.
    pub tls: Option<TlsConfig>,
    /// AWS region (for AWS OpenSearch Service).
    #[cfg(feature = "aws-auth")]
    pub aws_region: Option<String>,
    /// Enable request compression.
    pub compression: bool,
    /// Maximum number of retries.
    pub max_retries: u32,
}

impl OpenSearchConfig {
    /// Create a new configuration with a single URL.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            urls: vec![url.into()],
            username: None,
            password: None,
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(30),
            tls: None,
            #[cfg(feature = "aws-auth")]
            aws_region: None,
            compression: true,
            max_retries: 3,
        }
    }

    /// Create configuration with multiple URLs for a cluster.
    pub fn cluster(urls: Vec<String>) -> Self {
        Self {
            urls,
            ..Self::new("")
        }
    }

    /// Set basic authentication credentials.
    pub fn with_basic_auth(
        mut self,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        self.username = Some(username.into());
        self.password = Some(password.into());
        self
    }

    /// Set connection timeout.
    pub fn with_connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// Set request timeout.
    pub fn with_request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }

    /// Set TLS configuration.
    pub fn with_tls(mut self, tls: TlsConfig) -> Self {
        self.tls = Some(tls);
        self
    }

    /// Set AWS region for AWS OpenSearch Service.
    #[cfg(feature = "aws-auth")]
    pub fn with_aws_region(mut self, region: impl Into<String>) -> Self {
        self.aws_region = Some(region.into());
        self
    }

    /// Enable or disable compression.
    pub fn with_compression(mut self, enabled: bool) -> Self {
        self.compression = enabled;
        self
    }

    /// Set maximum retries.
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }
}

/// TLS configuration.
#[derive(Debug, Clone, Default)]
pub struct TlsConfig {
    /// Path to CA certificate.
    pub ca_cert: Option<String>,
    /// Path to client certificate.
    pub client_cert: Option<String>,
    /// Path to client key.
    pub client_key: Option<String>,
    /// Skip certificate verification (not recommended for production).
    pub danger_accept_invalid_certs: bool,
}

impl TlsConfig {
    /// Create TLS config with CA certificate.
    pub fn with_ca_cert(ca_cert: impl Into<String>) -> Self {
        Self {
            ca_cert: Some(ca_cert.into()),
            ..Default::default()
        }
    }

    /// Set client certificate and key for mutual TLS.
    pub fn with_client_cert(mut self, cert: impl Into<String>, key: impl Into<String>) -> Self {
        self.client_cert = Some(cert.into());
        self.client_key = Some(key.into());
        self
    }

    /// Skip certificate verification (DANGER: only for development).
    pub fn danger_accept_invalid_certs(mut self) -> Self {
        self.danger_accept_invalid_certs = true;
        self
    }
}
