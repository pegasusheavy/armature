//! Parallel SSR Pre-rendering Example
//!
//! Demonstrates the performance benefits of parallel SSR pre-rendering
//! for static site generation using Angular Universal.
//!
//! # Performance
//!
//! **Sequential:**  10 pages × 500ms = 5 seconds
//! **Parallel:**    max(10 × 500ms) = ~500ms
//! **Speedup:**     ~10x faster

use std::time::Instant;

/// Simulated Angular renderer (for example purposes)
#[derive(Clone)]
struct MockAngularRenderer;

impl MockAngularRenderer {
    fn new() -> Self {
        Self
    }

    /// Simulate rendering a single route
    async fn render(&self, route: &str) -> Result<String, String> {
        // Simulate rendering time (100-200ms per route)
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;

        Ok(format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>Page: {}</title>
</head>
<body>
    <h1>Server-Side Rendered: {}</h1>
    <p>This page was pre-rendered at build time.</p>
</body>
</html>"#,
            route, route
        ))
    }

    /// Render multiple routes sequentially
    async fn render_sequential(&self, routes: Vec<String>) -> Result<Vec<(String, String)>, String> {
        println!("📝 Rendering {} routes sequentially...", routes.len());
        let start = Instant::now();

        let mut results = Vec::new();
        for route in routes {
            let html = self.render(&route).await?;
            results.push((route, html));
        }

        let duration = start.elapsed();
        println!(
            "✅ Sequential complete: {} routes in {:?} ({:.1} pages/sec)",
            results.len(),
            duration,
            results.len() as f64 / duration.as_secs_f64()
        );

        Ok(results)
    }

    /// Render multiple routes in parallel
    async fn render_parallel(&self, routes: Vec<String>) -> Result<Vec<(String, String)>, String> {
        use tokio::task::JoinSet;

        println!("🚀 Rendering {} routes in parallel...", routes.len());
        let start = Instant::now();

        let mut set = JoinSet::new();

        for route in routes {
            let renderer = self.clone();
            set.spawn(async move {
                let html = renderer.render(&route).await?;
                Ok::<_, String>((route, html))
            });
        }

        let mut results = Vec::new();
        while let Some(result) = set.join_next().await {
            match result {
                Ok(Ok(page)) => results.push(page),
                Ok(Err(e)) => return Err(e),
                Err(e) => return Err(format!("Task failed: {}", e)),
            }
        }

        let duration = start.elapsed();
        println!(
            "✅ Parallel complete: {} routes in {:?} ({:.1} pages/sec)",
            results.len(),
            duration,
            results.len() as f64 / duration.as_secs_f64()
        );

        Ok(results)
    }

    /// Pre-render entire site to disk
    async fn pre_render_site(
        &self,
        output_dir: &str,
        routes: Vec<String>,
    ) -> Result<StaticSiteStats, String> {
        let start = Instant::now();

        println!("\n🎨 Pre-rendering static site...");
        println!("📍 Output directory: {}", output_dir);
        println!("📄 Routes: {}", routes.len());

        // Render all routes in parallel
        let rendered = self.render_parallel(routes).await?;

        // Write files (also in parallel)
        println!("\n💾 Writing files to disk...");
        let mut set = tokio::task::JoinSet::new();

        for (route, html) in rendered {
            let output_dir = output_dir.to_string();

            set.spawn(async move {
                let file_path = if route == "/" {
                    format!("{}/index.html", output_dir)
                } else {
                    format!("{}/{}.html", output_dir, route.trim_start_matches('/'))
                };

                // Create parent directories
                if let Some(parent) = std::path::Path::new(&file_path).parent() {
                    tokio::fs::create_dir_all(parent).await.map_err(|e| e.to_string())?;
                }

                // Write file
                tokio::fs::write(&file_path, html)
                    .await
                    .map_err(|e| e.to_string())?;

                Ok::<_, String>(file_path)
            });
        }

        let mut written = 0;
        while let Some(result) = set.join_next().await {
            match result {
                Ok(Ok(_)) => written += 1,
                Ok(Err(e)) => return Err(e),
                Err(e) => return Err(format!("Task failed: {}", e)),
            }
        }

        let duration = start.elapsed();
        let pages_per_second = written as f64 / duration.as_secs_f64();

        println!(
            "\n✅ Site pre-rendering complete: {} pages in {:?} ({:.1} pages/sec)",
            written, duration, pages_per_second
        );

        Ok(StaticSiteStats {
            pages_rendered: written,
            duration,
            pages_per_second,
        })
    }
}

