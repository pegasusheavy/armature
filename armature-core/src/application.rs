// Application bootstrapper and HTTP server

use crate::logging::{debug, error, info, trace, warn};
use crate::{
    Container, Error, HttpRequest, HttpResponse, HttpsConfig, LifecycleManager, Module, Router,
    TlsConfig,
};
use http_body_util::{BodyExt, Full};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, body::Incoming as IncomingBody};
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

/// The main application struct
pub struct Application {
    pub container: Container,
    pub router: Arc<Router>,
    pub lifecycle: Arc<LifecycleManager>,
}

impl Application {
    /// Create an application with a container and router
    pub fn new(container: Container, router: Router) -> Self {
        Self {
            container,
            router: Arc::new(router),
            lifecycle: Arc::new(LifecycleManager::new()),
        }
    }

    /// Create a new application from a root module with lifecycle support
    pub async fn create<M: Module + Default>() -> Self {
        info!("Bootstrapping Armature application");
        debug!(
            module_type = std::any::type_name::<M>(),
            "Creating application from root module"
        );

        let container = Container::new();
        debug!("DI container initialized");

        let mut router = Router::new();
        debug!("Router initialized");

        let lifecycle = Arc::new(LifecycleManager::new());
        debug!("Lifecycle manager initialized");

        // Initialize the root module
        let root_module = M::default();
        debug!("Root module instantiated");

        info!("Registering modules and dependencies");

        // Register all providers and controllers from the module tree
        Self::register_module(&container, &mut router, &root_module);

        info!("Executing lifecycle hooks");

        // Call module init hooks
        debug!("Calling OnModuleInit hooks");
        if let Err(errors) = lifecycle.call_module_init_hooks().await {
            warn!(error_count = errors.len(), "Some module init hooks failed");
            for (name, error) in errors {
                error!(hook_name = %name, error = %error, "Module init hook failed");
            }
        } else {
            debug!("All OnModuleInit hooks completed successfully");
        }

        // Call bootstrap hooks
        debug!("Calling OnApplicationBootstrap hooks");
        if let Err(errors) = lifecycle.call_bootstrap_hooks().await {
            warn!(error_count = errors.len(), "Some bootstrap hooks failed");
            for (name, error) in errors {
                error!(hook_name = %name, error = %error, "Bootstrap hook failed");
            }
        } else {
            debug!("All OnApplicationBootstrap hooks completed successfully");
        }

        info!("Application bootstrap complete");

        Self {
            container,
            router: Arc::new(router),
            lifecycle,
        }
    }

    /// Get a reference to the lifecycle manager
    pub fn lifecycle(&self) -> &Arc<LifecycleManager> {
        &self.lifecycle
    }

    /// Gracefully shutdown the application
    pub async fn shutdown(
        &self,
        signal: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!(signal = ?signal, "Gracefully shutting down application");

        // Call before shutdown hooks
        debug!("Calling BeforeApplicationShutdown hooks");
        if let Err(errors) = self
            .lifecycle
            .call_before_shutdown_hooks(signal.clone())
            .await
        {
            warn!(
                error_count = errors.len(),
                "Some before shutdown hooks failed"
            );
            for (name, error) in errors {
                error!(hook_name = %name, error = %error, "Before shutdown hook failed");
            }
        } else {
            debug!("All BeforeApplicationShutdown hooks completed successfully");
        }

        // Call shutdown hooks
        debug!("Calling OnApplicationShutdown hooks");
        if let Err(errors) = self.lifecycle.call_shutdown_hooks(signal.clone()).await {
            warn!(error_count = errors.len(), "Some shutdown hooks failed");
            for (name, error) in errors {
                error!(hook_name = %name, error = %error, "Shutdown hook failed");
            }
        } else {
            debug!("All OnApplicationShutdown hooks completed successfully");
        }

        // Call module destroy hooks
        debug!("Calling OnModuleDestroy hooks");
        if let Err(errors) = self.lifecycle.call_module_destroy_hooks().await {
            warn!(
                error_count = errors.len(),
                "Some module destroy hooks failed"
            );
            for (name, error) in errors {
                error!(hook_name = %name, error = %error, "Module destroy hook failed");
            }
        } else {
            debug!("All OnModuleDestroy hooks completed successfully");
        }

        info!("Application shutdown complete");
        Ok(())
    }

