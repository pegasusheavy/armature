# Armature Grafana Dashboards

Pre-built Grafana dashboards for monitoring Armature web applications. These dashboards provide comprehensive visibility into your application's performance, security, caching, and background job processing.

## Dashboards

| Dashboard | File | Description |
|-----------|------|-------------|
| **Application Overview** | `armature-overview.json` | High-level metrics: request rates, latency, error rates, CPU/memory usage |
| **Authentication & Security** | `armature-auth-security.json` | Login attempts, JWT tokens, rate limiting, security events |
| **Cache & Redis** | `armature-cache-redis.json` | Cache hit rates, Redis connections, operation latency, evictions |
| **Queues & Jobs** | `armature-queues-jobs.json` | Job throughput, queue depth, processing latency, cron tasks |

## Prerequisites

- **Grafana 9.x+** (dashboards use v10 schema but are backward compatible)
- **Prometheus** as data source
- **Armature application** with metrics enabled

## Quick Start

### 1. Enable Metrics in Your Application

```rust
use armature::prelude::*;
use armature_core::metrics::MetricsConfig;

#[tokio::main]
async fn main() {
    let config = MetricsConfig::builder()
        .enabled(true)
        .endpoint("/metrics")
        .build();

    let app = Application::builder()
        .metrics(config)
        .build();

    app.listen(3000).await.unwrap();
}
```

### 2. Configure Prometheus Scrape

Add to your `prometheus.yml`:

```yaml
scrape_configs:
  - job_name: 'armature'
    static_configs:
      - targets: ['your-app:3000']
    metrics_path: '/metrics'
    scrape_interval: 15s
```

### 3. Import Dashboards into Grafana

**Option A: Via Grafana UI**
1. Go to **Dashboards** → **Import**
2. Upload the JSON file or paste its contents
3. Select your Prometheus data source
4. Click **Import**

**Option B: Via Grafana Provisioning**

Create `/etc/grafana/provisioning/dashboards/armature.yaml`:

```yaml
apiVersion: 1
providers:
  - name: 'Armature'
    orgId: 1
    folder: 'Armature'
    type: file
    options:
      path: /var/lib/grafana/dashboards/armature
```

Then copy the dashboard JSON files to `/var/lib/grafana/dashboards/armature/`.

## Dashboard Details

### Application Overview

Key metrics for application health and performance:

- **Request Rate** - HTTP requests per second
- **P95 Latency** - 95th percentile response time
- **Error Rate** - Percentage of 5xx responses
- **Active Instances** - Number of running instances
- **CPU/Memory Usage** - Resource consumption

Panels:
- Request Rate by HTTP Method
- Request Rate by Status Code
- Response Time Percentiles (p50, p90, p95, p99)
- Top 10 Endpoints by Request Rate
- CPU Usage by Instance
- Memory Usage by Instance
- Tokio Runtime Threads
- Active HTTP Connections

### Authentication & Security

Monitor authentication flows and security events:

- **Successful/Failed Logins** - Login attempt counts
- **Active Sessions** - Current session count
- **JWT Tokens Issued** - Token generation rate
- **Rate Limited Requests** - Blocked requests

Panels:
- Login Attempts by Status
- OAuth Callbacks by Provider
- Rate Limit Decisions
- Top Rate-Limited Endpoints
- Security Events by Type
- Blocked Requests by Reason
- Top Failed Login IPs (24h)

### Cache & Redis

Track caching effectiveness and Redis performance:

- **Cache Hit Rate** - Percentage of cache hits
- **Cache Ops/sec** - Operation throughput
- **Cache Memory** - Memory usage
- **P95 Cache Latency** - Operation latency

Panels:
- Hit Rate by Cache Tier (L1/L2)
- Cache Operations by Type (get/set/delete)
- Redis Pool Connections
- Redis Command Latency
- Top Redis Commands
- Cache Invalidations by Tag
- Cache Evictions by Reason

### Queues & Jobs

Monitor background job processing:

- **Pending Jobs** - Jobs waiting in queue
- **Processing Jobs** - Currently processing
- **Throughput** - Jobs completed per second
- **Failure Rate** - Percentage of failed jobs
- **Dead Letter Queue** - Failed job count

