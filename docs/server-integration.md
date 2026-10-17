# Server Integration Guide

Integration strategies for using Armature with external web servers like NGINX and Ferron.

## Table of Contents

- [Current Architecture](#current-architecture)
- [Integration Patterns](#integration-patterns)
- [NGINX Integration](#nginx-integration)
- [Ferron Integration](#ferron-integration)
- [Pluggable Server Backend](#pluggable-server-backend)
- [Comparison](#comparison)
- [Recommendations](#recommendations)

---

## Current Architecture

Armature embeds its own HTTP server. HTTP/1.1 is served by `armature-h1` (the
default `h1-backend` feature); HTTP/2 and h2c by **Hyper**; HTTP/3 by `quinn` +
`h3` (the `http3` feature). Building with `default-features = false` serves
HTTP/1.1 through Hyper as well. Every transport runs the same pipeline: path
canonicalization, global middleware, guards, route-scoped middleware and
guards, CORS, body limits and the body read policy. Slow-client defences (10 s
head and TLS handshake deadlines, 30 s body inactivity / 1 KiB/s minimum body
rate) and a connection cap (`PipelineConfig::max_connections`, default 10 000)
are on by default; see the [Request Timeouts guide](request-timeouts-guide.md).

The hyper serve path looks like this:

```rust
// armature-core/src/application.rs
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, body::Incoming as IncomingBody};
```

**Benefits:**
- ✅ Zero-copy request handling
- ✅ Native async/await support
- ✅ Built-in HTTP/2 and HTTP/3 support
- ✅ No external dependencies
- ✅ Direct integration with Tokio

**Limitations:**
- ❌ No built-in load balancing
- ❌ No advanced routing/caching
- ❌ Limited static file optimization
- ❌ No zero-downtime reloads

---

## Integration Patterns

There are **three main approaches** to integrate external servers:

### 1. Reverse Proxy Pattern (Recommended)

```
┌─────────┐      ┌───────────┐      ┌───────────┐
│ Client  │─────▶│  NGINX/   │─────▶│ Armature  │
│         │      │  Ferron   │      │  (Hyper)  │
└─────────┘      └───────────┘      └───────────┘
                  Reverse Proxy      App Server
```

**Use Cases:**
- Load balancing across multiple Armature instances
- SSL/TLS termination
- Static asset serving
- Response caching
- Rate limiting
- DDoS protection

### 2. CGI/FastCGI/Module Pattern

```
┌─────────┐      ┌───────────────────────────┐
│ Client  │─────▶│  NGINX with Armature      │
│         │      │  as embedded module/CGI   │
└─────────┘      └───────────────────────────┘
```

**Use Cases:**
- Tight integration with web server
- Shared memory/cache
- Single process deployment

### 3. Pluggable Backend Pattern

```
┌───────────┐      ┌──────────────┐
│ Armature  │─────▶│   Backend    │
│   Core    │      │   Trait      │
└───────────┘      └──────────────┘
                          │
       ┌──────────────────┼──────────────────┐
       │                  │                  │
   ┌───▼───┐         ┌────▼────┐      ┌─────▼─────┐
   │ Hyper │         │  NGINX  │      │  Ferron   │
   └───────┘         └─────────┘      └───────────┘
```

**Use Cases:**
- Framework flexibility
- Testing different servers
- Custom server implementations

---

## NGINX Integration

### Approach 1: Reverse Proxy (Recommended)

#### Setup

**1. Armature Configuration:**

```rust
// main.rs
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Application::create::<AppModule>().await;

    // Listen on loopback only (NGINX will forward). `listen(port)` binds
    // 0.0.0.0, which would let clients bypass the proxy.
    app.listen_on(([127, 0, 0, 1], 3000)).await?;
    Ok(())
}
```

Behind a proxy, the socket peer is the proxy. Read the real client address with
`HttpRequest::client_address(1)` (one trusted proxy): it takes the right-most
`X-Forwarded-For` entry the proxy appended, across every `X-Forwarded-For` line.
`client_address(0)` (the default in `armature-ratelimit`, `armature-audit` and
`armature-analytics`) ignores the header and uses the socket peer. Armature
never reads `X-Real-IP`, and you should never trust the left-most
`X-Forwarded-For` entry, which the client controls. Set the depth to the exact
number of proxies in front of the app.

**2. NGINX Configuration:**

```nginx
# /etc/nginx/sites-available/armature

upstream armature_backend {
    # Multiple instances for load balancing
    server 127.0.0.1:3000 max_fails=3 fail_timeout=30s;
    server 127.0.0.1:3001 max_fails=3 fail_timeout=30s;
    server 127.0.0.1:3002 max_fails=3 fail_timeout=30s;

    # Load balancing method
    least_conn;

    # Keep-alive connections
    keepalive 32;
}

server {
    listen 80;
    listen [::]:80;
    server_name example.com;

    # Redirect HTTP to HTTPS
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    listen [::]:443 ssl http2;
    server_name example.com;

    # SSL Configuration
    ssl_certificate /etc/ssl/certs/example.com.crt;
    ssl_certificate_key /etc/ssl/private/example.com.key;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    ssl_prefer_server_ciphers on;

    # Security Headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;

    # Static files (serve directly from NGINX)
    location /static/ {
        alias /var/www/armature/static/;
        expires 1y;
        add_header Cache-Control "public, immutable";

        # Gzip compression
        gzip on;
        gzip_types text/css application/javascript image/svg+xml;
        gzip_min_length 1000;
    }

    # API routes (proxy to Armature)
    location / {
        proxy_pass http://armature_backend;

        # Headers
        proxy_set_header Host $host;
        # Armature reads the right-most X-Forwarded-For hop via
        # client_address(1); X-Real-IP is not consulted.
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Timeouts
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;

        # WebSocket support
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";

        # Buffering
        proxy_buffering on;
        proxy_buffer_size 4k;
        proxy_buffers 8 4k;
    }

    # Rate limiting
    location /api/ {
        limit_req zone=api burst=20 nodelay;
        proxy_pass http://armature_backend;
        # ... same proxy settings as above
    }

    # Health check endpoint
    location /health {
        access_log off;
        proxy_pass http://armature_backend/health;
    }
}

# Rate limiting zones
limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;
```

#### Systemd Service Files

**Armature Service:**

```ini
# /etc/systemd/system/armature@.service
[Unit]
Description=Armature Web Application (instance %i)
After=network.target

[Service]
Type=simple
User=armature
Group=armature
WorkingDirectory=/opt/armature
Environment="PORT=300%i"
Environment="RUST_LOG=info"
ExecStart=/opt/armature/target/release/armature-app
Restart=always
RestartSec=5

# Security
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/armature/logs

[Install]
WantedBy=multi-user.target
```

**Start multiple instances:**

```bash
# Enable and start 3 instances
sudo systemctl enable armature@0 armature@1 armature@2
sudo systemctl start armature@0 armature@1 armature@2

# Reload NGINX
sudo systemctl reload nginx
```

#### Benefits

✅ **Offloading**: NGINX handles SSL, compression, caching
✅ **Load Balancing**: Distribute load across multiple instances
✅ **Zero-Downtime Deploys**: Reload NGINX config without dropping connections
✅ **Static Assets**: NGINX serves static files efficiently
✅ **Security**: Rate limiting, DDoS protection, WAF integration
✅ **Monitoring**: NGINX logs, status page, metrics

### Approach 2: NGINX Dynamic Module

**Creating an Armature NGINX module** (advanced):

```c
// ngx_http_armature_module.c
#include <ngx_config.h>
#include <ngx_core.h>
#include <ngx_http.h>

// FFI to Rust
extern ngx_int_t armature_handle_request(
    ngx_http_request_t *r,
    u_char *method,
    u_char *uri,
    u_char *body,
    size_t body_len
);

static ngx_int_t ngx_http_armature_handler(ngx_http_request_t *r) {
    // Call into Armature Rust code
    return armature_handle_request(
        r,
        r->method_name.data,
        r->uri.data,
        r->request_body->bufs->buf->pos,
        r->request_body->bufs->buf->last - r->request_body->bufs->buf->pos
    );
}

// Module definition
static ngx_command_t ngx_http_armature_commands[] = {
    {
        ngx_string("armature"),
        NGX_HTTP_LOC_CONF|NGX_CONF_NOARGS,
        ngx_http_armature_handler,
        0,
        0,
        NULL
    },
    ngx_null_command
};

// ... module boilerplate
```

**Rust FFI side:**

```rust
// lib.rs
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_uchar};

#[no_mangle]
pub extern "C" fn armature_handle_request(
    r: *mut NgxHttpRequest,
    method: *const c_uchar,
    uri: *const c_uchar,
    body: *const c_uchar,
    body_len: usize,
) -> c_int {
    // Convert C strings to Rust
    let method = unsafe { CStr::from_ptr(method as *const c_char) }
        .to_str()
        .unwrap();

    // Route through Armature
    // ... implementation

    0 // NGX_OK
}
```

**Build configuration:**

```bash
# Configure NGINX with Armature module
./configure --add-dynamic-module=/path/to/armature-nginx-module

# Compile NGINX
make
make install
```

**NGINX config:**

```nginx
load_module modules/ngx_http_armature_module.so;

server {
    location / {
        armature;
    }
}
```

---

## Ferron Integration

[Ferron](https://ferron.sh/) is a modern web server written in Rust, making it potentially easier to integrate with Armature.

### Approach 1: Reverse Proxy

**Ferron Configuration:**

```toml
# ferron.toml
[server]
host = "0.0.0.0"
port = 80

[[proxies]]
name = "armature-backend"
path = "/"
backend = "http://127.0.0.1:3000"

# Load balancing
[[proxies.backends]]
url = "http://127.0.0.1:3000"
weight = 1

[[proxies.backends]]
url = "http://127.0.0.1:3001"
weight = 1

[[proxies.backends]]
url = "http://127.0.0.1:3002"
weight = 1

# Static files
[[static]]
path = "/static"
directory = "/var/www/static"
cache_control = "public, max-age=31536000"

# TLS
[tls]
enabled = true
cert = "/etc/ssl/certs/example.com.crt"
key = "/etc/ssl/private/example.com.key"
```

**Start Ferron:**

```bash
ferron --config ferron.toml
```

### Approach 2: Ferron as Library

Since Ferron is written in Rust, we could potentially embed it:

```rust
// Cargo.toml
[dependencies]
ferron-core = "0.1" # hypothetical

// main.rs
use ferron_core::Server as FerronServer;
use armature_framework::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Application::create::<AppModule>().await;

    // Wrap Armature with Ferron
    let ferron = FerronServer::new()
        .with_handler(app)
        .with_static("/static", "/var/www/static")
        .with_rate_limiting()
        .build();

    ferron.listen("0.0.0.0:3000").await?;
    Ok(())
}
```

### Approach 3: Git Submodule Integration

**Add Ferron as git submodule:**

```bash
# Add Ferron source as submodule
git submodule add https://github.com/ferron-project/ferron.git external/ferron
git submodule update --init --recursive

# Add to Cargo workspace
```

**Cargo.toml:**

```toml
[workspace]
members = [
    "armature-core",
    "armature-macro",
    "armature",
    "external/ferron", # If Ferron exposes a library crate
]

[dependencies]
ferron = { path = "external/ferron" }
```

---

## Pluggable Server Backend

We could make Armature's HTTP server **pluggable** with a trait-based approach:

### Server Trait Design

```rust
// armature-core/src/server.rs

use async_trait::async_trait;
use crate::{HttpRequest, HttpResponse, Error};
use std::net::SocketAddr;

/// Trait for HTTP server backends
#[async_trait]
pub trait HttpServer: Send + Sync + 'static {
    /// Start the HTTP server
    async fn listen(&self, addr: SocketAddr) -> Result<(), Error>;

    /// Start the HTTPS server
    async fn listen_tls(
        &self,
        addr: SocketAddr,
        cert: &[u8],
        key: &[u8],
    ) -> Result<(), Error>;

    /// Graceful shutdown
    async fn shutdown(&self) -> Result<(), Error>;

    /// Get server name
    fn name(&self) -> &str;
}

/// Request handler callback
pub type RequestHandler = Arc<
    dyn Fn(HttpRequest) -> Pin<Box<dyn Future<Output = Result<HttpResponse, Error>> + Send>>
        + Send
        + Sync,
>;
```

### Hyper Backend (Default)

```rust
// armature-server-hyper/src/lib.rs

pub struct HyperServer {
    router: Arc<Router>,
    lifecycle: Arc<LifecycleManager>,
}

#[async_trait]
impl HttpServer for HyperServer {
    async fn listen(&self, addr: SocketAddr) -> Result<(), Error> {
        // Current implementation
        let listener = TcpListener::bind(addr).await?;
        // ... existing code
        Ok(())
    }

    fn name(&self) -> &str {
        "Hyper"
    }
}
```

### NGINX Backend (via Unix Socket)

```rust
// armature-server-nginx/src/lib.rs

pub struct NginxServer {
    router: Arc<Router>,
    socket_path: PathBuf,
}

#[async_trait]
impl HttpServer for NginxServer {
    async fn listen(&self, _addr: SocketAddr) -> Result<(), Error> {
        // Listen on Unix socket
        let listener = UnixListener::bind(&self.socket_path)?;

        loop {
            let (stream, _) = listener.accept().await?;
            let router = self.router.clone();

            tokio::spawn(async move {
                // Handle FastCGI/SCGI protocol
                handle_nginx_connection(stream, router).await;
            });
        }
    }

    fn name(&self) -> &str {
        "NGINX-FastCGI"
    }
}
```

### Ferron Backend

```rust
// armature-server-ferron/src/lib.rs

pub struct FerronServer {
    router: Arc<Router>,
    config: FerronConfig,
}

#[async_trait]
impl HttpServer for FerronServer {
    async fn listen(&self, addr: SocketAddr) -> Result<(), Error> {
        // Use Ferron's server implementation
        ferron_core::serve(addr, |req| {
            let router = self.router.clone();
            async move {
                let armature_req = convert_request(req);
                let armature_resp = router.route(armature_req).await?;
                Ok(convert_response(armature_resp))
            }
        })
        .await
    }

    fn name(&self) -> &str {
        "Ferron"
    }
}
```

### Application with Pluggable Server

```rust
// armature-core/src/application.rs

impl Application {
    /// Create application with custom server backend
    pub async fn with_server<S: HttpServer>(
        self,
        server: S,
    ) -> Result<(), Error> {
        println!("🚀 Using {} server backend", server.name());
        server.listen(SocketAddr::from(([0, 0, 0, 0], 3000))).await
    }
}

// Usage
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Application::create::<AppModule>().await;

    // Choose backend
    #[cfg(feature = "nginx")]
    let server = NginxServer::new(app.router.clone());

    #[cfg(feature = "ferron")]
    let server = FerronServer::new(app.router.clone());

    #[cfg(not(any(feature = "nginx", feature = "ferron")))]
    let server = HyperServer::new(app.router.clone(), app.lifecycle.clone());

    app.with_server(server).await?;
    Ok(())
}
```

---

## Comparison

| Feature | Hyper (Current) | NGINX Reverse Proxy | Ferron Reverse Proxy | Pluggable Backend |
|---------|-----------------|---------------------|---------------------|-------------------|
| **Setup Complexity** | ⭐⭐⭐⭐⭐ Simple | ⭐⭐⭐ Moderate | ⭐⭐⭐⭐ Easy | ⭐⭐ Complex |
| **Performance** | ⭐⭐⭐⭐⭐ Excellent | ⭐⭐⭐⭐ Very Good | ⭐⭐⭐⭐ Very Good | ⭐⭐⭐⭐ Good |
| **Load Balancing** | ❌ No | ✅ Yes | ✅ Yes | ✅ Depends |
| **SSL Termination** | ⭐⭐⭐ Basic | ⭐⭐⭐⭐⭐ Advanced | ⭐⭐⭐⭐ Good | ✅ Depends |
| **Static Files** | ⭐⭐ Basic | ⭐⭐⭐⭐⭐ Optimized | ⭐⭐⭐⭐ Good | ✅ Depends |
| **Caching** | ❌ No | ✅ Advanced | ✅ Yes | ✅ Depends |
| **Zero Downtime** | ❌ No | ✅ Yes | ✅ Yes | ✅ Yes |
| **Rust Native** | ✅ Yes | ❌ No (C) | ✅ Yes | ✅ Yes |
| **Production Battle-Tested** | ✅ Yes | ⭐⭐⭐⭐⭐ Proven | ⭐⭐ Newer | ⭐⭐ Experimental |
| **Community Support** | ⭐⭐⭐⭐⭐ Large | ⭐⭐⭐⭐⭐ Huge | ⭐⭐⭐ Growing | ⭐⭐ Limited |

---

## Recommendations

### For Development

✅ **Use Hyper directly (current setup)**
- Fast iteration
- Simple debugging
- No extra dependencies

### For Production (Small Scale)

✅ **Hyper with systemd**
- Simple deployment
- Good performance
- No reverse proxy complexity

### For Production (Medium to Large Scale)

✅ **NGINX Reverse Proxy + Hyper**
- Battle-tested in production
- Advanced features (load balancing, caching, SSL)
- Industry standard
- Easy to find expertise

### For Rust-Only Stack

✅ **Ferron Reverse Proxy + Hyper**
- All-Rust stack
- Modern features
- Good performance
- Easier to customize

### For Maximum Flexibility

✅ **Implement Pluggable Backend Trait**
- Choose backend at runtime/compile-time
- Test different servers easily
- Custom implementations possible

---

## Implementation Roadmap

If we want to add pluggable server support:

### Phase 1: Define Server Trait
- [ ] Create `HttpServer` trait
- [ ] Refactor current code to use trait
- [ ] Add feature flags for backends

### Phase 2: Keep Hyper as Default
- [ ] Implement `HyperServer` backend
- [ ] Maintain current API compatibility
- [ ] Add benchmarks

### Phase 3: Add NGINX Backend (Optional)
- [ ] Implement FastCGI/SCGI protocol
- [ ] Create `NginxServer` backend
- [ ] Document integration

### Phase 4: Add Ferron Backend (Optional)
- [ ] Add Ferron as git submodule or dependency
- [ ] Create adapter layer
- [ ] Implement `FerronServer` backend

---

## Conclusion

**Current Recommendation:**

1. **Keep Hyper as default** for development and simple deployments
2. **Document NGINX reverse proxy setup** for production (add to docs)
3. **Consider Ferron** for users wanting all-Rust stack
4. **Implement pluggable backend trait** if there's strong demand

**Next Steps:**

1. Add NGINX configuration examples to documentation
2. Create deployment guides for various scenarios
3. Consider creating `armature-server-*` crates for alternative backends if needed

The **reverse proxy pattern with NGINX** is the industry-standard approach and provides the most operational features (TLS, load balancing, caching, WAF) without requiring changes to Armature's core architecture.

