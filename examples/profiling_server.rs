#![allow(clippy::all)]
//! Profiling Server Example
//!
//! This example runs an Armature server with CPU profiling enabled.
//! After running, it generates a flamegraph showing where CPU time is spent.
//!
//! **Note:** This example only works on Unix platforms (Linux, macOS).
//!
//! # Usage
//!
//! ```bash
//! # Run the profiling server
//! cargo run --example profiling_server --release
//!
//! # In another terminal, generate load:
//! wrk -t4 -c100 -d30s http://localhost:3000/tasks
//!
//! # Or use curl in a loop
//! for i in {1..1000}; do curl -s http://localhost:3000/tasks > /dev/null; done
//! ```
//!
//! After stopping the server (Ctrl+C), a flamegraph will be generated at
//! `flamegraph-profile.svg`

// This example only works on Unix platforms (pprof requires Unix APIs)
#[cfg(not(unix))]
fn main() {
    eprintln!("This example only works on Unix platforms (Linux, macOS).");
    eprintln!("The pprof crate requires Unix-specific APIs for CPU profiling.");
    std::process::exit(1);
}

#[cfg(unix)]
#[path = "common/mod.rs"]
mod common;

#[cfg(unix)]
fn main() {
    profiling_main();
}

#[cfg(unix)]
#[allow(dead_code)]
fn profiling_main() {
    use armature::prelude::*;
    use common::find_available_port;
    use pprof::ProfilerGuardBuilder;
    use serde::{Deserialize, Serialize};
    use std::collections::hash_map::DefaultHasher;
    use std::fs::File;
    use std::hash::{Hash, Hasher};
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::{Arc, RwLock};
    use std::time::Instant;

    // ============================================================================
    // Models
    // ============================================================================

    #[derive(Clone, Serialize, Deserialize)]
    struct Task {
        id: u32,
        title: String,
        completed: bool,
        hash: Option<String>,
    }

    #[derive(Deserialize)]
    struct CreateTask {
        title: String,
        completed: Option<bool>,
    }

    // ============================================================================
    // Service with CPU-intensive operations for profiling
    // ============================================================================

    #[derive(Default, Clone)]
    struct TaskService {
        tasks: Arc<RwLock<Vec<Task>>>,
        counter: Arc<AtomicU32>,
    }

    impl TaskService {
        fn new() -> Self {
            Self {
                tasks: Arc::new(RwLock::new(vec![
                    Task {
                        id: 1,
                        title: "Learn Rust".to_string(),
                        completed: true,
                        hash: None,
                    },
                    Task {
                        id: 2,
                        title: "Build with Armature".to_string(),
                        completed: false,
                        hash: None,
                    },
                    Task {
                        id: 3,
                        title: "Profile the application".to_string(),
                        completed: false,
                        hash: None,
                    },
                ])),
                counter: Arc::new(AtomicU32::new(4)),
            }
        }

        fn get_all(&self) -> Vec<Task> {
            self.tasks.read().unwrap().clone()
        }

        fn get_by_id(&self, id: u32) -> Option<Task> {
            self.tasks
                .read()
                .unwrap()
                .iter()
                .find(|t| t.id == id)
                .cloned()
        }

        fn create(&self, title: String, completed: bool) -> Task {
            let id = self.counter.fetch_add(1, Ordering::SeqCst);
            let task = Task {
                id,
                title,
                completed,
                hash: Some(self.compute_hash(id)),
            };
            self.tasks.write().unwrap().push(task.clone());
            task
        }

        fn update(&self, id: u32, title: String, completed: bool) -> Option<Task> {
            let mut tasks = self.tasks.write().unwrap();
            if let Some(task) = tasks.iter_mut().find(|t| t.id == id) {
                task.title = title;
                task.completed = completed;
                task.hash = Some(self.compute_hash(id));
                Some(task.clone())
            } else {
                None
            }
        }

        fn delete(&self, id: u32) -> bool {
            let mut tasks = self.tasks.write().unwrap();
            if let Some(pos) = tasks.iter().position(|t| t.id == id) {
                tasks.remove(pos);
                true
            } else {
                false
            }
        }

        // CPU-intensive operation for profiling demonstration
        fn compute_hash(&self, seed: u32) -> String {
            let mut result = String::new();
            let data = format!("task-{}-data", seed);

            // Do some work to make this show up in the profiler
            for i in 0..50 {
                let mut hasher = DefaultHasher::new();
                data.hash(&mut hasher);
                (i as u64).hash(&mut hasher);
                result.push_str(&format!("{:x}", hasher.finish()));
            }

            result[..32].to_string()
        }

        // More CPU-intensive operation
        fn compute_heavy(&self, iterations: u32) -> String {
            let mut result: u64 = 0;
            for i in 0..iterations {
                let mut hasher = DefaultHasher::new();
                i.hash(&mut hasher);
                result ^= hasher.finish();
            }
            format!("{:016x}", result)
        }
    }

    // ============================================================================
    // Runtime setup
    // ============================================================================

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let port = find_available_port();

        println!("🔬 Armature Profiling Server");
        println!("============================");
        println!();

        // Start CPU profiler
        println!("Starting CPU profiler (1000 Hz sampling)...");
        let guard = ProfilerGuardBuilder::default()
            .frequency(1000)
            .blocklist(&["libc", "libgcc", "pthread", "vdso"])
            .build()
            .expect("Failed to create profiler");

        println!("✅ Profiler started");
        println!();
        println!("📡 Server: http://localhost:{}", port);
        println!();
        println!("Endpoints:");
        println!("  GET  /tasks           - List all tasks");
        println!("  GET  /tasks/:id       - Get task by ID");
        println!("  POST /tasks           - Create task");
        println!("  GET  /compute/light   - Light CPU work");
        println!("  GET  /compute/heavy/N - Heavy CPU work (N iterations)");
        println!();
        println!("Generate load with:");
        println!("  curl http://localhost:{}/tasks", port);
        println!(
            "  for i in {{1..1000}}; do curl -s http://localhost:{}/compute/light > /dev/null; done",
            port
        );
        println!();
        println!("Press Ctrl+C to stop and generate flamegraph...");
        println!();

        let start_time = Instant::now();

        // Set up Ctrl+C handler
        let guard_ref = std::sync::Arc::new(std::sync::Mutex::new(Some(guard)));
        let guard_clone = guard_ref.clone();

        ctrlc::set_handler(move || {
            println!();
            println!("🛑 Stopping server...");

            let elapsed = start_time.elapsed();
            println!("⏱️  Server ran for {:.2}s", elapsed.as_secs_f64());

            // Generate flamegraph
            if let Some(guard) = guard_clone.lock().unwrap().take() {
                println!();
                println!("📊 Generating flamegraph...");

                match guard.report().build() {
                    Ok(report) => {
                        // Save SVG flamegraph
                        match File::create("flamegraph-profile.svg") {
                            Ok(file) => {
                                if report.flamegraph(file).is_ok() {
                                    println!("✅ Flamegraph saved: flamegraph-profile.svg");
                                    println!();
                                    println!(
                                        "Open the SVG file in a browser to explore the flamegraph."
                                    );
                                    println!("Wider bars = more CPU time spent in that function.");
                                }
                            }
                            Err(e) => println!("❌ Failed to create file: {}", e),
                        }
                    }
                    Err(e) => println!("❌ Failed to generate report: {}", e),
                }
            }

            println!();
            println!("🎉 Done!");
            std::process::exit(0);
        })
        .expect("Failed to set Ctrl+C handler");

        // Create a simple router with tasks and compute endpoints
        let mut router = Router::new();
        let service = TaskService::new();
        let service_clone = service.clone();

        // GET /tasks
        let svc = service.clone();
        router.add_route(Route {
            method: HttpMethod::GET,
            path: "/tasks".to_string(),
            handler: Arc::new(move |_req| {
                let svc = svc.clone();
                Box::pin(async move { HttpResponse::json(&svc.get_all()) })
            }),
            constraints: None,
        });

        // GET /tasks/:id
        let svc = service.clone();
        router.add_route(Route {
            method: HttpMethod::GET,
            path: "/tasks/:id".to_string(),
            handler: Arc::new(move |req| {
                let svc = svc.clone();
                Box::pin(async move {
                    let id_str = req
                        .param("id")
                        .ok_or_else(|| Error::Validation("Missing id".to_string()))?;
                    let id: u32 = id_str
                        .parse()
                        .map_err(|_| Error::Validation("Invalid id".to_string()))?;

                    let task = svc
                        .get_by_id(id)
                        .ok_or_else(|| Error::RouteNotFound("Task not found".to_string()))?;

                    HttpResponse::json(&task)
                })
            }),
            constraints: None,
        });

        // POST /tasks
        let svc = service.clone();
        router.add_route(Route {
            method: HttpMethod::POST,
            path: "/tasks".to_string(),
            handler: Arc::new(move |req| {
                let svc = svc.clone();
                Box::pin(async move {
                    let input: CreateTask = req.json()?;
                    let task = svc.create(input.title, input.completed.unwrap_or(false));
                    HttpResponse::json(&task)
                })
            }),
            constraints: None,
        });

        // GET /compute/light
        let svc = service.clone();
        router.add_route(Route {
            method: HttpMethod::GET,
            path: "/compute/light".to_string(),
            handler: Arc::new(move |_req| {
                let svc = svc.clone();
                Box::pin(async move {
                    let result = svc.compute_hash(42);
                    HttpResponse::json(&serde_json::json!({ "result": result }))
                })
            }),
            constraints: None,
        });

        // GET /compute/heavy/:iterations
        let svc = service_clone;
        router.add_route(Route {
            method: HttpMethod::GET,
            path: "/compute/heavy/:iterations".to_string(),
            handler: Arc::new(move |req| {
                let svc = svc.clone();
                Box::pin(async move {
                    let iterations_str = req
                        .param("iterations")
                        .ok_or_else(|| Error::Validation("Missing iterations".to_string()))?;
                    let iterations: u32 = iterations_str
                        .parse()
                        .map_err(|_| Error::Validation("Invalid iterations".to_string()))?;

                    let result = svc.compute_heavy(iterations.min(100000)); // Cap at 100k
                    HttpResponse::json(&serde_json::json!({
                        "iterations": iterations,
                        "result": result
                    }))
                })
            }),
            constraints: None,
        });

        // Start listening
        let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port))
            .await
            .unwrap();
        println!("Listening on http://127.0.0.1:{}", port);

        loop {
            let (stream, _) = listener.accept().await.unwrap();
            let router = router.clone();

            tokio::spawn(async move {
                if let Err(e) = handle_connection(stream, &router).await {
                    eprintln!("Connection error: {}", e);
                }
            });
        }
    });
}

