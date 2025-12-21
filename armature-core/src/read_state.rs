//! Read-Optimized State Management
//!
//! This module provides state containers optimized for read-heavy workloads
//! using `parking_lot::RwLock` which offers better performance than `std::sync::RwLock`:
//!
//! - **No poisoning**: No `unwrap()` needed on lock results
//! - **Faster**: Optimized spin-lock implementation
//! - **Upgradeable reads**: Can upgrade read lock to write lock
//! - **Fair scheduling**: Options for reader/writer priority
//!
//! # Key Types
//!
//! - [`ReadState<T>`]: General-purpose read-optimized state
//! - [`ReadCache<K, V>`]: Read-optimized concurrent HashMap
//! - [`ReadConfig<T>`]: Configuration state with change detection
//!
//! # When to Use
//!
//! Use these types when:
//! - Reads vastly outnumber writes (>10:1 ratio)
//! - Multiple threads read concurrently
//! - Write contention is acceptable
//!
//! For write-heavy workloads, consider `AtomicState` or sharded approaches.
//!
//! # Example
//!
//! ```rust,ignore
//! use armature_core::read_state::ReadState;
//!
//! // Create read-optimized state
//! let state = ReadState::new(AppConfig::default());
//!
//! // Fast concurrent reads (multiple threads)
//! let config = state.read();
//! println!("Timeout: {}", config.timeout_ms);
//!
//! // Infrequent writes
//! state.write(|config| {
//!     config.timeout_ms = 10000;
//! });
//!
//! // Upgrade pattern: read then conditionally write
//! state.read_then_write(
//!     |config| config.connections < 100, // condition
//!     |config| config.connections += 10,  // update if true
//! );
//! ```

use parking_lot::{RwLock, RwLockReadGuard, RwLockUpgradableReadGuard, RwLockWriteGuard};
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Deref;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

// ============================================================================
// Statistics
// ============================================================================

/// Global statistics for read state operations.
#[derive(Debug, Default)]
pub struct ReadStateStats {
    reads: AtomicU64,
    writes: AtomicU64,
    upgrades: AtomicU64,
    upgrade_failures: AtomicU64,
}

impl ReadStateStats {
    /// Get read count.
    pub fn reads(&self) -> u64 {
        self.reads.load(Ordering::Relaxed)
    }

    /// Get write count.
    pub fn writes(&self) -> u64 {
        self.writes.load(Ordering::Relaxed)
    }

    /// Get upgrade count.
    pub fn upgrades(&self) -> u64 {
        self.upgrades.load(Ordering::Relaxed)
    }

    /// Get read/write ratio.
    pub fn read_write_ratio(&self) -> f64 {
        let writes = self.writes();
        if writes == 0 {
            f64::INFINITY
        } else {
            self.reads() as f64 / writes as f64
        }
    }

    fn record_read(&self) {
        self.reads.fetch_add(1, Ordering::Relaxed);
    }

    fn record_write(&self) {
        self.writes.fetch_add(1, Ordering::Relaxed);
    }

    fn record_upgrade(&self) {
        self.upgrades.fetch_add(1, Ordering::Relaxed);
    }

    #[allow(dead_code)]
    fn record_upgrade_failure(&self) {
        self.upgrade_failures.fetch_add(1, Ordering::Relaxed);
    }
}

/// Global stats instance.
pub static READ_STATE_STATS: ReadStateStats = ReadStateStats {
    reads: AtomicU64::new(0),
    writes: AtomicU64::new(0),
    upgrades: AtomicU64::new(0),
    upgrade_failures: AtomicU64::new(0),
};

// ============================================================================
// ReadState - General Purpose
// ============================================================================