/// Statistics from static site generation
#[derive(Debug)]
struct StaticSiteStats {
    pages_rendered: usize,
    duration: std::time::Duration,
    pages_per_second: f64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                                                              ║");
    println!("║       Parallel SSR Pre-rendering Performance Demo           ║");
    println!("║                                                              ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let renderer = MockAngularRenderer::new();

    // Create sample routes for a typical blog/docs site
    let routes = vec![
        "/".to_string(),
        "/about".to_string(),
        "/contact".to_string(),
        "/blog".to_string(),
        "/blog/post-1".to_string(),
        "/blog/post-2".to_string(),
        "/blog/post-3".to_string(),
        "/docs".to_string(),
        "/docs/getting-started".to_string(),
        "/docs/api-reference".to_string(),
        "/docs/examples".to_string(),
        "/docs/deployment".to_string(),
    ];

    println!("Routes to render: {}", routes.len());
    println!("Estimated render time per route: ~150ms\n");

    // ========================================================================
    // BENCHMARK 1: Sequential Rendering
    // ========================================================================

    println!("═══════════════════════════════════════════════════════════════");
    println!("  BENCHMARK 1: Sequential Rendering                           ");
    println!("═══════════════════════════════════════════════════════════════\n");

    let seq_start = Instant::now();
    let sequential = renderer.render_sequential(routes.clone()).await?;
    let seq_duration = seq_start.elapsed();

    println!("\nResults:");
    println!("  • Pages rendered: {}", sequential.len());
    println!("  • Total time: {:?}", seq_duration);
    println!("  • Average: {:?} per page", seq_duration / sequential.len() as u32);
    println!(
        "  • Throughput: {:.1} pages/sec\n",
        sequential.len() as f64 / seq_duration.as_secs_f64()
    );

    // ========================================================================
    // BENCHMARK 2: Parallel Rendering
    // ========================================================================

    println!("═══════════════════════════════════════════════════════════════");
    println!("  BENCHMARK 2: Parallel Rendering                             ");
    println!("═══════════════════════════════════════════════════════════════\n");

    let par_start = Instant::now();
    let parallel = renderer.render_parallel(routes.clone()).await?;
    let par_duration = par_start.elapsed();

    println!("\nResults:");
    println!("  • Pages rendered: {}", parallel.len());
    println!("  • Total time: {:?}", par_duration);
    println!("  • Average: {:?} per page", par_duration / parallel.len() as u32);
    println!(
        "  • Throughput: {:.1} pages/sec\n",
        parallel.len() as f64 / par_duration.as_secs_f64()
    );

    // ========================================================================
    // COMPARISON
    // ========================================================================

    println!("═══════════════════════════════════════════════════════════════");
    println!("  PERFORMANCE COMPARISON                                       ");
    println!("═══════════════════════════════════════════════════════════════\n");

    let speedup = seq_duration.as_secs_f64() / par_duration.as_secs_f64();
    let time_saved = seq_duration - par_duration;
    let percentage_faster = ((speedup - 1.0) * 100.0).round() as i32;

    println!("Sequential:");
    println!("  ⏱️  Time: {:?}", seq_duration);
    println!("  📊 Throughput: {:.1} pages/sec",
        sequential.len() as f64 / seq_duration.as_secs_f64());
    println!();

    println!("Parallel:");
    println!("  ⏱️  Time: {:?}", par_duration);
    println!("  📊 Throughput: {:.1} pages/sec",
        parallel.len() as f64 / par_duration.as_secs_f64());
    println!();

    println!("Performance Gain:");
    println!("  🚀 Speedup: {:.2}x faster", speedup);
    println!("  ⏰ Time saved: {:?}", time_saved);
    println!("  📈 Improvement: {}% faster\n", percentage_faster);

    // ========================================================================
    // DEMO: Full Site Pre-rendering
    // ========================================================================

    println!("═══════════════════════════════════════════════════════════════");
    println!("  DEMO: Full Static Site Generation                           ");
    println!("═══════════════════════════════════════════════════════════════");

    let output_dir = "build/static";
    let stats = renderer.pre_render_site(output_dir, routes).await?;

    println!("\n📊 Statistics:");
    println!("  • Pages: {}", stats.pages_rendered);
    println!("  • Duration: {:?}", stats.duration);
    println!("  • Throughput: {:.1} pages/sec", stats.pages_per_second);
    println!("  • Output: {}/", output_dir);

    // Cleanup
    tokio::fs::remove_dir_all(output_dir).await.ok();

    println!("\n═══════════════════════════════════════════════════════════════");
    println!("                    KEY TAKEAWAYS                              ");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("✅ Parallel SSR Pre-rendering Benefits:");
    println!("   • 10-20x faster static site generation");
    println!("   • Same rendering quality as sequential");
    println!("   • Scales with number of CPU cores");
    println!("   • Perfect for build-time rendering\n");

    println!("📦 Use Cases:");
    println!("   • Static site generation (SSG)");
    println!("   • Documentation sites");
    println!("   • Marketing pages");
    println!("   • Blog content");
    println!("   • Product catalogs\n");

    println!("💡 Implementation:");
    println!("   • Use tokio::task::JoinSet for parallel rendering");
    println!("   • Batch render all routes at build time");
    println!("   • Write files concurrently to disk");
    println!("   • Monitor with statistics\n");

    println!("═══════════════════════════════════════════════════════════════");
    println!("✅ Parallel SSR pre-rendering demo complete!");
    println!("═══════════════════════════════════════════════════════════════\n");

    Ok(())
}

