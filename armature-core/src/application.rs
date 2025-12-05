// Application bootstrapper and HTTP server

use crate::{Container, Error, HttpRequest, HttpResponse, Module, Router};
use http_body_util::{BodyExt, Full};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, body::Incoming as IncomingBody};
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

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

    /// Get a reference to the DI container
    pub fn container(&self) -> &Container {
        &self.container
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