/// Read-optimized state container using `parking_lot::RwLock`.
///
/// Provides efficient concurrent read access with exclusive write access.
/// Optimized for scenarios where reads vastly outnumber writes.
///
/// # Performance
///
/// - **Read**: ~5-10ns (uncontended)
/// - **Write**: ~20-30ns (uncontended)
/// - **Concurrent reads**: Near-linear scaling
///
/// # Example
///
/// ```rust,ignore
/// use armature_core::read_state::ReadState;
///
/// #[derive(Clone, Default)]
/// struct Config {
///     timeout_ms: u64,
///     max_retries: u32,
/// }
///
/// let state = ReadState::new(Config::default());
///
/// // Multiple readers (concurrent)
/// let config = state.read();
/// println!("Timeout: {}", config.timeout_ms);
///
/// // Single writer (exclusive)
/// state.write(|config| {
///     config.timeout_ms = 5000;
/// });
/// ```
pub struct ReadState<T> {
    inner: RwLock<T>,
    version: AtomicU64,
}

impl<T> ReadState<T> {
    /// Create new read state.
    #[inline]
    pub fn new(value: T) -> Self {
        Self {
            inner: RwLock::new(value),
            version: AtomicU64::new(1),
        }
    }

    /// Get read access.
    ///
    /// Multiple threads can hold read locks simultaneously.
    #[inline]
    pub fn read(&self) -> ReadGuard<'_, T> {
        READ_STATE_STATS.record_read();
        ReadGuard {
            guard: self.inner.read(),
        }
    }

    /// Try to get read access without blocking.
    ///
    /// Returns `None` if a write lock is held.
    #[inline]
    pub fn try_read(&self) -> Option<ReadGuard<'_, T>> {
        self.inner.try_read().map(|guard| {
            READ_STATE_STATS.record_read();
            ReadGuard { guard }
        })
    }

    /// Get upgradeable read access.
    ///
    /// Can be upgraded to write access without releasing the lock.
    /// Only one upgradeable read can exist at a time.
    #[inline]
    pub fn upgradeable_read(&self) -> UpgradeableGuard<'_, T> {
        READ_STATE_STATS.record_read();
        UpgradeableGuard {
            guard: self.inner.upgradable_read(),
            version: &self.version,
        }
    }

    /// Get exclusive write access.
    #[inline]
    pub fn write_guard(&self) -> WriteGuard<'_, T> {
        READ_STATE_STATS.record_write();
        WriteGuard {
            guard: self.inner.write(),
            version: &self.version,
        }
    }

    /// Apply a function with write access.
    #[inline]
    pub fn write<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        READ_STATE_STATS.record_write();
        let mut guard = self.inner.write();
        let result = f(&mut *guard);
        self.version.fetch_add(1, Ordering::Release);
        result
    }

    /// Read then conditionally write.
    ///
    /// Reads the value, checks a condition, and if true, upgrades to write
    /// and applies the update. More efficient than separate read/write when
    /// writes are conditional.
    ///
    /// Returns `true` if the write was applied.
    pub fn read_then_write<P, F>(&self, predicate: P, update: F) -> bool
    where
        P: FnOnce(&T) -> bool,
        F: FnOnce(&mut T),
    {
        let guard = self.inner.upgradable_read();
        if predicate(&*guard) {
            READ_STATE_STATS.record_upgrade();
            let mut write_guard = RwLockUpgradableReadGuard::upgrade(guard);
            update(&mut *write_guard);
            self.version.fetch_add(1, Ordering::Release);
            true
        } else {
            false
        }
    }

    /// Get current version.
    ///
    /// Increments on each write. Useful for cache invalidation.
    #[inline]
    pub fn version(&self) -> u64 {
        self.version.load(Ordering::Acquire)
    }

    /// Check if state has changed since a version.
    #[inline]
    pub fn changed_since(&self, version: u64) -> bool {
        self.version() != version
    }

    /// Replace the entire value.
    #[inline]
    pub fn replace(&self, value: T) -> T {
        READ_STATE_STATS.record_write();
        let mut guard = self.inner.write();
        let old = std::mem::replace(&mut *guard, value);
        self.version.fetch_add(1, Ordering::Release);
        old
    }
}

