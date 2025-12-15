//! gRPC server implementation.

use std::future::Future;
use tonic::transport::Server;
use tracing::{info, error};

use crate::{GrpcServerConfig, GrpcError, Result};

/// Type alias for the body type used by tonic.
type TonicBody = http_body_util::combinators::UnsyncBoxBody<bytes::Bytes, tonic::Status>;

/// gRPC server builder.
pub struct GrpcServerBuilder {
    config: GrpcServerConfig,
}

impl GrpcServerBuilder {
    /// Create a new server builder.
    pub fn new(config: GrpcServerConfig) -> Self {
        Self { config }
    }

    /// Build and start the server with a service.
    pub async fn serve<S>(self, service: S) -> Result<()>
    where
        S: tower::Service<
                http::Request<tonic::body::Body>,
                Response = http::Response<TonicBody>,
                Error = std::convert::Infallible,
            > + tonic::server::NamedService
            + Clone
            + Send
            + Sync
            + 'static,
        S::Future: Send + 'static,
    {
        let addr = self.config.bind_address;

        info!(address = %addr, "Starting gRPC server");

        let mut builder = Server::builder()
            .tcp_nodelay(self.config.tcp_nodelay);

        if let Some(interval) = self.config.http2_keepalive_interval {
            builder = builder.http2_keepalive_interval(Some(interval));
        }
        if let Some(timeout) = self.config.http2_keepalive_timeout {
            builder = builder.http2_keepalive_timeout(Some(timeout));
        }
        if let Some(window) = self.config.initial_connection_window_size {
            builder = builder.initial_connection_window_size(window);
        }
        if let Some(window) = self.config.initial_stream_window_size {
            builder = builder.initial_stream_window_size(window);
        }
        if let Some(limit) = self.config.concurrency_limit_per_connection {
            builder = builder.concurrency_limit_per_connection(limit);
        }

        let router = builder.add_service(service);

        // Add health check service
        #[cfg(feature = "health")]
        let router = if self.config.enable_health_check {
            let (_health_reporter, health_service) = tonic_health::server::health_reporter();
            router.add_service(health_service)
        } else {
            router
        };

        router
            .serve(addr)
            .await
            .map_err(|e| {
                error!(error = %e, "gRPC server error");
                GrpcError::Server(e.to_string())
            })
    }

    /// Build and start the server with graceful shutdown.
    pub async fn serve_with_shutdown<S, F>(self, service: S, signal: F) -> Result<()>
    where
        S: tower::Service<
                http::Request<tonic::body::Body>,
                Response = http::Response<TonicBody>,
                Error = std::convert::Infallible,
            > + tonic::server::NamedService
            + Clone
            + Send
            + Sync
            + 'static,
        S::Future: Send + 'static,
        F: Future<Output = ()> + Send + 'static,
    {
        let addr = self.config.bind_address;

        info!(address = %addr, "Starting gRPC server with graceful shutdown");

        let mut builder = Server::builder()
            .tcp_nodelay(self.config.tcp_nodelay);

        if let Some(interval) = self.config.http2_keepalive_interval {
            builder = builder.http2_keepalive_interval(Some(interval));
        }
        if let Some(timeout) = self.config.http2_keepalive_timeout {
            builder = builder.http2_keepalive_timeout(Some(timeout));
        }

        let router = builder.add_service(service);

        router
            .serve_with_shutdown(addr, signal)
            .await
            .map_err(|e| {
                error!(error = %e, "gRPC server error");
                GrpcError::Server(e.to_string())
            })
    }
}

/// gRPC server wrapper.
pub struct GrpcServer;

impl GrpcServer {
    /// Create a server builder with the given configuration.
    pub fn builder(config: GrpcServerConfig) -> GrpcServerBuilder {
        GrpcServerBuilder::new(config)
    }

    /// Create a server builder with default configuration.
    pub fn with_default_config() -> GrpcServerBuilder {
        GrpcServerBuilder::new(GrpcServerConfig::default())
    }

    /// Create a server builder bound to the specified address.
    pub fn bind(addr: impl Into<String>) -> GrpcServerBuilder {
        let config = GrpcServerConfig::builder()
            .bind_address(addr)
            .build();
        GrpcServerBuilder::new(config)
    }
}
