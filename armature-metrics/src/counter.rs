//! Counter metrics
//!
//! Counters are metrics that only increase over time.

pub use prometheus::Counter;
pub use prometheus::CounterVec;

/// Counter metric builder
///
/// # Examples
///
/// ```
/// use armature_metrics::*;
///
/// let counter = CounterBuilder::new("requests_total", "Total requests")
///     .register()
///     .unwrap();
///
/// counter.inc();
/// counter.inc_by(5.0);
/// ```
pub struct CounterBuilder {
    name: String,
    help: String,
}

impl CounterBuilder {
    /// Create a new counter builder
    pub fn new(name: impl Into<String>, help: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            help: help.into(),
        }
    }

    /// Register the counter
    pub fn register(self) -> Result<Counter, prometheus::Error> {
        crate::register_counter(&self.name, &self.help)
    }
}

/// Counter with labels builder
///
/// # Examples
///
/// ```
/// use armature_metrics::*;
///
/// let counter = CounterVecBuilder::new("http_requests_total", "Total HTTP requests")
///     .labels(&["method", "status"])
///     .register()
///     .unwrap();
///
/// counter.with_label_values(&["GET", "200"]).inc();
/// ```
pub struct CounterVecBuilder {
    name: String,
    help: String,
    label_names: Vec<String>,
}

impl CounterVecBuilder {
    /// Create a new counter vec builder
    pub fn new(name: impl Into<String>, help: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            help: help.into(),
            label_names: Vec::new(),
        }
    }

    /// Set label names
    pub fn labels(mut self, labels: &[&str]) -> Self {
        self.label_names = labels.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Register the counter vec
    pub fn register(self) -> Result<CounterVec, prometheus::Error> {
        let label_refs: Vec<&str> = self.label_names.iter().map(|s| s.as_str()).collect();
        crate::register_counter_vec(&self.name, &self.help, &label_refs)
    }
}