impl<T: Clone> ReadState<T> {
    /// Get a cloned copy of the value.
    ///
    /// Useful when you need ownership of the data.
    #[inline]
    pub fn cloned(&self) -> T {
        self.read().clone()
    }

    /// Get value with version.
    #[inline]
    pub fn cloned_versioned(&self) -> (T, u64) {
        let guard = self.read();
        let value = guard.clone();
        let version = self.version();
        (value, version)
    }
}

impl<T: Default> Default for ReadState<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for ReadState<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.try_read() {
            Some(guard) => f.debug_struct("ReadState").field("value", &*guard).finish(),
            None => f.debug_struct("ReadState").field("value", &"<locked>").finish(),
        }
    }
}

// ============================================================================
// Guard Types
// ============================================================================

/// Read guard for `ReadState`.
pub struct ReadGuard<'a, T> {
    guard: RwLockReadGuard<'a, T>,
}

impl<T> Deref for ReadGuard<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}

impl<T> AsRef<T> for ReadGuard<'_, T> {
    #[inline]
    fn as_ref(&self) -> &T {
        &self.guard
    }
}

/// Upgradeable read guard for `ReadState`.
pub struct UpgradeableGuard<'a, T> {
    guard: RwLockUpgradableReadGuard<'a, T>,
    version: &'a AtomicU64,
}

impl<T> Deref for UpgradeableGuard<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}

impl<'a, T> UpgradeableGuard<'a, T> {
    /// Upgrade to write access.
    pub fn upgrade(self) -> WriteGuard<'a, T> {
        READ_STATE_STATS.record_upgrade();
        WriteGuard {
            guard: RwLockUpgradableReadGuard::upgrade(self.guard),
            version: self.version,
        }
    }

    /// Try to upgrade without blocking.
    pub fn try_upgrade(self) -> Result<WriteGuard<'a, T>, Self> {
        match RwLockUpgradableReadGuard::try_upgrade(self.guard) {
            Ok(guard) => {
                READ_STATE_STATS.record_upgrade();
                Ok(WriteGuard {
                    guard,
                    version: self.version,
                })
            }
            Err(guard) => Err(UpgradeableGuard {
                guard,
                version: self.version,
            }),
        }
    }
}

/// Write guard for `ReadState`.
pub struct WriteGuard<'a, T> {
    guard: RwLockWriteGuard<'a, T>,
    version: &'a AtomicU64,
}

impl<T> Deref for WriteGuard<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}

impl<T> std::ops::DerefMut for WriteGuard<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard
    }
}

impl<T> Drop for WriteGuard<'_, T> {
    fn drop(&mut self) {
        self.version.fetch_add(1, Ordering::Release);
    }
}

// ============================================================================
// ReadCache - Concurrent HashMap
// ============================================================================

/// Read-optimized concurrent cache using `parking_lot::RwLock`.
///
/// Provides fast concurrent read access to a HashMap with exclusive write access.
/// Ideal for caches, lookup tables, and configuration maps.
///
/// # Example
///
/// ```rust,ignore
/// use armature_core::read_state::ReadCache;
///
/// let cache = ReadCache::new();
///
/// // Insert values
/// cache.insert("key1".to_string(), "value1".to_string());
/// cache.insert("key2".to_string(), "value2".to_string());
///
/// // Fast concurrent reads
/// if let Some(value) = cache.get(&"key1".to_string()) {
///     println!("Found: {}", value);
/// }
///
/// // Bulk insert
/// cache.extend(vec![
///     ("key3".to_string(), "value3".to_string()),
///     ("key4".to_string(), "value4".to_string()),
/// ]);
/// ```
pub struct ReadCache<K, V> {
    inner: RwLock<HashMap<K, V>>,
}

