# Vue.js Server-Side Rendering Guide

This guide covers integrating Vue.js 3 server-side rendering with Armature.

## Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
- [Configuration](#configuration)
- [Server Setup](#server-setup)
- [Vue Setup](#vue-setup)
- [Advanced Usage](#advanced-usage)
- [Best Practices](#best-practices)

## Overview

The `armature-vue` module provides seamless integration between Armature and Vue.js SSR, enabling:

- Server-side rendering of Vue 3 components
- SEO-friendly pages
- Fast initial page load
- Client-side hydration
- Static file serving
- API data integration
- Composition API support

## Quick Start

### 1. Add to Cargo.toml

```toml
[dependencies]
armature = { version = "0.1", features = ["vue"] }
```

### 2. Configure Vue SSR

```rust
use armature::prelude::*;
use armature::armature_vue::{VueConfig, VueService};
use std::path::PathBuf;

let vue_config = VueConfig::new(PathBuf::from("dist"))
    .with_static_dir(PathBuf::from("dist/client"))
    .with_server_entry("server-bundle.js".to_string())
    .with_template(PathBuf::from("dist/index.html"));

let vue_service = VueService::new(vue_config);
```

### 3. Setup Routes

```rust
// SSR route
router.add_route(Route {
    method: HttpMethod::GET,
    path: "/".to_string(),
    handler: Arc::new(move |req| {
        let service = vue_service.clone();
        Box::pin(async move {
            service.render(&req).await
        })
    }),
});

// Static files
router.add_route(Route {
    method: HttpMethod::GET,
    path: "/assets".to_string(),
    handler: Arc::new(move |req| {
        let service = static_service.clone();
        Box::pin(async move {
            let path = req.path.trim_start_matches("/assets");
            service.serve_static(path).await
        })
    }),
});
```

## Configuration

### VueConfig Options

```rust
use armature_vue::VueConfig;
use std::path::PathBuf;

let config = VueConfig::new(PathBuf::from("dist"))
    // Static assets directory
    .with_static_dir(PathBuf::from("dist/client"))

    // Server entry point
    .with_server_entry("server-bundle.js".to_string())

    // HTML template
    .with_template(PathBuf::from("dist/index.html"))

    // Enable caching (optional - stays stateless)
    .with_cache(true)
    .with_cache_ttl(300) // 5 minutes

    // Node.js path
    .with_node_path("/usr/bin/node".to_string())

    // Compression
    .with_compression(true)

    // Client manifest (for preload/prefetch)
    .with_client_manifest(PathBuf::from("dist/client/client-manifest.json"));
```

### Configuration Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `build_dir` | `PathBuf` | `"dist"` | Build output directory |
| `static_dir` | `PathBuf` | `"dist/client"` | Static assets directory |
| `server_entry` | `String` | `"server-bundle.js"` | Server bundle entry point |
| `template_path` | `PathBuf` | `"index.html"` | HTML template |
| `cache_enabled` | `bool` | `false` | Enable render caching |
| `cache_ttl` | `u64` | `300` | Cache TTL in seconds |
| `node_path` | `String` | `"node"` | Node.js executable |
| `compression` | `bool` | `true` | Enable compression |
| `client_manifest` | `Option<PathBuf>` | `None` | Client manifest for preload |

## Server Setup

### Basic Server

```rust
use armature::prelude::*;
use armature::armature_vue::{VueConfig, VueService};
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let container = Container::new();
    let mut router = Router::new();

    // Configure Vue
    let vue_config = VueConfig::new(PathBuf::from("dist"));
    let vue_service = VueService::new(vue_config);

    // SSR route
    let ssr = vue_service.clone();
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/".to_string(),
        handler: Arc::new(move |req| {
            let service = ssr.clone();
            Box::pin(async move {
                service.render(&req).await
            })
        }),
    });

    let app = Application::new(container, router);
    app.listen(3000).await.unwrap();
}
```

### With API Integration

```rust
use armature::prelude::*;
use armature::armature_vue::{VueConfig, VueService};

#[injectable]
#[derive(Clone, Default)]
struct DataService;

impl DataService {
    fn get_page_data(&self) -> PageData {
        PageData {
            title: "My Page".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        }
    }
}

#[controller("/api")]
#[derive(Default, Clone)]
struct ApiController {
    data_service: DataService,
}

impl ApiController {
    fn get_data(&self) -> Result<Json<PageData>, Error> {
        Ok(Json(self.data_service.get_page_data()))
    }
}

// In main():
let vue_service = VueService::new(vue_config);
let data_service = DataService::default();

// Pre-fetch data and render
router.add_route(Route {
    method: HttpMethod::GET,
    path: "/".to_string(),
    handler: Arc::new(move |_req| {
        let service = vue_service.clone();
        let data_svc = data_service.clone();
        Box::pin(async move {
            // Fetch data server-side
            let data = data_svc.get_page_data();
            let json_data = serde_json::to_value(data)
                .map_err(|e| Error::Serialization(e.to_string()))?;

            // Render with data
            service.render_with_data("/", json_data).await
        })
    }),
});
```

## Vue Setup

### 1. Create Vue 3 App

```bash
npm create vue@latest my-app
cd my-app
npm install
```

### 2. Install SSR Dependencies

```bash
npm install vue@latest vue-router@latest
npm install --save-dev vite @vitejs/plugin-vue
```

### 3. Create Server Entry

Create `server-bundle.js`:

```javascript
const { createSSRApp } = require('vue');
const { renderToString } = require('vue/server-renderer');
const { createRouter, createMemoryHistory } = require('vue-router');

// Your Vue app factory
function createApp(context) {
  const app = createSSRApp({
    data() {
      return context.state || {};
    },
    template: `
      <div id="app">
        <h1>{{ title }}</h1>
        <ul>
          <li v-for="item in items" :key="item">{{ item }}</li>
        </ul>
      </div>
    `
  });

  const router = createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: '/', component: { template: '<div>Home</div>' } },
      { path: '/about', component: { template: '<div>About</div>' } }
    ]
  });

  app.use(router);
  return { app, router };
}

// Read render request from stdin
let input = '';
process.stdin.on('data', (chunk) => {
  input += chunk;
});

process.stdin.on('end', async () => {
  try {
    const { url, context } = JSON.parse(input);

    const { app, router } = createApp(context || {});

    // Navigate to the route
    await router.push(url);
    await router.isReady();

    // Render to string
    const html = await renderToString(app);

    // Output HTML
    console.log(html);
  } catch (error) {
    console.error('SSR Error:', error);
    process.exit(1);
  }
});
```

### 4. Update package.json

```json
{
  "scripts": {
    "build": "vite build && vite build --ssr",
    "build:server": "node server-bundle.js"
  }
}
```

## Advanced Usage

### With Vuex/Pinia State Management

```javascript
const { createSSRApp } = require('vue');
const { createPinia } = require('pinia');

function createApp(context) {
  const app = createSSRApp(AppComponent);
  const pinia = createPinia();

  // Restore state from context
  if (context.state) {
    pinia.state.value = context.state;
  }

  app.use(pinia);
  return { app, pinia };
}

process.stdin.on('end', async () => {
  const { url, context } = JSON.parse(input);
  const { app, pinia } = createApp(context);

  const html = await renderToString(app);

  // Serialize state for client hydration
  const state = JSON.stringify(pinia.state.value);
  const fullHtml = html + `<script>window.__INITIAL_STATE__=${state}</script>`;

  console.log(fullHtml);
});
```

### With Component-Level Data Fetching

```rust
use armature::prelude::*;
use armature::armature_vue::VueService;

async fn render_page_with_data(
    vue_service: &VueService,
    url: &str,
    data_service: &DataService,
) -> Result<HttpResponse, Error> {
    // Fetch data based on route
    let data = match url {
        "/" => data_service.get_home_data(),
        "/about" => data_service.get_about_data(),
        "/blog" => data_service.get_blog_data(),
        _ => return Err(Error::NotFound("Page not found".to_string())),
    };

    // Serialize for Vue
    let initial_state = serde_json::json!({
        "pageData": data,
        "user": null, // Stateless - would extract from JWT
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    // Render with pre-fetched data
    vue_service.render_with_data(url, initial_state).await
}
```

### With Vue Router

```javascript
// server-bundle.js with Vue Router
const { createSSRApp } = require('vue');
const { renderToString } = require('vue/server-renderer');
const { createRouter, createMemoryHistory } = require('vue-router');
const routes = require('./routes').default;

async function render(url, context) {
  const app = createSSRApp({
    data: () => context.state || {}
  });

  const router = createRouter({
    history: createMemoryHistory(),
    routes
  });

  app.use(router);

  // Navigate and wait
  await router.push(url);
  await router.isReady();

  // Render
  const html = await renderToString(app);
  return html;
}

// stdin processing...
```

### Static File Serving

```rust
use armature::prelude::*;
use armature::armature_vue::VueService;

// Serve Vue assets
router.add_route(Route {
    method: HttpMethod::GET,
    path: "/assets".to_string(),
    handler: Arc::new(move |req| {
        let service = vue_service.clone();
        Box::pin(async move {
            // Extract path after /assets
            let path = req.path.strip_prefix("/assets/")
                .unwrap_or("");

            service.serve_static(path).await
        })
    }),
});

// Serve with different patterns
router.add_route(Route {
    method: HttpMethod::GET,
    path: "/js".to_string(),
    handler: Arc::new(move |req| {
        let service = vue_service.clone();
        Box::pin(async move {
            let path = format!("js{}", req.path.strip_prefix("/js").unwrap_or(""));
            service.serve_static(&path).await
        })
    }),
});
```

## Best Practices

### 1. Pre-fetch Data Server-Side

```rust
// GOOD - Fetch data on server before rendering
let data = database.get_page_data(page_id).await?;
vue_service.render_with_data(url, serde_json::to_value(data)?).await

// BAD - Render without data, causes client-side fetch
vue_service.render(&req).await
```

### 2. Handle Errors Gracefully

```rust
match vue_service.render(&req).await {
    Ok(response) => Ok(response),
    Err(e) => {
        eprintln!("SSR error: {}", e);
        // Fallback to client-side rendering
        Ok(serve_fallback_html())
    }
}
```

### 3. Separate API Routes

```rust
// API routes (JSON)
router.add_route("/api/*", api_handler);

// Static files
router.add_route("/assets/*", static_handler);

// SSR (catch-all for app routes)
router.add_route("/*", ssr_handler);
```

### 4. Use Environment-Specific Config

```rust
let vue_config = if cfg!(debug_assertions) {
    VueConfig::new(PathBuf::from("dist"))
        .with_cache(false) // No cache in development
} else {
    VueConfig::new(PathBuf::from("dist"))
        .with_cache(true)
        .with_cache_ttl(600) // 10 minutes in production
        .with_compression(true)
};
```

### 5. Health Checks

```rust
// Before starting server
vue_service.health_check().await
    .expect("Vue SSR is not ready");

// Health endpoint
router.add_route(Route {
    method: HttpMethod::GET,
    path: "/health".to_string(),
    handler: Arc::new(move |_req| {
        let service = vue_service.clone();
        Box::pin(async move {
            service.health_check().await?;
            Ok(HttpResponse::ok().json(json!({ "status": "healthy" }))?)
        })
    }),
});
```

## Complete Example

### Rust Server

```rust
use armature::prelude::*;
use armature::armature_vue::{VueConfig, VueService};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
struct PageData {
    title: String,
    content: String,
    items: Vec<String>,
}

#[injectable]
#[derive(Clone, Default)]
struct DataService;

impl DataService {
    fn get_home_data(&self) -> PageData {
        PageData {
            title: "Home".to_string(),
            content: "Welcome to Vue SSR!".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        }
    }
}

#[tokio::main]
async fn main() {
    let container = Container::new();
    let mut router = Router::new();

    // Setup Vue
    let vue_config = VueConfig::new(PathBuf::from("dist"))
        .with_compression(true);
    let vue_service = VueService::new(vue_config);

    // Register services
    let data_service = DataService::default();
    container.register(data_service.clone());

    // API route
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/api/data".to_string(),
        handler: Arc::new(move |_req| {
            let svc = data_service.clone();
            Box::pin(async move {
                Ok(Json(svc.get_home_data()).into_response()?)
            })
        }),
    });

    // SSR route with data
    let ssr_service = vue_service.clone();
    let ssr_data = data_service.clone();
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/".to_string(),
        handler: Arc::new(move |_req| {
            let service = ssr_service.clone();
            let data_svc = ssr_data.clone();
            Box::pin(async move {
                let data = data_svc.get_home_data();
                let json_data = serde_json::to_value(data)
                    .map_err(|e| Error::Serialization(e.to_string()))?;
                service.render_with_data("/", json_data).await
            })
        }),
    });

    let app = Application::new(container, router);
    app.listen(3000).await.unwrap();
}
```

### Vue Server Bundle

```javascript
// server-bundle.js
const { createSSRApp } = require('vue');
const { renderToString } = require('vue/server-renderer');
const { createRouter, createMemoryHistory } = require('vue-router');

// App factory
function createApp(initialState = {}) {
  const app = createSSRApp({
    data() {
      return {
        ...initialState
      };
    },
    template: `
      <div id="app">
        <h1>{{ title }}</h1>
        <p>{{ content }}</p>
        <ul>
          <li v-for="item in items" :key="item">{{ item }}</li>
        </ul>
      </div>
    `
  });

  // Setup router
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: '/', component: { template: '<div>Home</div>' } },
      { path: '/about', component: { template: '<div>About</div>' } }
    ]
  });

  app.use(router);

  return { app, router };
}

// Handle render requests
let input = '';
process.stdin.setEncoding('utf8');

process.stdin.on('data', (chunk) => {
  input += chunk;
});

process.stdin.on('end', async () => {
  try {
    const { url, context } = JSON.parse(input);

    const { app, router } = createApp(context.state || {});

    await router.push(url);
    await router.isReady();

    const html = await renderToString(app);

    // Inject initial state
    const stateScript = `<script>window.__INITIAL_STATE__=${JSON.stringify(context.state || {})}</script>`;
    const fullHtml = html + stateScript;

    console.log(fullHtml);
  } catch (error) {
    console.error('SSR Error:', error.message);
    process.exit(1);
  }
});
```

### Client-Side Hydration

```javascript
// client.js
import { createSSRApp } from 'vue';
import { createRouter, createWebHistory } from 'vue-router';

const app = createSSRApp({
  data() {
    return window.__INITIAL_STATE__ || {};
  },
  // ... app definition
});

const router = createRouter({
  history: createWebHistory(),
  routes: [
    // ... routes
  ]
});

app.use(router);
app.mount('#app');
```

## Comparison with Other Frameworks

### Vite SSR

If using Vite, you can integrate directly:

```javascript
// vite.config.js
import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';

export default defineConfig({
  plugins: [vue()],
  build: {
    ssr: true,
    outDir: 'dist/server'
  }
});
```

### Nuxt.js

For Nuxt.js-style features, you can:
- Use Vue Router for routing
- Implement page-level data fetching in Rust
- Use Armature's DI for server-side services
- Leverage Armature's module system

## Performance Tips

### 1. Enable Caching

```rust
let config = VueConfig::new(PathBuf::from("dist"))
    .with_cache(true)
    .with_cache_ttl(300); // 5 minutes

// Cache is stateless - works across multiple servers
// Implementation would use Redis or similar external cache
```

### 2. Preload Critical Resources

```rust
// Add preload headers for critical resources
response.headers.insert(
    "Link".to_string(),
    "</app.js>; rel=preload; as=script, </style.css>; rel=preload; as=style".to_string()
);
```

### 3. Stream Rendering (Future)

```rust
// Future feature: stream HTML as it's generated
vue_service.render_stream(url).await
```

## Troubleshooting

### Node.js Not Found

```rust
let config = VueConfig::new(PathBuf::from("dist"))
    .with_node_path("/usr/local/bin/node".to_string());
```

### Template Not Found

```rust
let config = VueConfig::new(PathBuf::from("dist"))
    .with_template(PathBuf::from("public/index.html"));
```

### SSR Bundle Not Found

```bash
# Ensure you've built the server bundle
npm run build:server

# Check the output path
ls dist/server-bundle.js
```

### Port Already in Use

```rust
// Try different port
app.listen(3001).await
```

## See Also

- [Angular SSR Guide](ANGULAR_SSR_GUIDE.md) - Angular integration
- [React SSR](../armature-react) - React integration
- [Stateless Architecture](STATELESS_ARCHITECTURE.md) - No server-side sessions
- [API Integration](API_INTEGRATION.md) - Connecting Vue to APIs

