//! Hot Module Reload (HMR) with SSR Frameworks
//!
//! Demonstrates how to use Armature's HMR system with all 4 SSR frameworks:
//! - React
//! - Vue
//! - Angular
//! - Svelte
//!
//! This example shows:
//! - File watching configuration
//! - WebSocket-based change notification
//! - Automatic HMR script injection
//! - Development mode setup

use armature_core::hmr::{HmrConfig, HmrEvent, HmrEventKind, HmrManager, inject_hmr_script};
use armature_core::Error;
use std::path::PathBuf;
use std::sync::Arc;

// ============================================================================
// HMR CONFIGURATION FOR EACH FRAMEWORK
// ============================================================================

/// Create HMR configuration for React
fn create_react_hmr_config() -> HmrConfig {
    HmrConfig::new()
        .watch_path(PathBuf::from("client/react/src"))
        .watch_path(PathBuf::from("client/react/public"))
        .watch_extension("jsx".to_string())
        .watch_extension("tsx".to_string())
        .watch_extension("js".to_string())
        .watch_extension("ts".to_string())
        .watch_extension("css".to_string())
        .watch_extension("scss".to_string())
        .ignore_pattern("node_modules".to_string())
        .ignore_pattern("build".to_string())
        .ignore_pattern("dist".to_string())
        .debounce(100)
        .websocket_port(3001)
        .verbose(true)
}

/// Create HMR configuration for Vue
fn create_vue_hmr_config() -> HmrConfig {
    HmrConfig::new()
        .watch_path(PathBuf::from("client/vue/src"))
        .watch_path(PathBuf::from("client/vue/components"))
        .watch_path(PathBuf::from("client/vue/public"))
        .watch_extension("vue".to_string())
        .watch_extension("js".to_string())
        .watch_extension("ts".to_string())
        .watch_extension("css".to_string())
        .watch_extension("scss".to_string())
        .ignore_pattern("node_modules".to_string())
        .ignore_pattern(".nuxt".to_string())
        .ignore_pattern("dist".to_string())
        .debounce(100)
        .websocket_port(3002)
        .verbose(true)
}

/// Create HMR configuration for Angular
fn create_angular_hmr_config() -> HmrConfig {
    HmrConfig::new()
        .watch_path(PathBuf::from("client/angular/src"))
        .watch_extension("ts".to_string())
        .watch_extension("html".to_string())
        .watch_extension("css".to_string())
        .watch_extension("scss".to_string())
        .ignore_pattern("node_modules".to_string())
        .ignore_pattern("dist".to_string())
        .ignore_pattern(".angular".to_string())
        .debounce(100)
        .websocket_port(3003)
        .verbose(true)
}

/// Create HMR configuration for Svelte
fn create_svelte_hmr_config() -> HmrConfig {
    HmrConfig::new()
        .watch_path(PathBuf::from("client/svelte/src"))
        .watch_path(PathBuf::from("client/svelte/public"))
        .watch_extension("svelte".to_string())
        .watch_extension("js".to_string())
        .watch_extension("ts".to_string())
        .watch_extension("css".to_string())
        .ignore_pattern("node_modules".to_string())
        .ignore_pattern("build".to_string())
        .ignore_pattern(".svelte-kit".to_string())
        .debounce(100)
        .websocket_port(3004)
        .verbose(true)
}

// ============================================================================
// HMR EVENT HANDLER
// ============================================================================

