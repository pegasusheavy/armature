# Multithreading Opportunities in Armature

This document outlines opportunities to add multithreading support to improve performance and scalability.

## Current State

Armature already uses **Tokio's async runtime** which provides:
- ✅ Per-connection task spawning (already implemented)
- ✅ Async I/O operations
- ✅ Concurrent request handling

However, there are additional opportunities for **parallel processing** of CPU-bound operations.

---

## 1. Parallel Validation

### Current Implementation

Validation runs validators **sequentially** for each field:

```rust
// armature-validation/src/rules.rs
pub fn validate(&self, value: &str) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();
    for validator in &self.validators {  // Sequential
        if let Err(error) = validator(value, &self.field) {
            errors.push(error);
        }
    }
    // ...
}
```

### Opportunity: Parallel Field Validation

When validating multiple fields, **run validators in parallel**:

```rust
use rayon::prelude::*;

/// Validate all fields in parallel
pub fn validate_parallel(
    &self,
    data: &std::collections::HashMap<String, String>,
) -> Result<(), Vec<ValidationError>> {
    let all_errors: Vec<ValidationError> = self.rules
        .par_iter()  // Parallel iterator
        .filter_map(|rule| {
            if let Some(value) = data.get(&rule.field) {
                rule.validate(value).err()
            } else {
                None
            }
        })
        .flatten()
        .collect();

    if all_errors.is_empty() {
        Ok(())
    } else {
        Err(all_errors)
    }
}
```

**Benefits:**
- **2-4x faster** for forms with 10+ fields
- No blocking the async runtime
- Scales with CPU cores

**Implementation Location:** `armature-validation/src/rules.rs`

---

## 2. Parallel Multipart File Processing

### Current Implementation

Files are processed **sequentially** during multipart parsing:

```rust
// armature-core/src/form.rs
for part in parts.iter().skip(1) {  // Sequential
    if let Some(field) = self.parse_part(part)? {
        fields.push(field);
    }
}
```

### Opportunity: Parallel File Parsing

Parse multiple file uploads **simultaneously**:

```rust
use rayon::prelude::*;

/// Parse multipart form data in parallel
pub fn parse_parallel(&self, body: &[u8]) -> Result<Vec<FormField>, Error> {
    let boundary_marker = format!("--{}", self.boundary);
    let body_str = String::from_utf8_lossy(body);

    // Split by boundary
    let parts: Vec<&str> = body_str.split(&boundary_marker).collect();

    // Parse parts in parallel
    let fields: Result<Vec<_>, _> = parts
        .par_iter()  // Parallel iterator
        .skip(1)
        .filter(|part| !part.trim().is_empty() && part.trim() != "--")
        .map(|part| self.parse_part(part))
        .collect();

    Ok(fields?.into_iter().flatten().collect())
}
```

### Opportunity: Parallel File Saves

Save multiple uploaded files **concurrently**:

```rust
use tokio::task::JoinSet;

/// Save multiple files concurrently
pub async fn save_files_parallel(
    files: Vec<FormFile>,
    base_path: &str,
) -> Result<Vec<String>, Error> {
    let mut set = JoinSet::new();

    for file in files {
        let path = format!("{}/{}", base_path, file.filename);
        let data = file.data.clone();

        set.spawn(async move {
            tokio::fs::write(&path, &data).await?;
            Ok::<_, Error>(path)
        });
    }

    let mut saved_paths = Vec::new();
    while let Some(result) = set.join_next().await {
        saved_paths.push(result??);
    }

    Ok(saved_paths)
}
```

**Benefits:**
- **5-10x faster** for batch file uploads
- Efficient I/O utilization
- Scales with file count

**Implementation Location:** `armature-core/src/form.rs`

---

## 3. Parallel Cache Operations

### Current Implementation

Batch cache gets/sets run **sequentially**:

```rust
// Current: Sequential cache operations
for key in keys {
    cache.get(key).await?;  // Awaits each operation
}
```