#[cfg(unix)]
async fn handle_connection(
    mut stream: tokio::net::TcpStream,
    router: &armature_core::Router,
) -> Result<(), Box<dyn std::error::Error>> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let mut buffer = [0; 8192];
    let n = stream.read(&mut buffer).await?;

    if n == 0 {
        return Ok(());
    }

    let request_str = String::from_utf8_lossy(&buffer[..n]);
    let lines: Vec<&str> = request_str.lines().collect();

    if lines.is_empty() {
        return Ok(());
    }

    let first_line: Vec<&str> = lines[0].split_whitespace().collect();
    if first_line.len() < 2 {
        return Ok(());
    }

    let method = first_line[0];
    let path = first_line[1];

    // Parse headers and body
    let mut headers = std::collections::HashMap::new();
    let mut body_start = 0;

    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.is_empty() {
            body_start = i + 1;
            break;
        }
        if let Some((key, value)) = line.split_once(':') {
            headers.insert(key.trim().to_string(), value.trim().to_string());
        }
    }

    let body = if body_start > 0 && body_start < lines.len() {
        lines[body_start..].join("\n").into_bytes()
    } else {
        Vec::new()
    };

    let request = armature_core::HttpRequest {
        method: method.to_string(),
        path: path.to_string(),
        headers,
        body,
        query_params: std::collections::HashMap::new(),
        path_params: std::collections::HashMap::new(),
    };

    let response = match router.route(request).await {
        Ok(resp) => resp,
        Err(e) => armature_core::HttpResponse::internal_server_error()
            .with_body(format!("Error: {}", e).into_bytes()),
    };

    let status_line = format!("HTTP/1.1 {} OK\r\n", response.status);
    let mut response_headers = String::new();
    for (key, value) in &response.headers {
        response_headers.push_str(&format!("{}: {}\r\n", key, value));
    }
    response_headers.push_str(&format!("Content-Length: {}\r\n", response.body.len()));
    response_headers.push_str("\r\n");

    stream.write_all(status_line.as_bytes()).await?;
    stream.write_all(response_headers.as_bytes()).await?;
    stream.write_all(&response.body).await?;
    stream.flush().await?;

    Ok(())
}