    /// Initialize logging with default configuration
    ///
    /// This is a convenience method that initializes JSON logging to STDOUT.
    /// For more control, use `LogConfig` directly.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use armature_core::Application;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let _guard = Application::init_logging();
    ///     // Application code...
    /// }
    /// ```
    pub fn init_logging() -> Option<crate::logging::tracing_appender::non_blocking::WorkerGuard> {
        crate::logging::LogConfig::default().init()
    }

    /// Initialize logging with custom configuration
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use armature_core::{Application, LogConfig, LogLevel, LogFormat};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let config = LogConfig::new()
    ///         .level(LogLevel::Debug)
    ///         .format(LogFormat::Pretty);
    ///
    ///     let _guard = Application::init_logging_with_config(config);
    ///     // Application code...
    /// }
    /// ```
    pub fn init_logging_with_config(
        config: crate::logging::LogConfig,
    ) -> Option<crate::logging::tracing_appender::non_blocking::WorkerGuard> {
        config.init()
    }

    /// Register a module and its imports recursively
    fn register_module(container: &Container, router: &mut Router, module: &dyn Module) {
        let module_type = std::any::type_name_of_val(module);
        debug!(module_type = module_type, "Registering module");

        // First, recursively register imported modules
        let imports = module.imports();
        if !imports.is_empty() {
            debug!(
                module_type = module_type,
                import_count = imports.len(),
                "Registering imported modules"
            );
            for imported_module in imports {
                Self::register_module(container, router, imported_module.as_ref());
            }
        }

        // Register all providers
        let providers = module.providers();
        debug!(
            module_type = module_type,
            provider_count = providers.len(),
            "Registering providers"
        );
        for provider_reg in providers {
            // Call the registration function which will register the provider in the container
            (provider_reg.register_fn)(container);
            debug!(
                module_type = module_type,
                provider = provider_reg.type_name,
                "Provider registered"
            );
        }

        // Register all controllers
        let controllers = module.controllers();
        debug!(
            module_type = module_type,
            controller_count = controllers.len(),
            "Registering controllers"
        );
        for controller_reg in controllers {
            // Instantiate controller with DI
            match (controller_reg.factory)(container) {
                Ok(controller_instance) => {
                    // Register routes for this controller
                    if let Err(e) =
                        (controller_reg.route_registrar)(container, router, controller_instance)
                    {
                        error!(
                            module_type = module_type,
                            controller = controller_reg.type_name,
                            error = %e,
                            "Failed to register routes for controller"
                        );
                    } else {
                        debug!(
                            module_type = module_type,
                            controller = controller_reg.type_name,
                            base_path = controller_reg.base_path,
                            "Controller registered"
                        );
                    }
                }
                Err(e) => {
                    error!(
                        module_type = module_type,
                        controller = controller_reg.type_name,
                        error = %e,
                        "Failed to instantiate controller"
                    );
                }
            }
        }

        debug!(module_type = module_type, "Module registration complete");
    }