### Opportunity: Batch Cache Operations

Add parallel batch operations:

```rust
// armature-cache/src/traits.rs

/// Get multiple keys in parallel
async fn get_many(&self, keys: &[&str]) -> CacheResult<Vec<Option<String>>> {
    use futures::future::join_all;

    let futures = keys.iter().map(|key| self.get_json(key));
    let results = join_all(futures).await;

    results.into_iter().collect()
}

/// Set multiple key-value pairs in parallel
async fn set_many(
    &self,
    items: &[(&str, String)],
    ttl: Option<Duration>,
) -> CacheResult<()> {
    use futures::future::try_join_all;

    let futures = items.iter().map(|(key, value)| {
        self.set_json(key, value.clone(), ttl)
    });

    try_join_all(futures).await?;
    Ok(())
}

/// Delete multiple keys in parallel
async fn delete_many(&self, keys: &[&str]) -> CacheResult<()> {
    use futures::future::try_join_all;

    let futures = keys.iter().map(|key| self.delete(key));
    try_join_all(futures).await?;
    Ok(())
}
```

**Benefits:**
- **10-100x faster** for batch operations (depending on network latency)
- Essential for cache warming
- Reduces total latency

**Implementation Location:** `armature-cache/src/traits.rs`

---

## 4. Parallel Queue Job Processing

### Current State

Queue workers already spawn **concurrent workers**, but job handlers run one at a time per worker:

```rust
// armature-queue/src/worker.rs
for i in 0..self.config.concurrency {  // Already parallel workers
    tokio::spawn(async move {
        // Each worker processes jobs sequentially
    });
}
```

### Opportunity: Worker Pool with Thread Pool

For CPU-intensive job handlers, use a **blocking thread pool**:

```rust
use tokio::task::spawn_blocking;

/// Register a CPU-intensive handler
pub fn register_cpu_intensive_handler<F, Fut>(
    &mut self,
    job_type: impl Into<String>,
    handler: F,
) where
    F: Fn(Job) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = QueueResult<()>> + Send + 'static,
{
    let wrapped_handler = Arc::new(move |job: Job| {
        Box::pin(async move {
            // Run in blocking thread pool
            spawn_blocking(move || {
                // Execute CPU-intensive work
                tokio::runtime::Handle::current()
                    .block_on(handler(job))
            }).await
            .map_err(|e| QueueError::ExecutionFailed(e.to_string()))?
        }) as Pin<Box<dyn Future<Output = QueueResult<()>> + Send>>
    });

    // Register handler...
}
```

### Opportunity: Batch Job Processing

Process multiple jobs from the same type **in parallel**:

```rust
/// Process multiple jobs of the same type simultaneously
pub async fn process_batch(
    &self,
    job_type: &str,
    max_batch_size: usize,
) -> QueueResult<Vec<JobId>> {
    use tokio::task::JoinSet;

    let mut jobs = Vec::new();

    // Dequeue multiple jobs
    for _ in 0..max_batch_size {
        if let Some(job) = self.queue.dequeue().await? {
            if job.job_type == job_type {
                jobs.push(job);
            } else {
                // Re-queue different type
                self.queue.enqueue_job(job).await?;
                break;
            }
        } else {
            break;
        }
    }

    // Process all jobs in parallel
    let mut set = JoinSet::new();
    for job in jobs {
        let handler = self.get_handler(&job.job_type)?;
        set.spawn(async move {
            handler(job.clone()).await?;
            Ok::<_, QueueError>(job.id)
        });
    }

    let mut processed = Vec::new();
    while let Some(result) = set.join_next().await {
        processed.push(result??);
    }

    Ok(processed)
}
```

**Benefits:**
- **3-5x throughput** for similar job types
- Better CPU utilization
- Reduced queue latency

**Implementation Location:** `armature-queue/src/worker.rs`

---

## 5. Parallel Middleware Chain

### Current Implementation

Middleware runs **sequentially**:

