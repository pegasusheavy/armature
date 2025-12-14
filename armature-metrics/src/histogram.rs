//! Histogram metrics
//!
//! Histograms sample observations and count them in configurable buckets.

pub use prometheus::Histogram;
pub use prometheus::HistogramVec;

/// Default histogram buckets for HTTP request latency (in seconds)
pub const DEFAULT_LATENCY_BUCKETS: &[f64] = &[
    0.001, 0.005, 0.01, 0.025, 0.05, 0.075, 0.1, 0.25, 0.5, 0.75, 1.0, 2.5, 5.0, 7.5, 10.0,
];

/// Default histogram buckets for sizes (in bytes)
pub const DEFAULT_SIZE_BUCKETS: &[f64] = &[
    100.0, 1_000.0, 10_000.0, 100_000.0, 1_000_000.0, 10_000_000.0, 100_000_000.0,
];

/// Histogram metric builder
///
/// # Examples
///
/// ```
/// use armature_metrics::*;
///
/// let histogram = HistogramBuilder::new("request_duration_seconds", "Request duration")
///     .register()
///     .unwrap();
///
/// histogram.observe(0.5);
/// ```
pub struct HistogramBuilder {
    name: String,
    help: String,
    buckets: Option<Vec<f64>>,
}

impl HistogramBuilder {
    /// Create a new histogram builder
    pub fn new(name: impl Into<String>, help: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            help: help.into(),
            buckets: None,
        }
    }

    /// Set custom buckets
    pub fn buckets(mut self, buckets: Vec<f64>) -> Self {
        self.buckets = Some(buckets);
        self
    }

    /// Use default latency buckets
    pub fn latency_buckets(mut self) -> Self {
        self.buckets = Some(DEFAULT_LATENCY_BUCKETS.to_vec());
        self
    }

    /// Use default size buckets
    pub fn size_buckets(mut self) -> Self {
        self.buckets = Some(DEFAULT_SIZE_BUCKETS.to_vec());
        self
    }

    /// Register the histogram
    pub fn register(self) -> Result<Histogram, prometheus::Error> {
        if let Some(buckets) = self.buckets {
            crate::register_histogram_with_buckets(&self.name, &self.help, buckets)
        } else {
            crate::register_histogram(&self.name, &self.help)
        }
    }
}

/// Histogram with labels builder
///
/// # Examples
///
/// ```
/// use armature_metrics::*;
///
/// let histogram = HistogramVecBuilder::new("http_request_duration_seconds", "Request duration")
///     .labels(&["method", "endpoint"])
///     .latency_buckets()
///     .register()
///     .unwrap();
///
/// histogram.with_label_values(&["GET", "/api/users"]).observe(0.123);
/// ```
pub struct HistogramVecBuilder {
    name: String,
    help: String,
    label_names: Vec<String>,
    buckets: Option<Vec<f64>>,
}

impl HistogramVecBuilder {
    /// Create a new histogram vec builder
    pub fn new(name: impl Into<String>, help: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            help: help.into(),
            label_names: Vec::new(),
            buckets: None,
        }
    }

    /// Set label names
    pub fn labels(mut self, labels: &[&str]) -> Self {
        self.label_names = labels.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Set custom buckets
    pub fn buckets(mut self, buckets: Vec<f64>) -> Self {
        self.buckets = Some(buckets);
        self
    }

    /// Use default latency buckets
    pub fn latency_buckets(mut self) -> Self {
        self.buckets = Some(DEFAULT_LATENCY_BUCKETS.to_vec());
        self
    }

    /// Use default size buckets
    pub fn size_buckets(mut self) -> Self {
        self.buckets = Some(DEFAULT_SIZE_BUCKETS.to_vec());
        self
    }

    /// Register the histogram vec
    pub fn register(self) -> Result<HistogramVec, prometheus::Error> {
        let label_refs: Vec<&str> = self.label_names.iter().map(|s| s.as_str()).collect();

        if let Some(buckets) = self.buckets {
            crate::register_histogram_vec_with_buckets(&self.name, &self.help, &label_refs, buckets)
        } else {
            crate::register_histogram_vec(&self.name, &self.help, &label_refs)
        }
    }
}

