//! Gauge metrics
//!
//! Gauges are metrics that can increase and decrease.

pub use prometheus::Gauge;
pub use prometheus::GaugeVec;

/// Gauge metric builder
///
/// # Examples
///
/// ```
/// use armature_metrics::*;
///
/// let gauge = GaugeBuilder::new("active_connections", "Active connections")
///     .register()
///     .unwrap();
///
/// gauge.set(42.0);
/// gauge.inc();
/// gauge.dec();
/// ```
pub struct GaugeBuilder {
    name: String,
    help: String,
}

impl GaugeBuilder {
    /// Create a new gauge builder
    pub fn new(name: impl Into<String>, help: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            help: help.into(),
        }
    }

    /// Register the gauge
    pub fn register(self) -> Result<Gauge, prometheus::Error> {
        crate::register_gauge(&self.name, &self.help)
    }
}

/// Gauge with labels builder
///
/// # Examples
///
/// ```
/// use armature_metrics::*;
///
/// let gauge = GaugeVecBuilder::new("queue_size", "Queue size")
///     .labels(&["queue_name"])
///     .register()
///     .unwrap();
///
/// gauge.with_label_values(&["default"]).set(10.0);
/// ```
pub struct GaugeVecBuilder {
    name: String,
    help: String,
    label_names: Vec<String>,
}

impl GaugeVecBuilder {
    /// Create a new gauge vec builder
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

    /// Register the gauge vec
    pub fn register(self) -> Result<GaugeVec, prometheus::Error> {
        let label_refs: Vec<&str> = self.label_names.iter().map(|s| s.as_str()).collect();
        crate::register_gauge_vec(&self.name, &self.help, &label_refs)
    }
}