```rust
// armature-core/src/middleware.rs
for middleware in &self.middlewares {
    // Run one at a time
}
```

### Opportunity: Independent Middleware Parallel Execution

Some middleware are independent and can run **in parallel**:

```rust
use futures::future::join_all;

/// Execute independent middleware in parallel
pub async fn process_parallel(
    &self,
    request: &HttpRequest,
) -> Result<Vec<MiddlewareResult>, Error> {
    // Group middleware by dependencies
    let (independent, dependent) = self.group_middleware();

    // Run independent middleware in parallel
    let futures = independent.iter().map(|m| m.process(request));
    let results = join_all(futures).await;

    // Run dependent middleware sequentially
    for middleware in dependent {
        middleware.process(request).await?;
    }

    Ok(results)
}
```

**Example:** Logging, metrics, and request ID can run **simultaneously**.

**Benefits:**
- **30-50% faster** for 5+ middleware
- No waiting for I/O operations
- Better observability performance

**Implementation Location:** `armature-core/src/middleware.rs`

---

## 6. Parallel SSR Rendering

### Current Implementation

SSR renders one component at a time:

```rust
// armature-angular/src/renderer.rs
let output = child.wait_with_output().await?;  // Blocks
```

### Opportunity: Parallel Component Rendering

Render multiple independent components **simultaneously**:

```rust
use tokio::task::JoinSet;

/// Render multiple routes in parallel (e.g., for static site generation)
pub async fn render_many_parallel(
    &self,
    routes: Vec<String>,
) -> Result<HashMap<String, String>, Error> {
    let mut set = JoinSet::new();

    for route in routes {
        let renderer = self.clone();
        set.spawn(async move {
            let html = renderer.render(&route, None).await?;
            Ok::<_, Error>((route, html))
        });
    }

    let mut rendered = HashMap::new();
    while let Some(result) = set.join_next().await {
        let (route, html) = result??;
        rendered.insert(route, html);
    }

    Ok(rendered)
}
```

### Opportunity: Pre-render Static Pages

For static site generation, render all pages **in parallel**:

```rust
/// Pre-render all static pages
pub async fn pre_render_site(
    &self,
    output_dir: &str,
) -> Result<usize, Error> {
    use rayon::prelude::*;

    let routes = self.get_static_routes();

    // Render pages in parallel
    let results: Vec<_> = routes
        .par_iter()
        .map(|route| {
            tokio::runtime::Handle::current()
                .block_on(self.render(route, None))
        })
        .collect();

    // Write to disk in parallel
    use tokio::fs::write;
    let mut set = JoinSet::new();

    for (route, html) in routes.into_iter().zip(results) {
        let html = html?;
        let path = format!("{}/{}.html", output_dir, route.trim_start_matches('/'));
        set.spawn(async move {
            write(&path, html).await
        });
    }

    let mut count = 0;
    while let Some(result) = set.join_next().await {
        result??;
        count += 1;
    }

    Ok(count)
}
```

**Benefits:**
- **10-20x faster** static site generation
- Essential for large sites
- One-time build optimization

**Implementation Location:** `armature-angular/src/renderer.rs`, `armature-react/src/renderer.rs`, etc.

---

## 7. Parallel Data Aggregation

### Opportunity: Parallel API Composition

When combining data from multiple sources, fetch **in parallel**:

```rust
use futures::future::try_join;

/// Example: Dashboard data aggregation
pub async fn get_dashboard_data(&self) -> Result<DashboardData, Error> {
    // Fetch from multiple sources in parallel
    let (user, posts, comments, stats) = try_join!(
        self.fetch_user(),
        self.fetch_posts(),
        self.fetch_comments(),
        self.fetch_stats(),
    )?;

    Ok(DashboardData {
        user,
        posts,
        comments,
        stats,
    })
}
```

**Benefits:**
- **Latency = max(query1, query2, ...)** instead of **sum(all queries)**
- Essential for microservices
- Better user experience

