# Hot Module Reload (HMR) Guide

Comprehensive guide to using Armature's Hot Module Reload system for rapid development with SSR frameworks.

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Quick Start](#quick-start)
- [Configuration](#configuration)
- [Framework Integration](#framework-integration)
  - [React HMR](#react-hmr)
  - [Vue HMR](#vue-hmr)
  - [Angular HMR](#angular-hmr)
  - [Svelte HMR](#svelte-hmr)
- [How It Works](#how-it-works)
- [Advanced Usage](#advanced-usage)
- [Troubleshooting](#troubleshooting)

## Overview

Armature's Hot Module Reload (HMR) system provides **automatic reloading** of your web application during development without manually refreshing the browser. It watches your source files for changes and instantly reflects those changes in the browser.

### What Gets Hot Reloaded?

✅ **JavaScript/TypeScript** - Full page reload  
✅ **CSS/SCSS/Less** - Hot reload without page refresh  
✅ **HTML Templates** - Full page reload  
✅ **Vue/Svelte Components** - Hot reload  
✅ **Static Assets** - Automatic cache busting  

### Development vs Production

```rust
// Development - HMR enabled
let hmr_config = HmrConfig::new()
    .watch_path(PathBuf::from("src"))
    .watch_path(PathBuf::from("public"))
    .websocket_port(3001);

// Production - HMR disabled
let hmr_config = HmrConfig::new()
    .enabled(false); // No file watching or WebSocket overhead
```

## Features

### 🔥 File System Watching

Monitors your source files for changes in real-time using the `notify` crate:

```rust
let hmr_config = HmrConfig::new()
    .watch_path(PathBuf::from("src"))
    .watch_path(PathBuf::from("public"))
    .watch_extension("ts".to_string())
    .watch_extension("tsx".to_string())
    .watch_extension("css".to_string());
```

### 📡 WebSocket Communication

Establishes a WebSocket connection between the server and client for instant notifications:

- **Server → Client**: Notifies browser of file changes
- **Client → Server**: Heartbeat to maintain connection
- **Auto-reconnect**: Client automatically reconnects if connection drops

### 🎯 Smart Debouncing

Prevents rapid-fire reloads when multiple files change:

```rust
let hmr_config = HmrConfig::new()
    .debounce(100); // Wait 100ms before triggering reload
```

### 🚫 Ignore Patterns

Avoid watching unnecessary directories:

```rust
let hmr_config = HmrConfig::new()
    .ignore_pattern("node_modules".to_string())
    .ignore_pattern("dist".to_string())
    .ignore_pattern(".git".to_string());
```

### 🎨 CSS Hot Reload

CSS changes apply **without** a full page reload:

- Styles update instantly
- No loss of application state
- Smooth visual feedback

## Quick Start

### 1. Basic HMR Setup

```rust
use armature::prelude::*;
use armature::hmr::{HmrConfig, HmrManager, inject_hmr_script};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create HMR configuration
    let hmr_config = HmrConfig::new()
        .watch_path(PathBuf::from("src"))
        .watch_path(PathBuf::from("public"))
        .websocket_port(3001)
        .verbose(true);

    // Create HMR manager
    let hmr_manager = Arc::new(HmrManager::new(hmr_config));

    // Start watching for changes
    hmr_manager.start_watching().await?;

    println!("🔥 HMR enabled - watching for changes...");

    // Your application setup...
    
    Ok(())
}
```

### 2. Inject HMR Script

Automatically inject the HMR client into your HTML:

```rust
use armature::hmr::inject_hmr_script;

async fn render_page(hmr_manager: &HmrManager) -> String {
    let html = r#"
        <!DOCTYPE html>
        <html>
        <head><title>My App</title></head>
        <body>
            <div id="app">Hello World</div>
        </body>
        </html>
    "#.to_string();

    // Inject HMR script before </body>
    inject_hmr_script(html, hmr_manager).await
}
```

**Result:**

```html
<!DOCTYPE html>
<html>
<head><title>My App</title></head>
<body>
    <div id="app">Hello World</div>
    <script>
    (function() {
      console.log('🔥 HMR Client initialized');
      let ws = new WebSocket('ws://localhost:3001');
      // ... HMR client code ...
    })();
    </script>
</body>
</html>
```

## Configuration

### HmrConfig Options

```rust
pub struct HmrConfig {
    /// Enable/disable HMR (default: true)
    pub enabled: bool,
    
    /// Paths to watch (default: ["src", "public"])
    pub watch_paths: Vec<PathBuf>,
    
    /// File extensions to watch (default: js, ts, css, html, etc.)
    pub watch_extensions: Vec<String>,
    
    /// Patterns to ignore (default: node_modules, dist, .git)
    pub ignore_patterns: Vec<String>,
    
    /// Debounce delay in milliseconds (default: 100)
    pub debounce_ms: u64,
    
    /// WebSocket port (default: 3001)
    pub websocket_port: u16,
    
    /// Enable verbose logging (default: false)
    pub verbose: bool,
}
```

### Complete Configuration Example

```rust
let hmr_config = HmrConfig::new()
    // Watch specific directories
    .watch_path(PathBuf::from("src"))
    .watch_path(PathBuf::from("client"))
    .watch_path(PathBuf::from("styles"))
    
    // Watch specific extensions
    .watch_extension("ts".to_string())
    .watch_extension("tsx".to_string())
    .watch_extension("vue".to_string())
    .watch_extension("scss".to_string())
    
    // Ignore patterns
    .ignore_pattern("node_modules".to_string())
    .ignore_pattern("dist".to_string())
    .ignore_pattern("build".to_string())
    .ignore_pattern(".next".to_string())
    .ignore_pattern(".nuxt".to_string())
    
    // Fine-tune behavior
    .debounce(200)
    .websocket_port(3002)
    .verbose(true);
```

## Framework Integration

### React HMR

React with HMR using Vite or custom setup:

```rust
use armature::prelude::*;
use armature_react::{ReactConfig, ReactService};
use armature::hmr::{HmrConfig, HmrManager, inject_hmr_script};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // HMR Configuration
    let hmr_config = HmrConfig::new()
        .watch_path(PathBuf::from("client/src"))
        .watch_extension("jsx".to_string())
        .watch_extension("tsx".to_string())
        .websocket_port(3001);

    let hmr_manager = Arc::new(HmrManager::new(hmr_config));
    hmr_manager.start_watching().await?;

    // React Configuration
    let react_config = ReactConfig::new()
        .build_dir("client/dist")
        .dev_mode(true);

    let react_service = ReactService::new(react_config);

    // Middleware to inject HMR
    async fn hmr_middleware(
        req: HttpRequest,
        react_service: Arc<ReactService>,
        hmr_manager: Arc<HmrManager>,
    ) -> Result<HttpResponse, Error> {
        let response = react_service.render(&req).await?;
        
        // Inject HMR script into HTML
        let html = String::from_utf8(response.body)?;
        let html_with_hmr = inject_hmr_script(html, &hmr_manager).await;
        
        Ok(HttpResponse::ok()
            .with_header("Content-Type", "text/html; charset=utf-8")
            .with_body(html_with_hmr.into_bytes()))
    }

    println!("🔥 React app with HMR running!");
    println!("   Open http://localhost:3000");
    
    Ok(())
}
```

### Vue HMR

Vue with HMR using Vite:

```rust
use armature::prelude::*;
use armature_vue::{VueConfig, VueService};
use armature::hmr::{HmrConfig, HmrManager, inject_hmr_script};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // HMR Configuration for Vue
    let hmr_config = HmrConfig::new()
        .watch_path(PathBuf::from("client/src"))
        .watch_path(PathBuf::from("client/components"))
        .watch_extension("vue".to_string())
        .watch_extension("ts".to_string())
        .watch_extension("js".to_string())
        .websocket_port(3001);

    let hmr_manager = Arc::new(HmrManager::new(hmr_config));
    hmr_manager.start_watching().await?;

    // Vue Configuration
    let vue_config = VueConfig::new()
        .build_dir("client/dist")
        .ssr_bundle("server/vue-ssr-server-bundle.json")
        .dev_mode(true);

    let vue_service = VueService::new(vue_config);

    // Listen for HMR events
    let mut hmr_rx = hmr_manager.subscribe();
    tokio::spawn(async move {
        while let Ok(event) = hmr_rx.recv().await {
            println!("🔥 HMR Event: {:?}", event);
            
            // Notify connected clients
            // (WebSocket broadcast implementation)
        }
    });

    println!("🔥 Vue app with HMR running!");
    
    Ok(())
}
```

### Angular HMR

Angular with HMR using Angular CLI:

```rust
use armature::prelude::*;
use armature_angular::{AngularConfig, AngularService};
use armature::hmr::{HmrConfig, HmrManager, inject_hmr_script};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // HMR Configuration for Angular
    let hmr_config = HmrConfig::new()
        .watch_path(PathBuf::from("client/src"))
        .watch_extension("ts".to_string())
        .watch_extension("html".to_string())
        .watch_extension("scss".to_string())
        .websocket_port(3001)
        .verbose(true);

    let hmr_manager = Arc::new(HmrManager::new(hmr_config));
    hmr_manager.start_watching().await?;

    // Angular Configuration
    let angular_config = AngularConfig::new()
        .dist_path("dist/my-app/browser")
        .server_path("dist/my-app/server")
        .dev_mode(true);

    let angular_service = AngularService::new(angular_config);

    println!("🔥 Angular app with HMR running!");
    println!("   Open http://localhost:4200");
    
    Ok(())
}
```

### Svelte HMR

Svelte with HMR using SvelteKit/Vite:

```rust
use armature::prelude::*;
use armature_svelte::{SvelteConfig, SvelteService};
use armature::hmr::{HmrConfig, HmrManager, inject_hmr_script};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // HMR Configuration for Svelte
    let hmr_config = HmrConfig::new()
        .watch_path(PathBuf::from("client/src"))
        .watch_extension("svelte".to_string())
        .watch_extension("ts".to_string())
        .watch_extension("js".to_string())
        .websocket_port(3001);

    let hmr_manager = Arc::new(HmrManager::new(hmr_config));
    hmr_manager.start_watching().await?;

    // Svelte Configuration
    let svelte_config = SvelteConfig::new()
        .build_dir("client/public")
        .bundle_path("client/build/bundle.js")
        .dev_mode(true);

    let svelte_service = SvelteService::new(svelte_config);

    println!("🔥 Svelte app with HMR running!");
    
    Ok(())
}
```

## How It Works

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      FILE SYSTEM                            │
│  src/app.tsx, src/styles.css, public/index.html            │
└────────────────────┬────────────────────────────────────────┘
                     │
                     │ (file changes)
                     ▼
┌─────────────────────────────────────────────────────────────┐
│                   HMR MANAGER (Rust)                        │
│  • Watches files with notify crate                          │
│  • Debounces events                                         │
│  • Filters by extension                                     │
│  • Broadcasts to clients                                    │
└────────────────────┬────────────────────────────────────────┘
                     │
                     │ (WebSocket)
                     ▼
┌─────────────────────────────────────────────────────────────┐
│                 HMR CLIENT (Browser)                        │
│  • Receives change notifications                            │
│  • Determines reload strategy                               │
│  • Applies CSS changes without reload                       │
│  • Triggers full reload for JS/HTML                         │
└─────────────────────────────────────────────────────────────┘
```

### Event Flow

1. **File Change Detected**
   - Developer saves `src/App.tsx`
   - `notify` crate detects change

2. **Event Processing**
   - HmrManager receives event
   - Checks if file matches watched extensions
   - Applies debouncing
   - Creates `HmrEvent`

3. **Client Notification**
   - WebSocket message sent to browser
   - Message includes: `{ type: "js-update", path: "src/App.tsx" }`

4. **Browser Response**
   - Client receives message
   - For JS/TS: Full page reload
   - For CSS: Hot reload stylesheets
   - For HTML: Full page reload

### CSS Hot Reload (No Page Refresh)

```javascript
// Client-side code injected by HMR
function reloadCSS(path) {
  const links = document.querySelectorAll('link[rel="stylesheet"]');
  links.forEach(link => {
    if (!path || link.href.includes(path)) {
      const href = link.href.split('?')[0];
      link.href = href + '?t=' + Date.now(); // Cache busting
    }
  });
}
```

## Advanced Usage

### Custom Event Handling

Subscribe to HMR events in your application:

```rust
use armature::hmr::{HmrManager, HmrEventKind};

async fn handle_hmr_events(hmr_manager: Arc<HmrManager>) {
    let mut rx = hmr_manager.subscribe();

    while let Ok(event) = rx.recv().await {
        match event.kind {
            HmrEventKind::Modified => {
                println!("📝 File modified: {:?}", event.path);
                
                // Custom handling
                if let Some(ext) = event.extension {
                    match ext.as_str() {
                        "css" | "scss" => {
                            println!("🎨 CSS change detected - hot reload");
                        }
                        "ts" | "tsx" | "js" | "jsx" => {
                            println!("📜 JavaScript change - full reload");
                        }
                        _ => {}
                    }
                }
            }
            HmrEventKind::Created => {
                println!("➕ File created: {:?}", event.path);
            }
            HmrEventKind::Deleted => {
                println!("🗑️  File deleted: {:?}", event.path);
            }
        }
    }
}
```

### Conditional HMR

Enable HMR only in development:

```rust
let is_dev = std::env::var("RUST_ENV")
    .unwrap_or_else(|_| "development".to_string()) == "development";

let hmr_config = HmrConfig::new()
    .enabled(is_dev)
    .watch_path(PathBuf::from("src"));

if is_dev {
    println!("🔥 Development mode - HMR enabled");
} else {
    println!("🚀 Production mode - HMR disabled");
}
```

### Multiple Watch Paths

Watch different directories for different frameworks:

```rust
let hmr_config = HmrConfig::new()
    // Backend Rust code
    .watch_path(PathBuf::from("src"))
    
    // Frontend React code
    .watch_path(PathBuf::from("client/src"))
    
    // Shared types
    .watch_path(PathBuf::from("shared"))
    
    // Static assets
    .watch_path(PathBuf::from("public"));
```

### Performance Tuning

```rust
let hmr_config = HmrConfig::new()
    // Higher debounce for slower machines
    .debounce(300) // Wait 300ms
    
    // Watch only specific extensions
    .watch_extension("tsx".to_string())
    .watch_extension("css".to_string())
    
    // Ignore more patterns
    .ignore_pattern("test".to_string())
    .ignore_pattern("spec".to_string())
    .ignore_pattern("__tests__".to_string());
```

## Troubleshooting

### HMR Not Working

**Problem:** Browser doesn't reload when files change.

**Solutions:**

1. **Check WebSocket Connection**
   ```javascript
   // Open browser console
   // Look for: "🔥 HMR Connected"
   // If not, check firewall/port settings
   ```

2. **Verify File Extensions**
   ```rust
   let hmr_config = HmrConfig::new()
       .watch_extension("tsx".to_string()) // Add your extension
       .verbose(true); // Enable logging
   ```

3. **Check Watch Paths**
   ```rust
   let hmr_config = HmrConfig::new()
       .watch_path(PathBuf::from("YOUR_ACTUAL_PATH"))
       .verbose(true);
   ```

### Too Many Reloads

**Problem:** Page reloads too frequently.

**Solution:** Increase debounce delay

```rust
let hmr_config = HmrConfig::new()
    .debounce(500); // Increase from default 100ms
```

### CSS Not Hot Reloading

**Problem:** CSS requires full page reload.

**Solution:** Ensure CSS files match watched extensions

```rust
let hmr_config = HmrConfig::new()
    .watch_extension("css".to_string())
    .watch_extension("scss".to_string())
    .watch_extension("less".to_string());
```

### WebSocket Port Conflict

**Problem:** Port already in use.

**Solution:** Change WebSocket port

```rust
let hmr_config = HmrConfig::new()
    .websocket_port(3002); // Use different port
```

## Best Practices

### 1. Development vs Production

```rust
#[cfg(debug_assertions)]
let hmr_enabled = true;

#[cfg(not(debug_assertions))]
let hmr_enabled = false;

let hmr_config = HmrConfig::new().enabled(hmr_enabled);
```

### 2. Optimize Watch Patterns

Don't watch everything:

```rust
// ❌ Bad - watches too much
let hmr_config = HmrConfig::new()
    .watch_path(PathBuf::from(".")); // Watches entire project!

// ✅ Good - specific paths
let hmr_config = HmrConfig::new()
    .watch_path(PathBuf::from("src"))
    .watch_path(PathBuf::from("client/src"));
```

### 3. Ignore Build Artifacts

```rust
let hmr_config = HmrConfig::new()
    .ignore_pattern("node_modules".to_string())
    .ignore_pattern("dist".to_string())
    .ignore_pattern("build".to_string())
    .ignore_pattern("target".to_string())
    .ignore_pattern(".next".to_string());
```

### 4. Use Environment Variables

```rust
let websocket_port = std::env::var("HMR_PORT")
    .ok()
    .and_then(|p| p.parse().ok())
    .unwrap_or(3001);

let hmr_config = HmrConfig::new()
    .websocket_port(websocket_port);
```

## Summary

Armature's HMR system provides:

✅ **Automatic file watching** with the `notify` crate  
✅ **WebSocket-based notifications** for instant updates  
✅ **CSS hot reload** without page refresh  
✅ **Smart debouncing** to prevent rapid-fire reloads  
✅ **Framework-agnostic** - works with React, Vue, Angular, Svelte  
✅ **Production-safe** - easily disabled for production builds  
✅ **Configurable** - watch paths, extensions, debounce, etc.  

**Perfect for rapid development with SSR frameworks!** 🔥🚀

