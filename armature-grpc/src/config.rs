//! gRPC configuration types.

use std::net::SocketAddr;
use std::time::Duration;

/// gRPC server configuration.
#[derive(Debug, Clone)]
pub struct GrpcServerConfig {
    /// Address to bind to.
    pub bind_address: SocketAddr,
    /// Maximum message size for receiving.
    pub max_recv_message_size: usize,
    /// Maximum message size for sending.
    pub max_send_message_size: usize,
    /// Enable HTTP/2 keepalive.
    pub http2_keepalive_interval: Option<Duration>,
    /// HTTP/2 keepalive timeout.
    pub http2_keepalive_timeout: Option<Duration>,
    /// TCP keepalive.
    pub tcp_keepalive: Option<Duration>,
    /// TCP nodelay.
    pub tcp_nodelay: bool,
    /// Enable gRPC health checking service.
    pub enable_health_check: bool,
    /// Enable server reflection.
    pub enable_reflection: bool,
    /// Concurrency limit per connection.
    pub concurrency_limit_per_connection: Option<usize>,
    /// Initial connection window size.
    pub initial_connection_window_size: Option<u32>,
    /// Initial stream window size.
    pub initial_stream_window_size: Option<u32>,
}

impl Default for GrpcServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0:50051".parse().unwrap(),
            max_recv_message_size: 4 * 1024 * 1024, // 4 MB
            max_send_message_size: 4 * 1024 * 1024, // 4 MB
            http2_keepalive_interval: Some(Duration::from_secs(60)),
            http2_keepalive_timeout: Some(Duration::from_secs(20)),
            tcp_keepalive: Some(Duration::from_secs(60)),
            tcp_nodelay: true,
            enable_health_check: true,
            enable_reflection: false,
            concurrency_limit_per_connection: None,
            initial_connection_window_size: None,
            initial_stream_window_size: None,
        }
    }
}

impl GrpcServerConfig {
    /// Create a new builder.
    pub fn builder() -> GrpcServerConfigBuilder {
        GrpcServerConfigBuilder::default()
    }
}

/// Builder for gRPC server configuration.
#[derive(Debug, Default)]
pub struct GrpcServerConfigBuilder {
    config: GrpcServerConfig,
}

impl GrpcServerConfigBuilder {
    /// Set the bind address.
    pub fn bind_address(mut self, addr: impl Into<String>) -> Self {
        self.config.bind_address = addr.into().parse().unwrap();
        self
    }

    /// Set the bind address from a SocketAddr.
    pub fn bind_socket_addr(mut self, addr: SocketAddr) -> Self {
        self.config.bind_address = addr;
        self
    }

    /// Set the maximum receive message size.
    pub fn max_recv_message_size(mut self, size: usize) -> Self {
        self.config.max_recv_message_size = size;
        self
    }

    /// Set the maximum send message size.
    pub fn max_send_message_size(mut self, size: usize) -> Self {
        self.config.max_send_message_size = size;
        self
    }

    /// Enable HTTP/2 keepalive.
    pub fn http2_keepalive(mut self, interval: Duration, timeout: Duration) -> Self {
        self.config.http2_keepalive_interval = Some(interval);
        self.config.http2_keepalive_timeout = Some(timeout);
        self
    }

    /// Set TCP keepalive.
    pub fn tcp_keepalive(mut self, duration: Duration) -> Self {
        self.config.tcp_keepalive = Some(duration);
        self
    }

    /// Enable TCP nodelay.
    pub fn tcp_nodelay(mut self, enable: bool) -> Self {
        self.config.tcp_nodelay = enable;
        self
    }

    /// Enable gRPC health checking.
    pub fn enable_health_check(mut self) -> Self {
        self.config.enable_health_check = true;
        self
    }

    /// Enable server reflection.
    pub fn enable_reflection(mut self) -> Self {
        self.config.enable_reflection = true;
        self
    }

