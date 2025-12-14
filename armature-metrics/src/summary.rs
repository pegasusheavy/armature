//! Summary metrics
//!
//! Summaries calculate quantiles over a sliding time window.

// Re-export prometheus summary types for future use
// Note: Prometheus rust client doesn't have built-in Summary support yet
// This module is a placeholder for when it becomes available

/// Placeholder for future summary metric support
///
/// Summaries are not yet supported in the prometheus rust client.
/// Use histograms instead for now.
pub struct Summary;

impl Summary {
    /// Placeholder - not yet implemented
    pub fn new() -> Self {
        Self
    }
}

impl Default for Summary {
    fn default() -> Self {
        Self::new()
    }
}