Panels:
- Queue Depth by Queue
- Job Throughput by Queue
- Job Processing Duration
- Job Wait Time (Queue Latency)
- Cron Task Executions
- Cron Task Duration
- Job Retries by Type
- Job Failures by Error Type

## Metric Names Reference

The dashboards expect these metric names from your Armature application:

### HTTP Metrics
```
http_requests_total{method, status, endpoint}
http_request_duration_seconds_bucket{le, endpoint}
http_connections_active
```

### Authentication Metrics
```
auth_login_attempts_total{status}
auth_active_sessions
auth_jwt_tokens_issued_total
auth_jwt_validation_failures_total
auth_oauth_callbacks_total{provider}
```

### Security Metrics
```
security_blocked_requests_total{reason}
security_events_total{event_type}
ratelimit_requests_total{status, endpoint}
```

### Cache Metrics
```
cache_hits_total{tier}
cache_misses_total{tier}
cache_operations_total{operation}
cache_entries_total
cache_memory_bytes
cache_operation_duration_seconds_bucket{le}
cache_evictions_total{reason}
cache_invalidations_total{tag}
```

### Redis Metrics
```
redis_pool_connections_active
redis_pool_connections_idle
redis_command_duration_seconds_bucket{le}
redis_commands_total{command}
```

### Queue Metrics
```
queue_jobs_pending{queue}
queue_jobs_processing{queue}
queue_jobs_completed_total{queue}
queue_jobs_failed_total{queue, error_type}
queue_jobs_dead_letter{queue}
queue_workers_active
queue_job_duration_seconds_bucket{le, job_type}
queue_job_wait_seconds_bucket{le, queue}
queue_job_retries_total{job_type}
```

### Cron Metrics
```
cron_executions_total{task_name, status}
cron_task_duration_seconds_bucket{le, task_name}
```

### Process Metrics
```
process_cpu_usage
process_resident_memory_bytes
process_virtual_memory_max_bytes
tokio_runtime_workers_count
tokio_runtime_blocking_threads
```

## Variables

All dashboards support these template variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `datasource` | Prometheus data source | Auto-detected |
| `job` | Prometheus job label | `armature` |

## Alerting

Each dashboard includes sensible thresholds for alerting:

| Metric | Warning | Critical |
|--------|---------|----------|
| P95 Latency | 100ms | 500ms |
| Error Rate | 1% | 5% |
| CPU Usage | 70% | 90% |
| Memory Usage | 70% | 90% |
| Cache Hit Rate | <90% | <70% |
| Dead Letter Queue | 10 | 50 |

To set up alerts:
1. Navigate to the panel
2. Click the panel title → Edit
3. Go to the **Alert** tab
4. Configure alert rules based on the thresholds

## Customization

### Adding Custom Panels

1. Click **Add panel** in Grafana
2. Use the metric names from the reference above
3. Save the dashboard

### Modifying Thresholds

1. Edit the panel
2. Go to **Field** tab → **Thresholds**
3. Adjust values as needed

### Changing Time Ranges

Default refresh: 30 seconds
Default time range: Last 1 hour

Adjust using the time picker in Grafana's top-right corner.

## Troubleshooting

### No Data Displayed

1. **Check Prometheus scraping**: Verify your app's `/metrics` endpoint returns data
2. **Check job label**: Ensure the `job` variable matches your Prometheus config
3. **Check data source**: Verify the Prometheus data source is correctly configured

### Missing Metrics

Ensure you've enabled the relevant features in your Armature application:

```rust
// Enable all metrics
let app = Application::builder()
    .metrics(MetricsConfig::default())
    .cache(CacheConfig::default())
    .queue(QueueConfig::default())
    .auth(AuthConfig::default())
    .build();
```

### Dashboard Import Errors

If you encounter JSON parse errors:
1. Ensure you're using Grafana 9.x or later
2. Try removing the `id` field from the JSON (set to `null`)
3. Check for syntax errors if you've modified the JSON

## Contributing

To contribute new dashboards or improvements:

1. Create/modify the dashboard in Grafana
2. Export as JSON (Share → Export → Save to file)
3. Remove the `id` field (set to `null`)
4. Submit a PR with the JSON file

## License

These dashboards are part of the Armature project and are released under the same license.

