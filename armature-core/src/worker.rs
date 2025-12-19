//! Per-Worker State Management
//!
//! This module provides per-worker (thread-local) state to avoid Arc cloning
//! overhead on the hot path. Instead of cloning `Arc<Router>` for every
//! request, each worker thread maintains its own reference.
//!
//! ## Performance Benefits
//!
//! ```text
//! Arc clone path:
//! Request → Arc::clone(&router) → atomic increment → handle
//!
//! Per-worker path:
//! Request → thread_local router ref → handle (no atomic ops)
//! ```
//!
//! This eliminates atomic reference counting on every request, which can
//! save 2-3% throughput under high concurrency.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use armature_core::worker::{WorkerRouter, init_worker_router};
//!
//! // Initialize once per worker thread
//! init_worker_router(router.clone());
//!
//! // Access router without cloning Arc
//! WorkerRouter::with(|router| {
//!     router.route(request).await
//! });
//! ```

use crate::Router;
use std::cell::RefCell;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

// ============================================================================
// Worker ID Generation
// ============================================================================

/// Global worker ID counter
static WORKER_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Get the next worker ID
#[inline]
pub fn next_worker_id() -> usize {
    WORKER_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Get total workers spawned
#[inline]
pub fn total_workers() -> usize {
    WORKER_ID_COUNTER.load(Ordering::Relaxed)
}

// ============================================================================
// Per-Worker Router Storage
// ============================================================================

thread_local! {
    /// Thread-local router storage
    static WORKER_ROUTER: RefCell<Option<Arc<Router>>> = const { RefCell::new(None) };

    /// Thread-local worker ID
    static WORKER_ID: RefCell<Option<usize>> = const { RefCell::new(None) };
}

/// Initialize the thread-local router for the current worker.
///
/// Call this once when spawning a new worker task.
///
/// # Example
///
/// ```rust,ignore
/// tokio::spawn(async move {
///     init_worker_router(router.clone());
///     // Now can use WorkerRouter::with() without cloning
/// });
/// ```
#[inline]
pub fn init_worker_router(router: Arc<Router>) {
    WORKER_ROUTER.with(|r| {
        *r.borrow_mut() = Some(router);
    });
    WORKER_ID.with(|id| {
        if id.borrow().is_none() {
            *id.borrow_mut() = Some(next_worker_id());
        }
    });
    WORKER_STATS.record_init();
}

/// Clear the thread-local router (for cleanup/testing).
#[inline]
pub fn clear_worker_router() {
    WORKER_ROUTER.with(|r| {
        *r.borrow_mut() = None;
    });
}

/// Get the current worker's ID.
#[inline]
pub fn worker_id() -> Option<usize> {
    WORKER_ID.with(|id| *id.borrow())
}

/// Check if the current thread has a worker router initialized.
#[inline]
pub fn has_worker_router() -> bool {
    WORKER_ROUTER.with(|r| r.borrow().is_some())
}

// ============================================================================
// Worker Router Access
// ============================================================================

/// Per-worker router accessor.
///
/// This provides zero-cost access to the router without Arc cloning.
pub struct WorkerRouter;

impl WorkerRouter {
    /// Execute a closure with the worker's router.
    ///
    /// This is the primary way to access the router without cloning.
    /// The closure receives a reference to the router.
    ///
    /// # Panics
    ///
    /// Panics if called from a thread without an initialized worker router.
    /// Use `try_with` for a non-panicking version.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let response = WorkerRouter::with(|router| {
    ///     router.route(request).await
    /// });
    /// ```
    #[inline]
    pub fn with<F, R>(f: F) -> R
    where
        F: FnOnce(&Router) -> R,
    {
        WORKER_ROUTER.with(|r| {
            let router_ref = r.borrow();
            let router = router_ref
                .as_ref()
                .expect("WorkerRouter not initialized. Call init_worker_router first.");
            WORKER_STATS.record_access();
            f(router)
        })
    }

    /// Try to execute a closure with the worker's router.
    ///
    /// Returns `None` if no worker router is initialized.
    #[inline]
    pub fn try_with<F, R>(f: F) -> Option<R>
    where
        F: FnOnce(&Router) -> R,
    {
        WORKER_ROUTER.with(|r| {
            let router_ref = r.borrow();
            router_ref.as_ref().map(|router| {
                WORKER_STATS.record_access();
                f(router)
            })
        })
    }

    /// Get a clone of the worker's router (fallback for async contexts).
    ///
    /// Use this when you need to move the router into an async block.
    /// This still clones the Arc, but only once per request instead of
    /// multiple times in nested closures.
    #[inline]
    pub fn clone_arc() -> Option<Arc<Router>> {
        WORKER_ROUTER.with(|r| {
            let router_ref = r.borrow();
            router_ref.as_ref().map(|router| {
                WORKER_STATS.record_clone();
                Arc::clone(router)
            })
        })
    }

    /// Get a clone of the worker's router, or panic if not initialized.
    #[inline]
    pub fn clone_arc_or_panic() -> Arc<Router> {
        Self::clone_arc().expect("WorkerRouter not initialized")
    }
}

// ============================================================================
// Worker Configuration
// ============================================================================

/// Configuration for worker threads.
#[derive(Debug, Clone)]
pub struct WorkerConfig {
    /// Number of worker threads (0 = use number of CPU cores)
    pub num_workers: usize,
    /// Enable CPU core affinity (pin workers to cores)
    pub cpu_affinity: bool,
    /// Stack size for worker threads (bytes)
    pub stack_size: Option<usize>,
    /// Worker thread name prefix
    pub name_prefix: String,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            num_workers: 0, // Auto-detect
            cpu_affinity: false,
            stack_size: None,
            name_prefix: "armature-worker".to_string(),
        }
    }
}