    /// Start the HTTP server on the specified port
    pub async fn listen(self, port: u16) -> Result<(), Error> {
        let addr = SocketAddr::from(([0, 0, 0, 0], port));

        debug!(address = %addr, "Binding to address");
        let listener = TcpListener::bind(addr).await?;

        info!(address = %addr, "HTTP server listening");

        let router = self.router.clone();

        loop {
            let (stream, client_addr) = listener.accept().await?;
            trace!(client_address = %client_addr, "Connection accepted");

            let io = TokioIo::new(stream);
            let router = router.clone();

            tokio::spawn(async move {
                let service = service_fn(move |req: Request<IncomingBody>| {
                    let router = router.clone();
                    async move { handle_request(req, router).await }
                });

                if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                    error!(error = %err, client = %client_addr, "Error serving connection");
                }
            });
        }
    }

    /// Start the HTTPS server with TLS
    ///
    /// # Example
    ///
    /// ```ignore
    /// use armature_core::{Application, TlsConfig, Module};
    ///
    /// #[derive(Clone)]
    /// struct AppModule;
    /// impl Module for AppModule {
    ///     fn name(&self) -> &str { "AppModule" }
    ///     fn controllers(&self) -> Vec<Box<dyn Controller>> { vec![] }
    /// }
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut app = Application::new();
    /// let tls = TlsConfig::from_pem_files("cert.pem", "key.pem")?;
    /// app.listen_https(443, tls).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn listen_https(self, port: u16, tls_config: TlsConfig) -> Result<(), Error> {
        let addr = SocketAddr::from(([0, 0, 0, 0], port));

        debug!(address = %addr, "Binding to address (HTTPS)");
        let listener = TcpListener::bind(addr).await?;

        info!(address = %addr, "HTTPS server listening");

        let acceptor = TlsAcceptor::from(tls_config.server_config);
        let router = self.router.clone();

        loop {
            let (stream, client_addr) = listener.accept().await?;
            trace!(client_address = %client_addr, "HTTPS connection accepted");

            let acceptor = acceptor.clone();
            let router = router.clone();

            tokio::spawn(async move {
                match acceptor.accept(stream).await {
                    Ok(tls_stream) => {
                        debug!(client = %client_addr, "TLS handshake successful");
                        let io = TokioIo::new(tls_stream);

                        let service = service_fn(move |req: Request<IncomingBody>| {
                            let router = router.clone();
                            async move { handle_request(req, router).await }
                        });

                        if let Err(err) = http1::Builder::new().serve_connection(io, service).await
                        {
                            error!(error = %err, client = %client_addr, "Error serving HTTPS connection");
                        }
                    }
                    Err(err) => {
                        error!(error = %err, client = %client_addr, "TLS handshake failed");
                    }
                }
            });
        }
    }

    /// Start HTTPS server with optional HTTP to HTTPS redirect
    ///
    /// This method starts both an HTTPS server and optionally an HTTP server that redirects
    /// all traffic to HTTPS.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use armature_core::{Application, HttpsConfig, TlsConfig};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut app = Application::new();
    /// let tls = TlsConfig::from_pem_files("cert.pem", "key.pem")?;
    /// let https_config = HttpsConfig::new("0.0.0.0:443", tls)
    ///     .with_http_redirect("0.0.0.0:80");
    /// app.listen_with_config(https_config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn listen_with_config(self, config: HttpsConfig) -> Result<(), Error> {
        let router = self.router.clone();

        // Start HTTP redirect server if configured
        if let Some(ref http_addr) = config.http_redirect_addr {
            let https_port = config
                .https_addr
                .split(':')
                .next_back()
                .and_then(|p| p.parse::<u16>().ok())
                .unwrap_or(443);

            let http_addr = http_addr.clone();
            tokio::spawn(async move {
                if let Err(e) = start_http_redirect_server(&http_addr, https_port).await {
                    eprintln!("HTTP redirect server failed: {}", e);
                }
            });
        }

        // Parse HTTPS address
        let https_addr: SocketAddr = config
            .https_addr
            .parse()
            .map_err(|e| Error::Internal(format!("Invalid HTTPS address: {}", e)))?;

        let listener = TcpListener::bind(https_addr).await?;

        println!("🔒 HTTPS Server listening on https://{}", https_addr);
        if config.http_redirect_addr.is_some() {
            println!("↪️  HTTP redirect server enabled");
        }

        let acceptor = TlsAcceptor::from(config.tls.server_config);

        loop {
            let (stream, _) = listener.accept().await?;
            let acceptor = acceptor.clone();
            let router = router.clone();

            tokio::spawn(async move {
                match acceptor.accept(stream).await {
                    Ok(tls_stream) => {
                        let io = TokioIo::new(tls_stream);

                        let service = service_fn(move |req: Request<IncomingBody>| {
                            let router = router.clone();
                            async move { handle_request(req, router).await }
                        });

                        if let Err(err) = http1::Builder::new().serve_connection(io, service).await
                        {
                            eprintln!("Error serving HTTPS connection: {:?}", err);
                        }
                    }
                    Err(err) => {
                        eprintln!("TLS handshake failed: {:?}", err);
                    }
                }
            });
        }
    }

    /// Get a reference to the DI container
    pub fn container(&self) -> &Container {
        &self.container
    }
}