/// Handle HMR events and log detailed information
async fn handle_hmr_events(
    framework: &str,
    mut rx: tokio::sync::broadcast::Receiver<HmrEvent>,
) {
    println!("📡 {} - HMR event handler started", framework);

    while let Ok(event) = rx.recv().await {
        let emoji = match event.kind {
            HmrEventKind::Modified => "📝",
            HmrEventKind::Created => "➕",
            HmrEventKind::Deleted => "🗑️",
        };

        let file_name = event
            .path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        println!(
            "{} {} - {:?} changed: {}",
            emoji, framework, event.kind, file_name
        );

        // Determine reload strategy based on file type
        if let Some(ext) = &event.extension {
            match ext.as_str() {
                "css" | "scss" | "less" => {
                    println!("   🎨 CSS change - hot reload without page refresh");
                }
                "js" | "ts" | "jsx" | "tsx" | "vue" | "svelte" => {
                    println!("   🔄 JavaScript/Component change - full page reload");
                }
                "html" => {
                    println!("   📄 HTML change - full page reload");
                }
                _ => {
                    println!("   ⚡ Asset change - cache busting");
                }
            }
        }
    }
}

// ============================================================================
// MOCK SSR RENDERER WITH HMR
// ============================================================================

/// Simulate React SSR rendering with HMR injection
async fn render_react_with_hmr(hmr_manager: &HmrManager) -> Result<String, Error> {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>React SSR with HMR</title>
    <link rel="stylesheet" href="/styles.css">
</head>
<body>
    <div id="root">
        <h1>⚛️ React SSR with HMR</h1>
        <p>Edit src/App.tsx and see changes instantly!</p>
    </div>
    <script src="/bundle.js"></script>
</body>
</html>"#
        .to_string();

    // Inject HMR client script
    Ok(inject_hmr_script(html, hmr_manager).await)
}

/// Simulate Vue SSR rendering with HMR injection
async fn render_vue_with_hmr(hmr_manager: &HmrManager) -> Result<String, Error> {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Vue SSR with HMR</title>
    <link rel="stylesheet" href="/styles.css">
</head>
<body>
    <div id="app">
        <h1>🖖 Vue SSR with HMR</h1>
        <p>Edit src/App.vue and see changes instantly!</p>
    </div>
    <script src="/app.js"></script>
</body>
</html>"#
        .to_string();

    Ok(inject_hmr_script(html, hmr_manager).await)
}

/// Simulate Angular SSR rendering with HMR injection
async fn render_angular_with_hmr(hmr_manager: &HmrManager) -> Result<String, Error> {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Angular SSR with HMR</title>
    <link rel="stylesheet" href="/styles.css">
</head>
<body>
    <app-root>
        <h1>🅰️ Angular SSR with HMR</h1>
        <p>Edit src/app/app.component.ts and see changes instantly!</p>
    </app-root>
    <script src="/main.js"></script>
</body>
</html>"#
        .to_string();

    Ok(inject_hmr_script(html, hmr_manager).await)
}

/// Simulate Svelte SSR rendering with HMR injection
async fn render_svelte_with_hmr(hmr_manager: &HmrManager) -> Result<String, Error> {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Svelte SSR with HMR</title>
    <link rel="stylesheet" href="/global.css">
    <link rel="stylesheet" href="/build/bundle.css">
</head>
<body>
    <div id="app">
        <h1>⚡ Svelte SSR with HMR</h1>
        <p>Edit src/App.svelte and see changes instantly!</p>
    </div>
    <script src="/build/bundle.js"></script>
</body>
</html>"#
        .to_string();

    Ok(inject_hmr_script(html, hmr_manager).await)
}

