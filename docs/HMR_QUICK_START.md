# HMR Quick Start

Get hot module reload working in 5 minutes! ⚡

## Installation

HMR is built into `armature-core`, no additional dependencies needed.

```toml
[dependencies]
armature = "0.1"
armature-core = "0.1"
```

## Quick Start

### 1. Basic Setup (3 lines)

```rust
use armature::prelude::*;
use armature::hmr::{HmrConfig, HmrManager, inject_hmr_script};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create HMR config
    let hmr_config = HmrConfig::new()
        .watch_path(PathBuf::from("src"));

    // 2. Create HMR manager
    let hmr_manager = Arc::new(HmrManager::new(hmr_config));

    // 3. Start watching
    hmr_manager.start_watching().await?;

    // Your app here...
    Ok(())
}
```

### 2. Inject HMR Script

```rust
// In your SSR render function:
async fn render_page(hmr_manager: &HmrManager) -> String {
    let html = "<html><body><h1>Hello</h1></body></html>".to_string();
    
    // Inject HMR client
    inject_hmr_script(html, hmr_manager).await
}
```

**That's it!** 🎉 Your app now has hot reload.

## Framework-Specific

### React

```rust
let hmr_config = HmrConfig::new()
    .watch_path(PathBuf::from("client/src"))
    .watch_extension("jsx".to_string())
    .watch_extension("tsx".to_string())
    .websocket_port(3001);
```

### Vue

```rust
let hmr_config = HmrConfig::new()
    .watch_path(PathBuf::from("client/src"))
    .watch_extension("vue".to_string())
    .websocket_port(3002);
```

### Angular

```rust
let hmr_config = HmrConfig::new()
    .watch_path(PathBuf::from("client/src"))
    .watch_extension("ts".to_string())
    .watch_extension("html".to_string())
    .websocket_port(3003);
```

### Svelte

```rust
let hmr_config = HmrConfig::new()
    .watch_path(PathBuf::from("client/src"))
    .watch_extension("svelte".to_string())
    .websocket_port(3004);
```

## What You Get

✅ **Automatic file watching**  
✅ **WebSocket notifications**  
✅ **CSS hot reload** (no page refresh)  
✅ **JavaScript full reload**  
✅ **Automatic browser connection**  
✅ **Reconnection on disconnect**  

## Development vs Production

```rust
#[cfg(debug_assertions)]
let hmr_enabled = true;

#[cfg(not(debug_assertions))]
let hmr_enabled = false;

let hmr_config = HmrConfig::new().enabled(hmr_enabled);
```

## Common Patterns

### Watch Multiple Paths

```rust
let hmr_config = HmrConfig::new()
    .watch_path(PathBuf::from("src"))
    .watch_path(PathBuf::from("client"))
    .watch_path(PathBuf::from("public"));
```

### Watch Specific Extensions

```rust
let hmr_config = HmrConfig::new()
    .watch_extension("ts".to_string())
    .watch_extension("tsx".to_string())
    .watch_extension("css".to_string());
```

### Ignore Patterns

```rust
let hmr_config = HmrConfig::new()
    .ignore_pattern("node_modules".to_string())
    .ignore_pattern("dist".to_string());
```

### Custom Debounce

```rust
let hmr_config = HmrConfig::new()
    .debounce(200); // Wait 200ms before reload
```

## Troubleshooting

### Not Working?

1. Check browser console: Look for "🔥 HMR Connected"
2. Enable verbose mode: `.verbose(true)`
3. Verify watch paths exist
4. Check WebSocket port isn't blocked

### Too Many Reloads?

```rust
let hmr_config = HmrConfig::new()
    .debounce(500); // Increase delay
```

### CSS Not Hot Reloading?

```rust
let hmr_config = HmrConfig::new()
    .watch_extension("css".to_string())
    .watch_extension("scss".to_string());
```

## Examples

```bash
# Run HMR example
cargo run --example hmr_ssr_frameworks

# See full guide
cat docs/HMR_GUIDE.md
```

## Summary

**3 steps to HMR:**
1. Create `HmrConfig`
2. Create `HmrManager`
3. Call `start_watching()`

**Then inject script:**
```rust
inject_hmr_script(html, &hmr_manager).await
```

Done! 🔥🚀

## Full Guide

For complete documentation, see [HMR_GUIDE.md](HMR_GUIDE.md)