/// Start HTTP server that redirects all requests to HTTPS
async fn start_http_redirect_server(addr: &str, https_port: u16) -> Result<(), Error> {
    let addr: SocketAddr = addr
        .parse()
        .map_err(|e| Error::Internal(format!("Invalid HTTP redirect address: {}", e)))?;

    let listener = TcpListener::bind(addr).await?;

    println!("↪️  HTTP redirect server listening on http://{}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        tokio::spawn(async move {
            let service = service_fn(move |req: Request<IncomingBody>| async move {
                // Redirect to HTTPS
                let host = req
                    .headers()
                    .get("host")
                    .and_then(|h| h.to_str().ok())
                    .unwrap_or("localhost");

                // Remove port from host if present
                let host_without_port = host.split(':').next().unwrap_or(host);

                let location = if https_port == 443 {
                    format!("https://{}{}", host_without_port, req.uri().path())
                } else {
                    format!(
                        "https://{}:{}{}",
                        host_without_port,
                        https_port,
                        req.uri().path()
                    )
                };

                let response = Response::builder()
                    .status(301)
                    .header("Location", location)
                    .body(Full::new(bytes::Bytes::from("Redirecting to HTTPS...")))
                    .unwrap();

                Ok::<_, hyper::Error>(response)
            });

            if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                eprintln!("Error serving HTTP redirect: {:?}", err);
            }
        });
    }
}

/// Handle an incoming HTTP request
async fn handle_request(
    req: Request<IncomingBody>,
    router: Arc<Router>,
) -> Result<Response<Full<bytes::Bytes>>, hyper::Error> {
    use std::time::Instant;

    let start = Instant::now();

    // Convert hyper request to our HttpRequest
    let method = req.method().to_string();
    let path = req.uri().path().to_string();

    trace!(method = %method, path = %path, "Incoming request");

    let mut armature_req = HttpRequest::new(method.clone(), path.clone());

    // Copy headers
    let header_count = req.headers().len();
    for (name, value) in req.headers() {
        if let Ok(value_str) = value.to_str() {
            armature_req
                .headers
                .insert(name.to_string(), value_str.to_string());
        }
    }
    trace!(header_count = header_count, "Headers parsed");

    // Read body
    let body_bytes = req.collect().await?.to_bytes();
    let body_size = body_bytes.len();
    armature_req.body = body_bytes.to_vec();

    if body_size > 0 {
        trace!(body_size = body_size, "Request body received");
    }

    // Route the request
    debug!(method = %method, path = %path, "Routing request");
    let response = match router.route(armature_req).await {
        Ok(resp) => {
            debug!(method = %method, path = %path, status = resp.status, "Request handled successfully");
            resp
        }
        Err(err) => {
            warn!(method = %method, path = %path, error = %err, "Request handling failed");
            // Convert error to response
            let status = err.status_code();
            let body = serde_json::json!({
                "error": err.to_string(),
                "status": status,
            });
            HttpResponse::new(status)
                .with_json(&body)
                .unwrap_or_else(|_| HttpResponse::internal_server_error())
        }
    };

    let duration = start.elapsed();
    debug!(
        method = %method,
        path = %path,
        status = response.status,
        duration_ms = duration.as_millis(),
        "Request completed"
    );

    // Convert our HttpResponse to hyper Response
    let mut builder = Response::builder().status(response.status);

    for (key, value) in response.headers {
        builder = builder.header(key, value);
    }

    let body = Full::new(bytes::Bytes::from(response.body));
    Ok(builder.body(body).unwrap())
}