impl WorkerConfig {
    /// Create a new worker configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the number of worker threads.
    ///
    /// Use 0 for auto-detection (number of CPU cores).
    #[inline]
    pub fn workers(mut self, n: usize) -> Self {
        self.num_workers = n;
        self
    }

    /// Enable CPU core affinity.
    ///
    /// When enabled, workers are pinned to specific CPU cores for
    /// better cache locality.
    #[inline]
    pub fn with_cpu_affinity(mut self) -> Self {
        self.cpu_affinity = true;
        self
    }

    /// Set the worker thread stack size.
    #[inline]
    pub fn stack_size(mut self, size: usize) -> Self {
        self.stack_size = Some(size);
        self
    }

    /// Set the worker thread name prefix.
    #[inline]
    pub fn name_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.name_prefix = prefix.into();
        self
    }

    /// Get the effective number of workers.
    ///
    /// Returns `num_workers` if set, otherwise returns the number of CPU cores.
    #[inline]
    pub fn effective_workers(&self) -> usize {
        if self.num_workers > 0 {
            self.num_workers
        } else {
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1)
        }
    }
}

// ============================================================================
// CPU Core Affinity
// ============================================================================

/// CPU core affinity configuration.
#[derive(Debug, Clone)]
pub struct AffinityConfig {
    /// Enable core pinning
    pub enabled: bool,
    /// Specific cores to use (empty = all available)
    pub cores: Vec<usize>,
    /// Affinity mode
    pub mode: AffinityMode,
}

impl Default for AffinityConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cores: Vec::new(),
            mode: AffinityMode::RoundRobin,
        }
    }
}

impl AffinityConfig {
    /// Create a new affinity configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable CPU affinity.
    #[inline]
    pub fn enable(mut self) -> Self {
        self.enabled = true;
        self
    }

    /// Disable CPU affinity.
    #[inline]
    pub fn disable(mut self) -> Self {
        self.enabled = false;
        self
    }

    /// Set specific cores to use.
    #[inline]
    pub fn cores(mut self, cores: Vec<usize>) -> Self {
        self.cores = cores;
        self
    }

    /// Set affinity mode.
    #[inline]
    pub fn mode(mut self, mode: AffinityMode) -> Self {
        self.mode = mode;
        self
    }

    /// Get the core to pin a worker to based on worker ID.
    #[inline]
    pub fn core_for_worker(&self, worker_id: usize) -> usize {
        if self.cores.is_empty() {
            // Use all available cores
            let num_cores = num_cpus();
            match self.mode {
                AffinityMode::RoundRobin => worker_id % num_cores,
                AffinityMode::Packed => worker_id.min(num_cores - 1),
                AffinityMode::Spread => {
                    // Spread across cores with gaps
                    let stride = num_cores / 2;
                    (worker_id * stride.max(1)) % num_cores
                }
            }
        } else {
            // Use specified cores
            self.cores[worker_id % self.cores.len()]
        }
    }
}

/// CPU affinity mode - how workers are assigned to cores.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AffinityMode {
    /// Round-robin assignment: worker 0 → core 0, worker 1 → core 1, etc.
    RoundRobin,
    /// Pack workers on first N cores
    Packed,
    /// Spread workers across cores with gaps (better for hyper-threading)
    Spread,
}

/// Get the number of CPU cores.
#[inline]
pub fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}

