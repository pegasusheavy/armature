import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { DocPageComponent, DocPage } from '../../shared/doc-page.component';

@Component({
  selector: 'app-profiling-guide',
  standalone: true,
  imports: [CommonModule, DocPageComponent],
  template: `<app-doc-page [page]="page"></app-doc-page>`
})
export class ProfilingGuideComponent {
  page: DocPage = {
    title: 'CPU Profiling',
    subtitle: 'Profile your Armature application to identify performance bottlenecks and generate interactive flamegraphs.',
    icon: '🔬',
    badge: 'Performance',
    features: [
      { icon: '🔥', title: 'Flamegraphs', description: 'Interactive CPU visualization' },
      { icon: '📊', title: 'Sampling Profiler', description: '1000 Hz CPU sampling' },
      { icon: '🎯', title: 'Hotspot Detection', description: 'Find slow functions' },
      { icon: '📈', title: 'Performance Analysis', description: 'Optimize critical paths' }
    ],
    sections: [
      {
        id: 'overview',
        title: 'Overview',
        content: `<p>CPU profiling helps you understand where your application spends time. Armature includes built-in profiling support using <code>pprof</code> that generates interactive flamegraphs.</p>
        <p>A <strong>flamegraph</strong> is a visualization where:</p>
        <ul>
          <li>Each box represents a function in the call stack</li>
          <li>The width of a box shows how much CPU time was spent in that function</li>
          <li>Boxes are stacked to show the call hierarchy</li>
          <li>Wider boxes = more time = potential optimization targets</li>
        </ul>`
      },
      {
        id: 'quick-start',
        title: 'Quick Start',
        content: `<p>Run the built-in profiling server example:</p>`,
        codeBlocks: [
          {
            language: 'bash',
            code: `# Run the profiling server in release mode
cargo run --example profiling_server --release

# In another terminal, generate load
for i in {1..1000}; do
  curl -s http://localhost:PORT/tasks > /dev/null
done

# Press Ctrl+C to stop and generate flamegraph
# Open flamegraph-profile.svg in your browser`
          }
        ]
      },
      {
        id: 'setup',
        title: 'Adding Profiling to Your App',
        content: `<p>Add the profiling dependencies to your <code>Cargo.toml</code>:</p>`,
        codeBlocks: [
          {
            language: 'toml',
            filename: 'Cargo.toml',
            code: `[dev-dependencies]
pprof = { version = "0.14", features = ["flamegraph", "criterion", "prost-codec"] }
ctrlc = "3.4"

# Enable debug symbols in release for better stack traces
[profile.profiling]
inherits = "release"
debug = true`
          }
        ]
      },
      {
        id: 'integration',
        title: 'Integrating the Profiler',
        content: `<p>Wrap your application with the profiler:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `use pprof::ProfilerGuardBuilder;
use std::fs::File;

#[tokio::main]
async fn main() {
    // Start the CPU profiler
    let guard = ProfilerGuardBuilder::default()
        .frequency(1000)  // Sample 1000 times per second
        .blocklist(&["libc", "libgcc", "pthread", "vdso"])
        .build()
        .expect("Failed to create profiler");

    println!("Profiler started (1000 Hz sampling)");

    // Set up Ctrl+C handler to generate flamegraph on exit
    let guard = std::sync::Arc::new(std::sync::Mutex::new(Some(guard)));
    let guard_clone = guard.clone();

    ctrlc::set_handler(move || {
        if let Some(guard) = guard_clone.lock().unwrap().take() {
            println!("Generating flamegraph...");

            if let Ok(report) = guard.report().build() {
                let file = File::create("flamegraph.svg").unwrap();
                report.flamegraph(file).unwrap();
                println!("Saved to flamegraph.svg");
            }
        }
        std::process::exit(0);
    }).unwrap();

    // Run your application
    let app = Application::create::<AppModule>().await;
    app.listen(3000).await.unwrap();
}`
          }
        ]
      },
      {
        id: 'load-testing',
        title: 'Generating Load',
        content: `<p>To get meaningful profiling data, you need to generate realistic load:</p>`,
        codeBlocks: [
          {
            language: 'bash',
            code: `# Simple load with curl
for i in {1..1000}; do
  curl -s http://localhost:3000/api/users > /dev/null
done

# Using wrk (recommended for high load)
wrk -t4 -c100 -d30s http://localhost:3000/api/users

# Using hey
hey -n 10000 -c 100 http://localhost:3000/api/users

# Using ab (Apache Bench)
ab -n 10000 -c 100 http://localhost:3000/api/users`
          }
        ]
      },
      {
        id: 'reading-flamegraphs',
        title: 'Reading Flamegraphs',
        content: `<p>Understanding the flamegraph output:</p>
        <ul>
          <li><strong>X-axis:</strong> Stack frames sorted alphabetically (not time order)</li>
          <li><strong>Y-axis:</strong> Stack depth (callers below, callees above)</li>
          <li><strong>Width:</strong> Percentage of total CPU time</li>
          <li><strong>Color:</strong> Random (for visual distinction)</li>
        </ul>
        <p>Look for:</p>
        <ul>
          <li><strong>Wide plateaus</strong> — Functions using lots of CPU</li>
          <li><strong>Deep stacks</strong> — Complex call chains</li>
          <li><strong>Your code</strong> — Search for your module names</li>
        </ul>
        <p>Common hotspots in web servers:</p>
        <ul>
          <li><code>serde_json::*</code> — JSON serialization/deserialization</li>
          <li><code>httparse::*</code> — HTTP request parsing</li>
          <li><code>tokio::*</code> — Async runtime overhead</li>
          <li><code>hyper::*</code> — HTTP protocol handling</li>
        </ul>`
      },
      {
        id: 'profile-script',
        title: 'Automated Profiling Script',
        content: `<p>Use the included profiling script for convenience:</p>`,
        codeBlocks: [
          {
            language: 'bash',
            code: `# Run profiling for 30 seconds (default)
./scripts/profile.sh

# Run profiling for 60 seconds
./scripts/profile.sh 60

# The script will:
# 1. Build in release mode with debug symbols
# 2. Start the profiling server
# 3. Generate load for the specified duration
# 4. Stop the server and generate flamegraph
# 5. Output the path to the SVG file`
          }
        ]
      },
      {
        id: 'criterion-integration',
        title: 'Profiling Benchmarks',
        content: `<p>Integrate profiling with Criterion benchmarks:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            filename: 'benches/my_benchmark.rs',
            code: `use criterion::{criterion_group, criterion_main, Criterion};
use pprof::criterion::{PProfProfiler, Output};

fn my_benchmark(c: &mut Criterion) {
    c.bench_function("my_function", |b| {
        b.iter(|| {
            // Your code to benchmark
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = my_benchmark
}
criterion_main!(benches);`
          },
          {
            language: 'bash',
            code: `# Run benchmark with profiling
cargo bench --bench my_benchmark

# Flamegraphs are saved in target/criterion/*/profile/`
          }
        ]
      },
      {
        id: 'optimization-tips',
        title: 'Optimization Tips',
        content: `<p>Common optimizations based on profiling results:</p>
        <table>
          <thead>
            <tr>
              <th>Hotspot</th>
              <th>Optimization</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td><code>serde_json</code></td>
              <td>Use <code>simd-json</code> or <code>sonic-rs</code> for faster JSON</td>
            </tr>
            <tr>
              <td><code>String::clone</code></td>
              <td>Use <code>&str</code> or <code>Arc&lt;str&gt;</code> to avoid cloning</td>
            </tr>
            <tr>
              <td><code>Vec::push</code></td>
              <td>Pre-allocate with <code>Vec::with_capacity</code></td>
            </tr>
            <tr>
              <td><code>HashMap</code></td>
              <td>Use <code>hashbrown</code> or <code>indexmap</code></td>
            </tr>
            <tr>
              <td>Regex compilation</td>
              <td>Compile once with <code>lazy_static</code> or <code>once_cell</code></td>
            </tr>
            <tr>
              <td>Database queries</td>
              <td>Add indexes, use connection pooling</td>
            </tr>
          </tbody>
        </table>`
      },
      {
        id: 'best-practices',
        title: 'Best Practices',
        content: `<ul>
          <li><strong>Profile in release mode</strong> — Debug builds are not representative</li>
          <li><strong>Enable debug symbols</strong> — Use <code>[profile.profiling]</code> for readable stack traces</li>
          <li><strong>Generate realistic load</strong> — Use production-like request patterns</li>
          <li><strong>Profile for adequate duration</strong> — At least 30 seconds for stable results</li>
          <li><strong>Compare before/after</strong> — Save flamegraphs to track improvements</li>
          <li><strong>Focus on the biggest wins</strong> — Optimize the widest bars first</li>
          <li><strong>Profile regularly</strong> — Catch regressions early</li>
        </ul>`
      }
    ],
    relatedDocs: [
      { id: 'metrics-guide', title: 'Metrics', description: 'Runtime performance metrics' },
      { id: 'testing-guide', title: 'Testing', description: 'Benchmark testing' }
    ],
    seeAlso: [
      { title: 'Grafana Dashboards', id: 'grafana-dashboards' },
      { title: 'OpenTelemetry', id: 'opentelemetry-guide' }
    ]
  };
}