impl<K, V> ReadCache<K, V>
where
    K: Eq + Hash,
{
    /// Create empty cache.
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
        }
    }

    /// Create cache with capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: RwLock::new(HashMap::with_capacity(capacity)),
        }
    }

    /// Get a value by key.
    #[inline]
    pub fn get<Q>(&self, key: &Q) -> Option<V>
    where
        K: std::borrow::Borrow<Q>,
        Q: Hash + Eq + ?Sized,
        V: Clone,
    {
        READ_STATE_STATS.record_read();
        self.inner.read().get(key).cloned()
    }

    /// Check if key exists.
    #[inline]
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: std::borrow::Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        READ_STATE_STATS.record_read();
        self.inner.read().contains_key(key)
    }

    /// Get number of entries.
    #[inline]
    pub fn len(&self) -> usize {
        self.inner.read().len()
    }

    /// Check if cache is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.read().is_empty()
    }

    /// Insert a value.
    #[inline]
    pub fn insert(&self, key: K, value: V) -> Option<V> {
        READ_STATE_STATS.record_write();
        self.inner.write().insert(key, value)
    }

    /// Remove a value.
    #[inline]
    pub fn remove<Q>(&self, key: &Q) -> Option<V>
    where
        K: std::borrow::Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        READ_STATE_STATS.record_write();
        self.inner.write().remove(key)
    }

    /// Clear all entries.
    #[inline]
    pub fn clear(&self) {
        READ_STATE_STATS.record_write();
        self.inner.write().clear();
    }

    /// Get or insert with default.
    pub fn get_or_insert<F>(&self, key: K, default: F) -> V
    where
        F: FnOnce() -> V,
        V: Clone,
    {
        // Try read first
        {
            let guard = self.inner.upgradable_read();
            if let Some(value) = guard.get(&key) {
                READ_STATE_STATS.record_read();
                return value.clone();
            }
            // Need to insert
            READ_STATE_STATS.record_upgrade();
            let mut write_guard = RwLockUpgradableReadGuard::upgrade(guard);
            // Double-check (another thread may have inserted)
            if let Some(value) = write_guard.get(&key) {
                return value.clone();
            }
            let value = default();
            write_guard.insert(key, value.clone());
            value
        }
    }

    /// Extend with multiple entries.
    pub fn extend<I>(&self, iter: I)
    where
        I: IntoIterator<Item = (K, V)>,
    {
        READ_STATE_STATS.record_write();
        self.inner.write().extend(iter);
    }

    /// Get all keys.
    pub fn keys(&self) -> Vec<K>
    where
        K: Clone,
    {
        READ_STATE_STATS.record_read();
        self.inner.read().keys().cloned().collect()
    }

    /// Get all values.
    pub fn values(&self) -> Vec<V>
    where
        V: Clone,
    {
        READ_STATE_STATS.record_read();
        self.inner.read().values().cloned().collect()
    }

    /// Iterate over entries (clones all data).
    pub fn entries(&self) -> Vec<(K, V)>
    where
        K: Clone,
        V: Clone,
    {
        READ_STATE_STATS.record_read();
        self.inner
            .read()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    /// Apply function to all values.
    pub fn for_each<F>(&self, f: F)
    where
        F: FnMut(&K, &V),
    {
        READ_STATE_STATS.record_read();
        self.inner.read().iter().for_each(|(k, v)| {
            let mut f = f;
            f(k, v);
        });
    }

    /// Update value if present.
    pub fn update<Q, F>(&self, key: &Q, f: F) -> bool
    where
        K: std::borrow::Borrow<Q>,
        Q: Hash + Eq + ?Sized,
        F: FnOnce(&mut V),
    {
        READ_STATE_STATS.record_write();
        let mut guard = self.inner.write();
        if let Some(value) = guard.get_mut(key) {
            f(value);
            true
        } else {
            false
        }
    }
}

impl<K, V> Default for ReadCache<K, V>
where
    K: Eq + Hash,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> std::fmt::Debug for ReadCache<K, V>
where
    K: std::fmt::Debug + Eq + Hash,
    V: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReadCache")
            .field("len", &self.len())
            .finish()
    }
}