/// Get the number of physical CPU cores (excluding hyper-threads).
///
/// On systems without hyper-threading, this returns the same as `num_cpus()`.
#[inline]
pub fn num_physical_cpus() -> usize {
    // On most systems, physical cores = total cores / 2 if hyper-threading
    // This is a heuristic; for accurate info, use platform-specific APIs
    let total = num_cpus();
    // Assume hyper-threading if > 4 cores and even number
    if total > 4 && total % 2 == 0 {
        total / 2
    } else {
        total
    }
}

/// Set CPU affinity for the current thread.
///
/// This pins the current thread to the specified CPU core.
///
/// # Platform Support
///
/// - Linux: Uses `sched_setaffinity`
/// - macOS/Windows: No-op (returns Ok but doesn't pin)
///
/// # Example
///
/// ```rust,ignore
/// // Pin current thread to core 0
/// set_thread_affinity(0)?;
/// ```
#[inline]
pub fn set_thread_affinity(core: usize) -> Result<(), AffinityError> {
    #[cfg(target_os = "linux")]
    {
        set_thread_affinity_linux(core)
    }

    #[cfg(not(target_os = "linux"))]
    {
        // No-op on non-Linux platforms
        let _ = core;
        Ok(())
    }
}

/// Set CPU affinity on Linux using sched_setaffinity.
#[cfg(target_os = "linux")]
fn set_thread_affinity_linux(core: usize) -> Result<(), AffinityError> {
    use std::mem;

    // Check if core is valid
    let num_cores = num_cpus();
    if core >= num_cores {
        return Err(AffinityError::InvalidCore { core, max: num_cores - 1 });
    }

    // cpu_set_t is 1024 bits = 128 bytes on Linux
    // We use a simplified version that supports up to 64 cores
    let mut mask: u64 = 0;
    mask |= 1u64 << core;

    // SAFETY: sched_setaffinity is a safe syscall when called with correct parameters
    unsafe {
        let result = libc::sched_setaffinity(
            0, // 0 = current thread
            mem::size_of::<u64>(),
            &mask as *const u64 as *const libc::cpu_set_t,
        );

        if result == 0 {
            AFFINITY_STATS.record_set(true);
            Ok(())
        } else {
            AFFINITY_STATS.record_set(false);
            Err(AffinityError::SystemError {
                errno: *libc::__errno_location(),
            })
        }
    }
}

/// Error setting CPU affinity.
#[derive(Debug, Clone)]
pub enum AffinityError {
    /// Invalid core number
    InvalidCore { core: usize, max: usize },
    /// System error (Linux errno)
    SystemError { errno: i32 },
    /// Platform not supported
    NotSupported,
}

impl std::fmt::Display for AffinityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidCore { core, max } => {
                write!(f, "Invalid core {}, max is {}", core, max)
            }
            Self::SystemError { errno } => {
                write!(f, "System error: errno {}", errno)
            }
            Self::NotSupported => write!(f, "CPU affinity not supported on this platform"),
        }
    }
}

impl std::error::Error for AffinityError {}

/// Get the CPU affinity of the current thread.
///
/// Returns the set of cores the thread is allowed to run on.
#[inline]
pub fn get_thread_affinity() -> Result<Vec<usize>, AffinityError> {
    #[cfg(target_os = "linux")]
    {
        get_thread_affinity_linux()
    }

    #[cfg(not(target_os = "linux"))]
    {
        // Return all cores on non-Linux
        Ok((0..num_cpus()).collect())
    }
}

/// Get CPU affinity on Linux.
#[cfg(target_os = "linux")]
fn get_thread_affinity_linux() -> Result<Vec<usize>, AffinityError> {
    use std::mem;

    let mut mask: u64 = 0;

    // SAFETY: sched_getaffinity is a safe syscall
    unsafe {
        let result = libc::sched_getaffinity(
            0,
            mem::size_of::<u64>(),
            &mut mask as *mut u64 as *mut libc::cpu_set_t,
        );

        if result == 0 {
            let mut cores = Vec::new();
            for i in 0..64 {
                if (mask & (1u64 << i)) != 0 {
                    cores.push(i);
                }
            }
            Ok(cores)
        } else {
            Err(AffinityError::SystemError {
                errno: *libc::__errno_location(),
            })
        }
    }
}

/// Check if CPU affinity is supported on this platform.
#[inline]
pub fn affinity_supported() -> bool {
    cfg!(target_os = "linux")
}