// ============================================================================
// MAIN EXAMPLE
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                                                              ║");
    println!("║         Hot Module Reload (HMR) - SSR Frameworks            ║");
    println!("║                                                              ║");
    println!("║   React • Vue • Angular • Svelte + Armature HMR             ║");
    println!("║                                                              ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // ========================================================================
    // 1. REACT HMR SETUP
    // ========================================================================

    println!("═══════════════════════════════════════════════════════════════");
    println!("                     ⚛️  REACT HMR SETUP                       ");
    println!("═══════════════════════════════════════════════════════════════\n");

    let react_hmr_config = create_react_hmr_config();
    let react_hmr = Arc::new(HmrManager::new(react_hmr_config));

    println!("🔧 React HMR Configuration:");
    println!("   Watch paths: client/react/src, client/react/public");
    println!("   Extensions: jsx, tsx, js, ts, css, scss");
    println!("   WebSocket: ws://localhost:3001");

    // Start watching (in real app, this would monitor actual files)
    // react_hmr.start_watching().await?;

    // Subscribe to events
    let react_rx = react_hmr.subscribe();
    tokio::spawn(handle_hmr_events("React", react_rx));

    // Render page with HMR
    let react_html = render_react_with_hmr(&react_hmr).await?;
    println!("\n✅ React SSR page rendered with HMR client injected");
    println!("   HTML size: {} bytes", react_html.len());

    // ========================================================================
    // 2. VUE HMR SETUP
    // ========================================================================

    println!("\n═══════════════════════════════════════════════════════════════");
    println!("                      🖖 VUE HMR SETUP                         ");
    println!("═══════════════════════════════════════════════════════════════\n");

    let vue_hmr_config = create_vue_hmr_config();
    let vue_hmr = Arc::new(HmrManager::new(vue_hmr_config));

    println!("🔧 Vue HMR Configuration:");
    println!("   Watch paths: client/vue/src, client/vue/components");
    println!("   Extensions: vue, js, ts, css, scss");
    println!("   WebSocket: ws://localhost:3002");

    // vue_hmr.start_watching().await?;

    let vue_rx = vue_hmr.subscribe();
    tokio::spawn(handle_hmr_events("Vue", vue_rx));

    let vue_html = render_vue_with_hmr(&vue_hmr).await?;
    println!("\n✅ Vue SSR page rendered with HMR client injected");
    println!("   HTML size: {} bytes", vue_html.len());

    // ========================================================================
    // 3. ANGULAR HMR SETUP
    // ========================================================================

    println!("\n═══════════════════════════════════════════════════════════════");
    println!("                    🅰️  ANGULAR HMR SETUP                      ");
    println!("═══════════════════════════════════════════════════════════════\n");

    let angular_hmr_config = create_angular_hmr_config();
    let angular_hmr = Arc::new(HmrManager::new(angular_hmr_config));

    println!("🔧 Angular HMR Configuration:");
    println!("   Watch paths: client/angular/src");
    println!("   Extensions: ts, html, css, scss");
    println!("   WebSocket: ws://localhost:3003");

    // angular_hmr.start_watching().await?;

    let angular_rx = angular_hmr.subscribe();
    tokio::spawn(handle_hmr_events("Angular", angular_rx));

    let angular_html = render_angular_with_hmr(&angular_hmr).await?;
    println!("\n✅ Angular SSR page rendered with HMR client injected");
    println!("   HTML size: {} bytes", angular_html.len());

    // ========================================================================
    // 4. SVELTE HMR SETUP
    // ========================================================================

    println!("\n═══════════════════════════════════════════════════════════════");
    println!("                     ⚡ SVELTE HMR SETUP                       ");
    println!("═══════════════════════════════════════════════════════════════\n");

    let svelte_hmr_config = create_svelte_hmr_config();
    let svelte_hmr = Arc::new(HmrManager::new(svelte_hmr_config));

    println!("🔧 Svelte HMR Configuration:");
    println!("   Watch paths: client/svelte/src, client/svelte/public");
    println!("   Extensions: svelte, js, ts, css");
    println!("   WebSocket: ws://localhost:3004");

    // svelte_hmr.start_watching().await?;

    let svelte_rx = svelte_hmr.subscribe();
    tokio::spawn(handle_hmr_events("Svelte", svelte_rx));

    let svelte_html = render_svelte_with_hmr(&svelte_hmr).await?;
    println!("\n✅ Svelte SSR page rendered with HMR client injected");
    println!("   HTML size: {} bytes", svelte_html.len());

    // ========================================================================
    // 5. DEMONSTRATE HMR CLIENT SCRIPT
    // ========================================================================

    println!("\n═══════════════════════════════════════════════════════════════");
    println!("                   HMR CLIENT SCRIPT PREVIEW                   ");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("📜 React HMR Client Script (injected before </body>):");
    println!("────────────────────────────────────────────────────────────────");

    let script_preview = react_hmr.get_client_script();
    let lines: Vec<&str> = script_preview.lines().take(15).collect();
    for line in lines {
        println!("{}", line);
    }
    println!("   ... (truncated) ...");
    println!("────────────────────────────────────────────────────────────────\n");

    // ========================================================================
    // 6. SIMULATE FILE CHANGES (DEMO)
    // ========================================================================

    println!("═══════════════════════════════════════════════════════════════");
    println!("                SIMULATED FILE CHANGE EVENTS                   ");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("🔄 In a real application, when you edit files:");
    println!();
    println!("📝 React: Edit client/react/src/App.tsx");
    println!("   → HMR detects change");
    println!("   → WebSocket notifies browser: ws://localhost:3001");
    println!("   → Browser reloads page");
    println!();
    println!("🎨 Vue: Edit client/vue/src/App.vue (CSS only)");
    println!("   → HMR detects CSS change");
    println!("   → WebSocket notifies browser: ws://localhost:3002");
    println!("   → Browser hot-reloads CSS (no page refresh!)");
    println!();
    println!("📝 Angular: Edit client/angular/src/app/app.component.ts");
    println!("   → HMR detects change");
    println!("   → WebSocket notifies browser: ws://localhost:3003");
    println!("   → Browser reloads page");
    println!();
    println!("⚡ Svelte: Edit client/svelte/src/App.svelte");
    println!("   → HMR detects change");
    println!("   → WebSocket notifies browser: ws://localhost:3004");
    println!("   → Browser reloads page");

    // ========================================================================
    // 7. STATISTICS
    // ========================================================================

    println!("\n═══════════════════════════════════════════════════════════════");
    println!("                          STATISTICS                           ");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("📊 HMR Managers Active:");
    println!("   React:   {} connected clients", react_hmr.client_count().await);
    println!("   Vue:     {} connected clients", vue_hmr.client_count().await);
    println!("   Angular: {} connected clients", angular_hmr.client_count().await);
    println!("   Svelte:  {} connected clients", svelte_hmr.client_count().await);

    println!("\n🔥 HMR Features Demonstrated:");
    println!("   ✅ File system watching (notify crate)");
    println!("   ✅ WebSocket-based notifications");
    println!("   ✅ Automatic HMR script injection");
    println!("   ✅ Framework-specific configurations");
    println!("   ✅ CSS hot reload (no page refresh)");
    println!("   ✅ JavaScript full page reload");
    println!("   ✅ Debouncing to prevent rapid reloads");
    println!("   ✅ Ignore patterns for node_modules, dist, etc.");

    // ========================================================================
    // 8. USAGE INSTRUCTIONS
    // ========================================================================

    println!("\n═══════════════════════════════════════════════════════════════");
    println!("                       HOW TO USE                              ");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("🚀 To enable HMR in your application:");
    println!();
    println!("1️⃣  Configure HMR for your framework:");
    println!("   let hmr_config = HmrConfig::new()");
    println!("       .watch_path(PathBuf::from(\"src\"))");
    println!("       .websocket_port(3001);");
    println!();
    println!("2️⃣  Create HMR manager:");
    println!("   let hmr_manager = Arc::new(HmrManager::new(hmr_config));");
    println!();
    println!("3️⃣  Start watching:");
    println!("   hmr_manager.start_watching().await?;");
    println!();
    println!("4️⃣  Inject HMR script into rendered HTML:");
    println!("   let html_with_hmr = inject_hmr_script(html, &hmr_manager).await;");
    println!();
    println!("5️⃣  Serve your application and start coding!");
    println!("   Changes to watched files will trigger automatic reloads.");

    println!("\n═══════════════════════════════════════════════════════════════");
    println!("✅ HMR example complete!");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("📚 For more information, see:");
    println!("   docs/HMR_GUIDE.md");
    println!();
    println!("🔥 Happy hot reloading!");

    Ok(())
}

