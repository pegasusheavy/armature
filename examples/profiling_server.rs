//! Profiling Server Example
//!
//! This example runs an Armature server with CPU profiling enabled.
//! After running, it generates a flamegraph showing where CPU time is spent.
//!
//! **Note:** This example only works on Unix platforms (Linux, macOS).
//! On Windows, it will print an error message and exit.
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

// Non-Unix platforms: print error and exit
#[cfg(not(unix))]
fn main() {
    eprintln!("Error: CPU profiling is only supported on Unix platforms (Linux, macOS).");
    eprintln!("The pprof crate requires Unix-specific APIs for CPU sampling.");
    eprintln!();
    eprintln!("To profile on Windows, consider using:");
    eprintln!("  - Windows Performance Analyzer (WPA)");
    eprintln!("  - Visual Studio Profiler");
    eprintln!("  - Intel VTune");
    std::process::exit(1);
}

// Unix platforms: full profiling support
#[cfg(unix)]
fn main() {
    unix_impl::run();
}

#[cfg(unix)]
mod unix_impl {
    #![allow(dead_code)]

    use std::net::TcpListener;

    /// Finds an available port on localhost.
    fn find_available_port() -> u16 {
        TcpListener::bind("127.0.0.1:0")
            .expect("Failed to bind to random port")
            .local_addr()
            .expect("Failed to get local address")
            .port()
    }

    use armature::prelude::*;
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

    #[injectable]
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
    // Controller
    // ============================================================================

    #[controller("/tasks")]
    #[derive(Default, Clone)]
    struct TaskController;

    #[routes]
    impl TaskController {
        #[get("")]
        async fn list() -> Result<HttpResponse, Error> {
            let service = TaskService::new();
            HttpResponse::json(&service.get_all())
        }

        #[get("/:id")]
        async fn get(req: HttpRequest) -> Result<HttpResponse, Error> {
            let id_str = req
                .param("id")
                .ok_or_else(|| Error::Validation("Missing id".to_string()))?;
            let id: u32 = id_str
                .parse()
                .map_err(|_| Error::Validation("Invalid id".to_string()))?;

            let service = TaskService::new();
            let task = service
                .get_by_id(id)
                .ok_or_else(|| Error::RouteNotFound("Task not found".to_string()))?;

            HttpResponse::json(&task)
        }

        #[post("")]
        async fn create(req: HttpRequest) -> Result<HttpResponse, Error> {
            let input: CreateTask = req.json()?;
            let service = TaskService::new();
            let task = service.create(input.title, input.completed.unwrap_or(false));
            HttpResponse::json(&task)
        }

        #[put("/:id")]
        async fn update(req: HttpRequest) -> Result<HttpResponse, Error> {
            let id_str = req
                .param("id")
                .ok_or_else(|| Error::Validation("Missing id".to_string()))?;
            let id: u32 = id_str
                .parse()
                .map_err(|_| Error::Validation("Invalid id".to_string()))?;

            let input: CreateTask = req.json()?;
            let service = TaskService::new();
            let task = service
                .update(id, input.title, input.completed.unwrap_or(false))
                .ok_or_else(|| Error::RouteNotFound("Task not found".to_string()))?;

            HttpResponse::json(&task)
        }

        #[delete("/:id")]
        async fn delete(req: HttpRequest) -> Result<HttpResponse, Error> {
            let id_str = req
                .param("id")
                .ok_or_else(|| Error::Validation("Missing id".to_string()))?;
            let id: u32 = id_str
                .parse()
                .map_err(|_| Error::Validation("Invalid id".to_string()))?;

            let service = TaskService::new();
            if service.delete(id) {
                Ok(HttpResponse::no_content())
            } else {
                Err(Error::RouteNotFound("Task not found".to_string()))
            }
        }
    }

    // CPU-intensive endpoint for stress testing
    #[controller("/compute")]
    #[derive(Default, Clone)]
    struct ComputeController;

    #[routes]
    impl ComputeController {
        #[get("/light")]
        async fn light() -> Result<HttpResponse, Error> {
            let service = TaskService::new();
            let result = service.compute_hash(42);
            HttpResponse::json(&serde_json::json!({ "result": result }))
        }

        #[get("/heavy/:iterations")]
        async fn heavy(req: HttpRequest) -> Result<HttpResponse, Error> {
            let iterations_str = req
                .param("iterations")
                .ok_or_else(|| Error::Validation("Missing iterations".to_string()))?;
            let iterations: u32 = iterations_str
                .parse()
                .map_err(|_| Error::Validation("Invalid iterations".to_string()))?;

            let service = TaskService::new();
            let result = service.compute_heavy(iterations.min(100000)); // Cap at 100k
            HttpResponse::json(&serde_json::json!({
                "iterations": iterations,
                "result": result
            }))
        }

        #[get("/health")]
        async fn health() -> Result<HttpResponse, Error> {
            HttpResponse::json(&serde_json::json!({
                "status": "healthy"
            }))
        }
    }

    // ============================================================================
    // Module
    // ============================================================================

    #[module(
        providers: [TaskService],
        controllers: [TaskController, ComputeController]
    )]
    #[derive(Default)]
    struct AppModule;

    // ============================================================================
    // Main
    // ============================================================================

    #[tokio::main]
    pub async fn run() {
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

        // Start the server
        let app = Application::create::<AppModule>().await;
        app.listen(port).await.unwrap();
    }
}