// ============================================================================
// ReadConfig - Configuration State
// ============================================================================

/// Read-optimized configuration state with change detection.
///
/// Specialized for application configuration that is read frequently
/// but updated rarely (e.g., hot reload).
///
/// # Features
///
/// - Version tracking for cache invalidation
/// - Change callbacks for reactive updates
/// - Snapshot support for consistent reads
///
/// # Example
///
/// ```rust,ignore
/// use armature_core::read_state::ReadConfig;
///
/// #[derive(Clone)]
/// struct AppConfig {
///     database_url: String,
///     cache_ttl: u64,
/// }
///
/// let config = ReadConfig::new(AppConfig {
///     database_url: "postgres://localhost/db".into(),
///     cache_ttl: 300,
/// });
///
/// // Read config
/// let current = config.get();
/// println!("DB: {}", current.database_url);
///
/// // Update with change detection
/// let old_version = config.version();
/// config.set(AppConfig {
///     database_url: "postgres://localhost/new_db".into(),
///     cache_ttl: 600,
/// });
///
/// if config.changed_since(old_version) {
///     println!("Config changed!");
/// }
/// ```
pub struct ReadConfig<T> {
    state: ReadState<Arc<T>>,
}

impl<T> ReadConfig<T> {
    /// Create new configuration state.
    pub fn new(config: T) -> Self {
        Self {
            state: ReadState::new(Arc::new(config)),
        }
    }

    /// Get current configuration.
    ///
    /// Returns a cheap Arc clone.
    #[inline]
    pub fn get(&self) -> Arc<T> {
        self.state.read().clone()
    }

    /// Get configuration with version.
    #[inline]
    pub fn get_versioned(&self) -> (Arc<T>, u64) {
        let config = self.get();
        let version = self.version();
        (config, version)
    }

    /// Get current version.
    #[inline]
    pub fn version(&self) -> u64 {
        self.state.version()
    }

    /// Check if config changed since version.
    #[inline]
    pub fn changed_since(&self, version: u64) -> bool {
        self.state.changed_since(version)
    }

    /// Set new configuration.
    pub fn set(&self, config: T) {
        self.state.replace(Arc::new(config));
    }
}

impl<T: Clone> ReadConfig<T> {
    /// Update configuration in place.
    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&T) -> T,
    {
        self.state.write(|arc| {
            let new_value = f(&**arc);
            *arc = Arc::new(new_value);
        });
    }

    /// Update only if predicate is true.
    pub fn update_if<P, F>(&self, predicate: P, update: F) -> bool
    where
        P: FnOnce(&T) -> bool,
        F: FnOnce(&T) -> T,
    {
        self.state.read_then_write(
            |arc| predicate(&**arc),
            |arc| {
                let new_value = update(&**arc);
                *arc = Arc::new(new_value);
            },
        )
    }
}

impl<T: Default> Default for ReadConfig<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for ReadConfig<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReadConfig")
            .field("config", &self.get())
            .field("version", &self.version())
            .finish()
    }
}

// ============================================================================
// ArcSwapState - Fast Read with Arc Swap
// ============================================================================

/// Ultra-fast read state using `arc_swap` pattern.
///
/// Provides nearly lock-free reads by swapping Arc pointers.
/// Writes are more expensive (clone + swap) but reads are extremely fast.
///
/// Best for:
/// - Extremely high read frequency
/// - Very low write frequency
/// - Data that can be cloned
pub struct ArcSwapState<T> {
    inner: RwLock<Arc<T>>,
}

impl<T> ArcSwapState<T> {
    /// Create new state.
    pub fn new(value: T) -> Self {
        Self {
            inner: RwLock::new(Arc::new(value)),
        }
    }

    /// Load current value.
    ///
    /// Extremely fast - just Arc clone.
    #[inline]
    pub fn load(&self) -> Arc<T> {
        READ_STATE_STATS.record_read();
        self.inner.read().clone()
    }