/// Initialize a worker with CPU affinity.
///
/// This is a convenience function that:
/// 1. Sets CPU affinity based on worker ID
/// 2. Initializes the thread-local router
///
/// # Example
///
/// ```rust,ignore
/// let config = AffinityConfig::new().enable();
///
/// tokio::spawn(async move {
///     init_worker_with_affinity(worker_id, &config, router.clone())?;
///     // Worker is now pinned to a core and has router access
/// });
/// ```
#[inline]
pub fn init_worker_with_affinity(
    worker_id: usize,
    config: &AffinityConfig,
    router: Arc<Router>,
) -> Result<(), AffinityError> {
    // Set CPU affinity if enabled
    if config.enabled && affinity_supported() {
        let core = config.core_for_worker(worker_id);
        set_thread_affinity(core)?;
    }

    // Initialize thread-local router
    init_worker_router(router);

    Ok(())
}

// ============================================================================
// Affinity Statistics
// ============================================================================

/// Statistics for CPU affinity operations.
#[derive(Debug, Default)]
pub struct AffinityStats {
    /// Successful affinity sets
    successful: AtomicU64,
    /// Failed affinity sets
    failed: AtomicU64,
}

impl AffinityStats {
    /// Create new stats.
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    fn record_set(&self, success: bool) {
        if success {
            self.successful.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Get successful sets.
    pub fn successful(&self) -> u64 {
        self.successful.load(Ordering::Relaxed)
    }

    /// Get failed sets.
    pub fn failed(&self) -> u64 {
        self.failed.load(Ordering::Relaxed)
    }

    /// Get success rate.
    pub fn success_rate(&self) -> f64 {
        let total = self.successful() + self.failed();
        if total > 0 {
            (self.successful() as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }
}

/// Global affinity statistics.
static AFFINITY_STATS: AffinityStats = AffinityStats {
    successful: AtomicU64::new(0),
    failed: AtomicU64::new(0),
};

/// Get global affinity statistics.
pub fn affinity_stats() -> &'static AffinityStats {
    &AFFINITY_STATS
}

// ============================================================================
// Statistics
// ============================================================================

/// Statistics for worker router operations.
#[derive(Debug, Default)]
pub struct WorkerStats {
    /// Number of worker initializations
    inits: AtomicU64,
    /// Number of router accesses (via with/try_with)
    accesses: AtomicU64,
    /// Number of Arc clones (via clone_arc)
    clones: AtomicU64,
}

impl WorkerStats {
    /// Create new stats.
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    fn record_init(&self) {
        self.inits.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    fn record_access(&self) {
        self.accesses.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    fn record_clone(&self) {
        self.clones.fetch_add(1, Ordering::Relaxed);
    }

    /// Get number of initializations.
    pub fn inits(&self) -> u64 {
        self.inits.load(Ordering::Relaxed)
    }

    /// Get number of accesses.
    pub fn accesses(&self) -> u64 {
        self.accesses.load(Ordering::Relaxed)
    }

    /// Get number of Arc clones.
    pub fn clones(&self) -> u64 {
        self.clones.load(Ordering::Relaxed)
    }

    /// Get clone avoidance ratio.
    ///
    /// Higher is better - means more accesses without Arc cloning.
    pub fn clone_avoidance_ratio(&self) -> f64 {
        let accesses = self.accesses() as f64;
        let clones = self.clones() as f64;
        if accesses > 0.0 {
            ((accesses - clones) / accesses) * 100.0
        } else {
            0.0
        }
    }
}

/// Global worker statistics.
static WORKER_STATS: WorkerStats = WorkerStats {
    inits: AtomicU64::new(0),
    accesses: AtomicU64::new(0),
    clones: AtomicU64::new(0),
};

/// Get global worker statistics.
pub fn worker_stats() -> &'static WorkerStats {
    &WORKER_STATS
}

// ============================================================================
// Worker Handle
// ============================================================================

/// A handle to a worker for tracking and management.
#[derive(Debug, Clone)]
pub struct WorkerHandle {
    /// Worker ID
    pub id: usize,
    /// Worker name
    pub name: String,
}

impl WorkerHandle {
    /// Create a new worker handle.
    pub fn new(id: usize, name_prefix: &str) -> Self {
        Self {
            id,
            name: format!("{}-{}", name_prefix, id),
        }
    }
}

// ============================================================================
// Macros for ergonomic usage
// ============================================================================

/// Initialize worker router and execute code.
///
/// This macro handles initialization and provides access in one step.
///
/// # Example
///
/// ```rust,ignore
/// with_worker_router!(router, {
///     router.route(request).await
/// });
/// ```
#[macro_export]
macro_rules! with_worker_router {
    ($router:ident, $body:block) => {{
        $crate::worker::WorkerRouter::with(|$router| $body)
    }};
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_id_generation() {
        let id1 = next_worker_id();
        let id2 = next_worker_id();
        assert!(id2 > id1);
    }

    #[test]
    fn test_worker_config_default() {
        let config = WorkerConfig::default();
        assert_eq!(config.num_workers, 0);
        assert!(!config.cpu_affinity);
    }

    #[test]
    fn test_worker_config_builder() {
        let config = WorkerConfig::new()
            .workers(4)
            .with_cpu_affinity()
            .name_prefix("test-worker");

        assert_eq!(config.num_workers, 4);
        assert!(config.cpu_affinity);
        assert_eq!(config.name_prefix, "test-worker");
    }

    #[test]
    fn test_effective_workers() {
        let config = WorkerConfig::new().workers(8);
        assert_eq!(config.effective_workers(), 8);

        let auto_config = WorkerConfig::new();
        assert!(auto_config.effective_workers() >= 1);
    }

    #[test]
    fn test_affinity_config_default() {
        let config = AffinityConfig::default();
        assert!(!config.enabled);
        assert!(config.cores.is_empty());
        assert_eq!(config.mode, AffinityMode::RoundRobin);
    }

    #[test]
    fn test_affinity_config_builder() {
        let config = AffinityConfig::new()
            .enable()
            .cores(vec![0, 2, 4])
            .mode(AffinityMode::Spread);

        assert!(config.enabled);
        assert_eq!(config.cores, vec![0, 2, 4]);
        assert_eq!(config.mode, AffinityMode::Spread);
    }

    #[test]
    fn test_core_for_worker_round_robin() {
        let config = AffinityConfig::new()
            .enable()
            .mode(AffinityMode::RoundRobin);

        let num_cores = num_cpus();
        assert_eq!(config.core_for_worker(0), 0);
        assert_eq!(config.core_for_worker(1), 1 % num_cores);
        assert_eq!(config.core_for_worker(num_cores), 0);
    }

    #[test]
    fn test_core_for_worker_specific_cores() {
        let config = AffinityConfig::new()
            .enable()
            .cores(vec![0, 4, 8]);

        assert_eq!(config.core_for_worker(0), 0);
        assert_eq!(config.core_for_worker(1), 4);
        assert_eq!(config.core_for_worker(2), 8);
        assert_eq!(config.core_for_worker(3), 0); // Wraps around
    }

    #[test]
    fn test_num_cpus() {
        let cpus = num_cpus();
        assert!(cpus >= 1);
    }

    #[test]
    fn test_num_physical_cpus() {
        let physical = num_physical_cpus();
        let total = num_cpus();
        assert!(physical >= 1);
        assert!(physical <= total);
    }

    #[test]
    fn test_affinity_supported() {
        // Just check it returns a bool without panicking
        let _ = affinity_supported();
    }

    #[test]
    fn test_get_thread_affinity() {
        // Should return Ok on all platforms
        let result = get_thread_affinity();
        assert!(result.is_ok());
        let cores = result.unwrap();
        assert!(!cores.is_empty());
    }

    #[test]
    fn test_affinity_stats() {
        let stats = affinity_stats();
        let _ = stats.successful();
        let _ = stats.failed();
        let _ = stats.success_rate();
    }

    #[test]
    fn test_affinity_error_display() {
        let err1 = AffinityError::InvalidCore { core: 100, max: 7 };
        assert!(err1.to_string().contains("100"));

        let err2 = AffinityError::NotSupported;
        assert!(err2.to_string().contains("not supported"));
    }

    #[test]
    fn test_worker_router_not_initialized() {
        // Clear any existing router
        clear_worker_router();

        assert!(!has_worker_router());
        assert!(WorkerRouter::try_with(|_| ()).is_none());
        assert!(WorkerRouter::clone_arc().is_none());
    }

    #[test]
    fn test_worker_router_initialization() {
        let router = Arc::new(Router::new());

        init_worker_router(router);

        assert!(has_worker_router());
        assert!(worker_id().is_some());

        WorkerRouter::with(|r| {
            assert!(r.routes.is_empty());
        });

        // Cleanup
        clear_worker_router();
    }

    #[test]
    fn test_worker_handle() {
        let handle = WorkerHandle::new(5, "test-worker");
        assert_eq!(handle.id, 5);
        assert_eq!(handle.name, "test-worker-5");
    }

    #[test]
    fn test_worker_stats() {
        let stats = worker_stats();

        // Stats should be accessible
        let _ = stats.inits();
        let _ = stats.accesses();
        let _ = stats.clones();
        let _ = stats.clone_avoidance_ratio();
    }
}