    /// Set concurrency limit per connection.
    pub fn concurrency_limit(mut self, limit: usize) -> Self {
        self.config.concurrency_limit_per_connection = Some(limit);
        self
    }

    /// Build the configuration.
    pub fn build(self) -> GrpcServerConfig {
        self.config
    }
}

/// gRPC client configuration.
#[derive(Debug, Clone)]
pub struct GrpcClientConfig {
    /// Server endpoint URL.
    pub endpoint: String,
    /// Request timeout.
    pub timeout: Duration,
    /// Connect timeout.
    pub connect_timeout: Duration,
    /// Enable HTTP/2 keepalive.
    pub http2_keepalive_interval: Option<Duration>,
    /// HTTP/2 keepalive timeout.
    pub http2_keepalive_timeout: Option<Duration>,
    /// TCP keepalive.
    pub tcp_keepalive: Option<Duration>,
    /// TCP nodelay.
    pub tcp_nodelay: bool,
    /// Concurrency limit.
    pub concurrency_limit: Option<usize>,
    /// Rate limit (requests per second).
    pub rate_limit: Option<u64>,
    /// Initial connection window size.
    pub initial_connection_window_size: Option<u32>,
    /// Initial stream window size.
    pub initial_stream_window_size: Option<u32>,
    /// Enable retry.
    pub retry_enabled: bool,
    /// Maximum retry attempts.
    pub max_retry_attempts: u32,
}

impl Default for GrpcClientConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:50051".to_string(),
            timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            http2_keepalive_interval: Some(Duration::from_secs(60)),
            http2_keepalive_timeout: Some(Duration::from_secs(20)),
            tcp_keepalive: Some(Duration::from_secs(60)),
            tcp_nodelay: true,
            concurrency_limit: None,
            rate_limit: None,
            initial_connection_window_size: None,
            initial_stream_window_size: None,
            retry_enabled: true,
            max_retry_attempts: 3,
        }
    }
}

impl GrpcClientConfig {
    /// Create a new builder.
    pub fn builder() -> GrpcClientConfigBuilder {
        GrpcClientConfigBuilder::default()
    }
}

/// Builder for gRPC client configuration.
#[derive(Debug, Default)]
pub struct GrpcClientConfigBuilder {
    config: GrpcClientConfig,
}

impl GrpcClientConfigBuilder {
    /// Set the endpoint URL.
    pub fn endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.config.endpoint = endpoint.into();
        self
    }

    /// Set the request timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }

    /// Set the connect timeout.
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.config.connect_timeout = timeout;
        self
    }

    /// Enable HTTP/2 keepalive.
    pub fn http2_keepalive(mut self, interval: Duration, timeout: Duration) -> Self {
        self.config.http2_keepalive_interval = Some(interval);
        self.config.http2_keepalive_timeout = Some(timeout);
        self
    }

    /// Set TCP keepalive.
    pub fn tcp_keepalive(mut self, duration: Duration) -> Self {
        self.config.tcp_keepalive = Some(duration);
        self
    }

    /// Enable TCP nodelay.
    pub fn tcp_nodelay(mut self, enable: bool) -> Self {
        self.config.tcp_nodelay = enable;
        self
    }

    /// Set concurrency limit.
    pub fn concurrency_limit(mut self, limit: usize) -> Self {
        self.config.concurrency_limit = Some(limit);
        self
    }

    /// Set rate limit (requests per second).
    pub fn rate_limit(mut self, rps: u64) -> Self {
        self.config.rate_limit = Some(rps);
        self
    }

    /// Enable or disable retry.
    pub fn retry(mut self, enabled: bool) -> Self {
        self.config.retry_enabled = enabled;
        self
    }

    /// Set maximum retry attempts.
    pub fn max_retry_attempts(mut self, attempts: u32) -> Self {
        self.config.max_retry_attempts = attempts;
        self
    }

    /// Build the configuration.
    pub fn build(self) -> GrpcClientConfig {
        self.config
    }
}