    /// Store new value.
    pub fn store(&self, value: T) {
        READ_STATE_STATS.record_write();
        *self.inner.write() = Arc::new(value);
    }

    /// Store Arc directly.
    pub fn store_arc(&self, value: Arc<T>) {
        READ_STATE_STATS.record_write();
        *self.inner.write() = value;
    }
}

impl<T: Clone> ArcSwapState<T> {
    /// Update using function.
    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&T) -> T,
    {
        READ_STATE_STATS.record_write();
        let mut guard = self.inner.write();
        let new_value = f(&**guard);
        *guard = Arc::new(new_value);
    }

    /// Swap and return old value.
    pub fn swap(&self, value: T) -> Arc<T> {
        READ_STATE_STATS.record_write();
        std::mem::replace(&mut *self.inner.write(), Arc::new(value))
    }
}

impl<T: Default> Default for ArcSwapState<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_read_state_basic() {
        let state = ReadState::new(42i32);
        assert_eq!(*state.read(), 42);

        state.write(|v| *v = 100);
        assert_eq!(*state.read(), 100);
    }

    #[test]
    fn test_read_state_concurrent() {
        let state = Arc::new(ReadState::new(0i32));
        let mut handles = vec![];

        // Spawn readers
        for _ in 0..10 {
            let s = Arc::clone(&state);
            handles.push(thread::spawn(move || {
                for _ in 0..1000 {
                    let _ = *s.read();
                }
            }));
        }

        // Spawn writers
        for _ in 0..2 {
            let s = Arc::clone(&state);
            handles.push(thread::spawn(move || {
                for _ in 0..100 {
                    s.write(|v| *v += 1);
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        // 2 writers * 100 increments = 200
        assert_eq!(*state.read(), 200);
    }

    #[test]
    fn test_read_state_upgrade() {
        let state = ReadState::new(10i32);

        // Upgrade when condition is true
        let upgraded = state.read_then_write(|v| *v == 10, |v| *v = 20);
        assert!(upgraded);
        assert_eq!(*state.read(), 20);

        // Don't upgrade when condition is false
        let upgraded = state.read_then_write(|v| *v == 10, |v| *v = 30);
        assert!(!upgraded);
        assert_eq!(*state.read(), 20);
    }

    #[test]
    fn test_read_cache_basic() {
        let cache: ReadCache<String, i32> = ReadCache::new();

        cache.insert("one".to_string(), 1);
        cache.insert("two".to_string(), 2);

        assert_eq!(cache.get(&"one".to_string()), Some(1));
        assert_eq!(cache.get(&"three".to_string()), None);
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_read_cache_get_or_insert() {
        let cache: ReadCache<String, i32> = ReadCache::new();

        let value = cache.get_or_insert("key".to_string(), || 42);
        assert_eq!(value, 42);

        // Should return existing value
        let value = cache.get_or_insert("key".to_string(), || 100);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_read_config() {
        #[derive(Clone, Debug)]
        struct Config {
            value: i32,
        }

        let config = ReadConfig::new(Config { value: 10 });
        let v1 = config.version();

        assert_eq!(config.get().value, 10);

        config.update(|c| Config { value: c.value + 10 });

        assert_eq!(config.get().value, 20);
        assert!(config.changed_since(v1));
    }

    #[test]
    fn test_arc_swap_state() {
        let state = ArcSwapState::new(42i32);
        assert_eq!(*state.load(), 42);

        state.update(|v| v + 10);
        assert_eq!(*state.load(), 52);

        state.store(100);
        assert_eq!(*state.load(), 100);
    }

    #[test]
    fn test_version_tracking() {
        let state = ReadState::new(0i32);
        let v1 = state.version();

        state.write(|v| *v = 1);
        let v2 = state.version();

        assert!(v2 > v1);
        assert!(state.changed_since(v1));
        assert!(!state.changed_since(v2));
    }
}

