# armature-metrics

Prometheus metrics and monitoring for the Armature framework.

## Features

- **Prometheus Format** - Standard metrics endpoint
- **Auto Instrumentation** - HTTP request metrics
- **Custom Metrics** - Counters, gauges, histograms
- **Labels** - Dimensional metrics
- **Push Gateway** - Push metrics to Prometheus

## Installation

```toml
[dependencies]
armature-metrics = "0.1"
```

## Quick Start

```rust
use armature_metrics::{Metrics, Counter, Histogram};

// Create metrics
let request_counter = Counter::new("http_requests_total", "Total HTTP requests");
let response_time = Histogram::new("http_response_time_seconds", "Response time");

// Record metrics
request_counter.inc();
response_time.observe(0.042);

// Expose /metrics endpoint
let app = Application::new()
    .with_middleware(MetricsMiddleware::new())
    .get("/metrics", metrics_handler());
```

## Auto Instrumentation

```rust
let app = Application::new()
    .with_middleware(MetricsMiddleware::auto());
```

Automatically records:
- `http_requests_total` - Request count by method, path, status
- `http_request_duration_seconds` - Request duration histogram
- `http_requests_in_flight` - Current active requests

## License

MIT OR Apache-2.0

