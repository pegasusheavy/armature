//! gRPC client implementation.

use std::time::Duration;
use tonic::transport::{Channel, Endpoint};
use tracing::{debug, info};

use crate::{GrpcClientConfig, GrpcError, Result};

/// gRPC channel wrapper with configuration.
#[derive(Clone)]
pub struct GrpcChannel {
    inner: Channel,
    config: GrpcClientConfig,
}

impl GrpcChannel {
    /// Get the inner tonic channel.
    pub fn inner(&self) -> &Channel {
        &self.inner
    }

    /// Get the configuration.
    pub fn config(&self) -> &GrpcClientConfig {
        &self.config
    }
}

impl std::ops::Deref for GrpcChannel {
    type Target = Channel;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// gRPC client builder and connector.
pub struct GrpcClient;

impl GrpcClient {
    /// Connect to a gRPC server with the given configuration.
    pub async fn connect(config: GrpcClientConfig) -> Result<GrpcChannel> {
        info!(endpoint = %config.endpoint, "Connecting to gRPC server");

        let mut endpoint = Endpoint::from_shared(config.endpoint.clone())
            .map_err(|e| GrpcError::Config(e.to_string()))?
            .timeout(config.timeout)
            .connect_timeout(config.connect_timeout)
            .tcp_nodelay(config.tcp_nodelay);

        if let Some(interval) = config.http2_keepalive_interval {
            endpoint = endpoint.http2_keep_alive_interval(interval);
        }
        if let Some(timeout) = config.http2_keepalive_timeout {
            endpoint = endpoint.keep_alive_timeout(timeout);
        }
        if let Some(window) = config.initial_connection_window_size {
            endpoint = endpoint.initial_connection_window_size(window);
        }
        if let Some(window) = config.initial_stream_window_size {
            endpoint = endpoint.initial_stream_window_size(window);
        }
        if let Some(limit) = config.concurrency_limit {
            endpoint = endpoint.concurrency_limit(limit);
        }
        if let Some(rps) = config.rate_limit {
            endpoint = endpoint.rate_limit(rps, Duration::from_secs(1));
        }

        let channel = endpoint
            .connect()
            .await
            .map_err(GrpcError::Transport)?;

        debug!("gRPC client connected");

        Ok(GrpcChannel {
            inner: channel,
            config,
        })
    }

    /// Connect to a gRPC server with default configuration.
    pub async fn connect_default(endpoint: impl Into<String>) -> Result<GrpcChannel> {
        let config = GrpcClientConfig::builder()
            .endpoint(endpoint)
            .build();
        Self::connect(config).await
    }

    /// Create a lazy channel that connects on first use.
    pub fn lazy(config: GrpcClientConfig) -> Result<GrpcChannel> {
        info!(endpoint = %config.endpoint, "Creating lazy gRPC channel");

        let mut endpoint = Endpoint::from_shared(config.endpoint.clone())
            .map_err(|e| GrpcError::Config(e.to_string()))?
            .timeout(config.timeout)
            .connect_timeout(config.connect_timeout)
            .tcp_nodelay(config.tcp_nodelay);

        if let Some(interval) = config.http2_keepalive_interval {
            endpoint = endpoint.http2_keep_alive_interval(interval);
        }
        if let Some(timeout) = config.http2_keepalive_timeout {
            endpoint = endpoint.keep_alive_timeout(timeout);
        }
        if let Some(limit) = config.concurrency_limit {
            endpoint = endpoint.concurrency_limit(limit);
        }

        let channel = endpoint.connect_lazy();

        Ok(GrpcChannel {
            inner: channel,
            config,
        })
    }

    /// Create a channel with load balancing across multiple endpoints.
    pub async fn connect_balanced(
        endpoints: Vec<String>,
        config: GrpcClientConfig,
    ) -> Result<GrpcChannel> {
        info!(
            endpoints = ?endpoints,
            "Creating load-balanced gRPC channel"
        );

        let endpoints: Vec<Endpoint> = endpoints
            .into_iter()
            .map(|ep| {
                Endpoint::from_shared(ep)
                    .map(|e| {
                        e.timeout(config.timeout)
                            .connect_timeout(config.connect_timeout)
                            .tcp_nodelay(config.tcp_nodelay)
                    })
            })
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| GrpcError::Config(e.to_string()))?;

        let channel = Channel::balance_list(endpoints.into_iter());

        Ok(GrpcChannel {
            inner: channel,
            config,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_config() {
        let config = GrpcClientConfig::builder()
            .endpoint("http://localhost:50051")
            .timeout(Duration::from_secs(60))
            .build();

        assert_eq!(config.endpoint, "http://localhost:50051");
        assert_eq!(config.timeout, Duration::from_secs(60));
    }
}

