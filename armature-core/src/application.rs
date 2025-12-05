// Application bootstrapper and HTTP server

use crate::{Container, Error, HttpRequest, HttpResponse, HttpsConfig, Module, Router, TlsConfig};
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
}

impl Application {
    /// Create an application with a container and router
    pub fn new(container: Container, router: Router) -> Self {
        Self {
            container,
            router: Arc::new(router),
        }
    }

    /// Create a new application from a root module
    pub fn create<M: Module + Default>() -> Self {
        println!("🚀 Bootstrapping Armature application...\n");

        let container = Container::new();
        let mut router = Router::new();

        // Initialize the root module
        let root_module = M::default();

        println!("📦 Registering modules and dependencies:");

        // Register all providers and controllers from the module tree
        Self::register_module(&container, &mut router, &root_module);

        println!("\n✅ Application bootstrap complete!\n");

        Self {
            container,
            router: Arc::new(router),
        }
    }

    /// Register a module and its imports recursively
    fn register_module(container: &Container, router: &mut Router, module: &dyn Module) {
        // First, recursively register imported modules
        for imported_module in module.imports() {
            Self::register_module(container, router, imported_module.as_ref());
        }

        // Register all providers
        for provider_reg in module.providers() {
            // Call the registration function which will register the provider in the container
            (provider_reg.register_fn)(container);
            println!("✓ Registered provider: {}", provider_reg.type_name);
        }

        // Register all controllers
        for controller_reg in module.controllers() {
            // Instantiate controller with DI
            match (controller_reg.factory)(container) {
                Ok(controller_instance) => {
                    // Register routes for this controller
                    if let Err(e) =
                        (controller_reg.route_registrar)(container, router, controller_instance)
                    {
                        eprintln!(
                            "Failed to register routes for {}: {}",
                            controller_reg.type_name, e
                        );
                    } else {
                        println!(
                            "Registered controller: {} at {}",
                            controller_reg.type_name, controller_reg.base_path
                        );
                    }
                }
                Err(e) => {
                    eprintln!(
                        "Failed to instantiate controller {}: {}",
                        controller_reg.type_name, e
                    );
                }
            }
        }
    }

    /// Start the HTTP server on the specified port
    pub async fn listen(self, port: u16) -> Result<(), Error> {
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        let listener = TcpListener::bind(addr).await?;

        println!("🚀 Server listening on http://{}", addr);

        let router = self.router.clone();

        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);
            let router = router.clone();

            tokio::spawn(async move {
                let service = service_fn(move |req: Request<IncomingBody>| {
                    let router = router.clone();
                    async move { handle_request(req, router).await }
                });

                if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                    eprintln!("Error serving connection: {:?}", err);
                }
            });
        }
    }

    /// Start the HTTPS server with TLS
    ///
    /// # Example
    ///
    /// ```no_run
    /// use armature_core::{Application, TlsConfig};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let app = Application::create::<MyModule>();
    /// let tls = TlsConfig::from_pem_files("cert.pem", "key.pem")?;
    /// app.listen_https(443, tls).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn listen_https(self, port: u16, tls_config: TlsConfig) -> Result<(), Error> {
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        let listener = TcpListener::bind(addr).await?;

        println!("🔒 HTTPS Server listening on https://{}", addr);

        let acceptor = TlsAcceptor::from(tls_config.server_config);
        let router = self.router.clone();

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

    /// Start HTTPS server with optional HTTP to HTTPS redirect
    ///
    /// This method starts both an HTTPS server and optionally an HTTP server that redirects
    /// all traffic to HTTPS.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use armature_core::{Application, HttpsConfig, TlsConfig};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let app = Application::create::<MyModule>();
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
                .last()
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
    // Convert hyper request to our HttpRequest
    let method = req.method().to_string();
    let path = req.uri().path().to_string();

    let mut armature_req = HttpRequest::new(method, path);

    // Copy headers
    for (name, value) in req.headers() {
        if let Ok(value_str) = value.to_str() {
            armature_req
                .headers
                .insert(name.to_string(), value_str.to_string());
        }
    }

    // Read body
    let body_bytes = req.collect().await?.to_bytes();
    armature_req.body = body_bytes.to_vec();

    // Route the request
    let response = match router.route(armature_req).await {
        Ok(resp) => resp,
        Err(err) => {
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

    // Convert our HttpResponse to hyper Response
    let mut builder = Response::builder().status(response.status);

    for (key, value) in response.headers {
        builder = builder.header(key, value);
    }

    let body = Full::new(bytes::Bytes::from(response.body));
    Ok(builder.body(body).unwrap())
}