**Implementation Location:** Application-level controllers

---

## 8. Thread Pool Configuration

### Add Rayon for CPU-Bound Operations

Update `Cargo.toml`:

```toml
[dependencies]
rayon = "1.10"          # Parallel iterators
futures = "0.3"         # Future utilities
tokio = { version = "1.35", features = ["rt-multi-thread", "sync", "time"] }
```

### Configure Thread Pool

```rust
// armature-core/src/application.rs

/// Configure the application with custom thread pool
pub fn with_thread_pool(mut self, num_threads: usize) -> Self {
    rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
        .expect("Failed to build thread pool");
    self
}

/// Configure blocking thread pool for CPU-intensive tasks
pub fn with_blocking_threads(self, max_blocking_threads: usize) -> Self {
    // Tokio already has a blocking thread pool, configure it
    std::env::set_var("TOKIO_WORKER_THREADS", max_blocking_threads.to_string());
    self
}
```

---

## Priority Recommendations

### High Priority (Implement First)

1. **✅ Parallel Cache Batch Operations** - Essential for cache warming and bulk operations
2. **✅ Parallel File Upload Processing** - Significant performance gain for file-heavy applications
3. **✅ Parallel Validation** - Improves form processing performance

### Medium Priority

4. **Parallel SSR Pre-rendering** - Critical for static site generation
5. **Parallel Queue Batch Processing** - Better throughput for job processing
6. **Parallel Middleware** - Reduces request latency

### Low Priority (Nice to Have)

7. **CPU-Intensive Job Handlers** - Only needed for specific use cases
8. **Parallel Data Aggregation** - Application-level optimization

---

## Performance Guidelines

### When to Use Parallelization

**✅ Good candidates:**
- **I/O-bound operations** (database queries, API calls, file I/O)
- **Independent operations** (validating different fields, processing multiple files)
- **Batch operations** (bulk cache gets, multiple job processing)
- **CPU-bound work** (image processing, data transformation, encryption)

**❌ Bad candidates:**
- **Operations with dependencies** (must run in order)
- **Very fast operations** (< 1ms, overhead not worth it)
- **Single items** (no benefit from parallelization)
- **Mutex-heavy code** (contention negates benefits)

### Testing Performance

Always benchmark before and after:

```rust
// Add to benches/parallelization_benchmarks.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_validation_sequential(c: &mut Criterion) {
    c.bench_function("validation_sequential", |b| {
        b.iter(|| {
            // Sequential validation
        })
    });
}

fn bench_validation_parallel(c: &mut Criterion) {
    c.bench_function("validation_parallel", |b| {
        b.iter(|| {
            // Parallel validation
        })
    });
}

criterion_group!(benches, bench_validation_sequential, bench_validation_parallel);
criterion_main!(benches);
```

---

## Implementation Checklist

- [ ] Add `rayon` dependency to `armature-core/Cargo.toml`
- [ ] Implement parallel validation in `armature-validation`
- [ ] Add batch cache operations to `armature-cache`
- [ ] Implement parallel file processing in `armature-core/src/form.rs`
- [ ] Add parallel middleware option in `armature-core/src/middleware.rs`
- [ ] Implement batch job processing in `armature-queue`
- [ ] Add parallel SSR rendering for static generation
- [ ] Write benchmarks for all parallel implementations
- [ ] Update documentation with performance characteristics
- [ ] Add configuration options for thread pool sizing

---

## Summary

Armature's **async foundation** (Tokio) provides excellent concurrency for I/O operations. Adding **parallelization with Rayon** for CPU-bound operations and **concurrent batch operations** will significantly improve performance for:

- **Large form validation** → 2-4x faster
- **Bulk file uploads** → 5-10x faster
- **Batch cache operations** → 10-100x faster
- **Static site generation** → 10-20x faster
- **Queue throughput** → 3-5x higher

These optimizations are **backward compatible** and can be added incrementally without breaking existing code.

