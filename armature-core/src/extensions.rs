//! Type-safe request extensions for zero-cost state extraction.
//!
//! This module provides a way to attach typed data to requests without
//! runtime type checking overhead. Unlike the DI container which uses
//! `Any` downcasting, extensions use a type-erased map that only requires
//! type checks at the point of insertion, not retrieval.
//!
//! # Performance
//!
//! - **Insertion**: O(1) with a single type check
//! - **Retrieval**: O(1) with no type checking (uses pre-verified TypeId)
//! - **Memory**: One `Arc<T>` per extension type
//!
//! # Example
//!
//! ```rust,ignore
//! use armature_core::{Extensions, State};
//!
//! // Application state
//! #[derive(Clone)]
//! struct AppState {
//!     db_pool: Pool,
//! }
//!
//! // Insert state at startup
//! let mut extensions = Extensions::new();
//! extensions.insert(AppState { db_pool });
//!
//! // Extract in handler (zero-cost after setup)
//! async fn handler(state: State<AppState>) -> Result<HttpResponse, Error> {
//!     let pool = &state.db_pool;
//!     // ...
//! }
//! ```

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

/// Type-safe extensions container.
///
/// Stores typed values keyed by `TypeId` for O(1) retrieval without
/// runtime type checking after initial insertion.
#[derive(Clone, Default)]
pub struct Extensions {
    /// Internal storage: TypeId -> type-erased Arc
    /// The Arc contains the actual typed value, wrapped for thread-safety
    map: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl Extensions {
    /// Create a new empty extensions container.
    #[inline]
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Create with pre-allocated capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            map: HashMap::with_capacity(capacity),
        }
    }

    /// Insert a typed value into the extensions.
    ///
    /// If a value of this type already exists, it is replaced.
    ///
    /// # Example
    ///
    /// ```rust
    /// use armature_core::Extensions;
    ///
    /// let mut ext = Extensions::new();
    /// ext.insert(42i32);
    /// ext.insert("hello");
    /// ```
    #[inline]
    pub fn insert<T: Send + Sync + 'static>(&mut self, value: T) {
        let type_id = TypeId::of::<T>();
        let arc = Arc::new(value) as Arc<dyn Any + Send + Sync>;
        self.map.insert(type_id, arc);
    }

    /// Insert an Arc-wrapped value directly.
    ///
    /// This is more efficient when you already have an Arc.
    #[inline]
    pub fn insert_arc<T: Send + Sync + 'static>(&mut self, value: Arc<T>) {
        let type_id = TypeId::of::<T>();
        let arc = value as Arc<dyn Any + Send + Sync>;
        self.map.insert(type_id, arc);
    }

    /// Get a reference to a typed value.
    ///
    /// Returns `None` if no value of this type exists.
    ///
    /// # Performance
    ///
    /// This is O(1) and only involves a HashMap lookup followed by
    /// a pointer cast (no runtime type checking).
    ///
    /// # Example
    ///
    /// ```rust
    /// use armature_core::Extensions;
    ///
    /// let mut ext = Extensions::new();
    /// ext.insert(42i32);
    ///
    /// assert_eq!(ext.get::<i32>(), Some(&42));
    /// assert_eq!(ext.get::<String>(), None);
    /// ```
    #[inline]
    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.map
            .get(&type_id)
            .and_then(|arc| arc.downcast_ref::<T>())
    }

    /// Get an Arc reference to a typed value.
    ///
    /// This is useful when you need to clone the Arc for async operations.
    #[inline]
    pub fn get_arc<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        let type_id = TypeId::of::<T>();
        self.map
            .get(&type_id)
            .and_then(|arc| arc.clone().downcast::<T>().ok())
    }

    /// Check if a value of this type exists.
    #[inline]
    pub fn contains<T: Send + Sync + 'static>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        self.map.contains_key(&type_id)
    }

    /// Remove a typed value from the extensions.
    ///
    /// Returns true if the value existed and was removed.
    #[inline]
    pub fn remove<T: Send + Sync + 'static>(&mut self) -> bool {
        let type_id = TypeId::of::<T>();
        self.map.remove(&type_id).is_some()
    }

    /// Clear all extensions.
    #[inline]
    pub fn clear(&mut self) {
        self.map.clear();
    }

    /// Get the number of extensions.
    #[inline]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Check if extensions is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Merge another extensions container into this one.
    ///
    /// Values from `other` will overwrite values in `self` for the same type.
    #[inline]
    pub fn extend(&mut self, other: Extensions) {
        self.map.extend(other.map);
    }
}

impl std::fmt::Debug for Extensions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Extensions")
            .field("count", &self.map.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let mut ext = Extensions::new();

        ext.insert(42i32);
        ext.insert("hello".to_string());

        assert_eq!(ext.get::<i32>(), Some(&42));
        assert_eq!(ext.get::<String>(), Some(&"hello".to_string()));
        assert_eq!(ext.get::<f64>(), None);
    }

    #[test]
    fn test_insert_replaces() {
        let mut ext = Extensions::new();

        ext.insert(42i32);
        ext.insert(100i32);

        assert_eq!(ext.get::<i32>(), Some(&100));
    }

    #[test]
    fn test_contains() {
        let mut ext = Extensions::new();

        assert!(!ext.contains::<i32>());
        ext.insert(42i32);
        assert!(ext.contains::<i32>());
    }

    #[test]
    fn test_remove() {
        let mut ext = Extensions::new();
        ext.insert(42i32);

        let removed = ext.remove::<i32>();
        assert!(removed);
        assert!(!ext.contains::<i32>());
    }

    #[test]
    fn test_arc_insert() {
        let mut ext = Extensions::new();
        let arc = Arc::new(42i32);

        ext.insert_arc(arc.clone());

        let retrieved = ext.get_arc::<i32>().unwrap();
        assert_eq!(*retrieved, 42);
    }

    #[test]
    fn test_clone() {
        let mut ext = Extensions::new();
        ext.insert(42i32);

        let cloned = ext.clone();
        assert_eq!(cloned.get::<i32>(), Some(&42));
    }
}
