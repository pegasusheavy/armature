import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterModule } from '@angular/router';

interface RoadmapItem {
  feature: string;
  description: string;
  module: string;
  status: 'completed' | 'critical' | 'high' | 'medium' | 'low';
}

interface RoadmapCategory {
  name: string;
  items: RoadmapItem[];
}

@Component({
  selector: 'app-roadmap',
  standalone: true,
  imports: [CommonModule, RouterModule],
  templateUrl: './roadmap.component.html',
  styleUrls: ['./roadmap.component.scss'],
})
export class RoadmapComponent {
  phases = [
    {
      name: 'Phase 1: Production Essentials',
      quarter: 'Q1 ✅',
      items: [
        { feature: 'Health Check Module', status: 'completed' },
        { feature: 'Request Timeout & Size Limits', status: 'completed' },
        { feature: 'Global Exception Filters', status: 'completed' },
        { feature: 'Pagination Helpers', status: 'completed' },
        { feature: 'Circuit Breaker', status: 'completed' },
        { feature: 'Graceful Shutdown', status: 'completed' },
        { feature: 'Multipart Upload', status: 'completed' },
        { feature: 'SMTP Integration', status: 'completed' },
      ],
    },
    {
      name: 'Phase 2: Enterprise Features',
      quarter: 'Q2 ✅',
      items: [
        { feature: 'Prometheus Metrics', status: 'completed' },
        { feature: 'API Versioning', status: 'completed' },
        { feature: 'Audit Logging', status: 'completed' },
        { feature: 'Multi-tenancy', status: 'completed' },
        { feature: 'Feature Flags', status: 'completed' },
        { feature: 'i18n Support', status: 'high' },
      ],
    },
    {
      name: 'Phase 3: Advanced Capabilities',
      quarter: 'Q3 ✅',
      items: [
        { feature: 'RabbitMQ/Kafka Integration', status: 'completed' },
        { feature: 'Event Bus', status: 'completed' },
        { feature: 'HTTP Client with Retry', status: 'completed' },
        { feature: 'Distributed Locks', status: 'completed' },
        { feature: 'Request Correlation', status: 'completed' },
        { feature: 'Cloud Provider SDKs', status: 'completed' },
      ],
    },
    {
      name: 'Phase 4: Developer Experience',
      quarter: 'Q4 ✅',
      items: [
        { feature: 'Test Containers', status: 'completed' },
        { feature: 'Push Notifications', status: 'completed' },
        { feature: 'Advanced Caching', status: 'completed' },
        { feature: 'Serverless Deployment', status: 'completed' },
        { feature: 'CLI Enhancements', status: 'completed' },
        { feature: 'Grafana Dashboards', status: 'medium' },
      ],
    },
  ];

  categories: RoadmapCategory[] = [
    {
      name: 'Request & Response Handling',
      items: [
        { feature: 'Request Timeout', description: 'Configurable request timeouts with graceful handling', module: 'armature-core', status: 'completed' },
        { feature: 'Request Size Limits', description: 'Max body size, max header size configuration', module: 'armature-core', status: 'completed' },
        { feature: 'Content Negotiation', description: 'Accept header parsing, response format selection', module: 'armature-core', status: 'completed' },
        { feature: 'ETags & Conditional Requests', description: 'If-Match, If-None-Match, If-Modified-Since support', module: 'armature-core', status: 'completed' },
        { feature: 'Response Caching Headers', description: 'Cache-Control, Expires, Vary header helpers', module: 'armature-core', status: 'completed' },
        { feature: 'Streaming Responses', description: 'Chunked transfer encoding, streaming large files', module: 'armature-core', status: 'completed' },
        { feature: 'Request Extractors', description: 'Body, Query, Path, Header extractors', module: 'armature-core', status: 'completed' },
      ],
    },
    {
      name: 'Routing & Controllers',
      items: [
        { feature: 'API Versioning', description: 'URL-based, header-based, and query-based versioning', module: 'armature-core', status: 'completed' },
        { feature: 'Route Groups', description: 'Group routes with shared middleware/guards', module: 'armature-core', status: 'completed' },
        { feature: 'Route Constraints', description: 'Parameter validation at route level', module: 'armature-core', status: 'completed' },
        { feature: 'Middleware Decorator', description: 'Apply middleware via #[use_middleware] syntax', module: 'armature-macro', status: 'completed' },
        { feature: 'Guard Decorator', description: 'Apply guards via #[use_guard] syntax', module: 'armature-macro', status: 'completed' },
        { feature: 'Path Parameters', description: ':id style path parameters', module: 'armature-core', status: 'completed' },
        { feature: 'Query Parameters', description: 'Query string parsing', module: 'armature-core', status: 'completed' },
      ],
    },
    {
      name: 'Error Handling',
      items: [
        { feature: 'Global Exception Filters', description: 'Centralized error transformation', module: 'armature-core', status: 'completed' },
        { feature: 'Problem Details (RFC 7807)', description: 'Standardized error response format', module: 'armature-core', status: 'completed' },
        { feature: 'Error Correlation', description: 'Tie errors to request IDs for debugging', module: 'armature-core', status: 'completed' },
        { feature: 'HTTP Status Errors', description: 'Type-safe error responses', module: 'armature-core', status: 'completed' },
      ],
    },
    {
      name: 'Observability & Monitoring',
      items: [
        { feature: 'Health Check Module', description: '/health, /ready, /live endpoints', module: 'armature-core', status: 'completed' },
        { feature: 'Custom Health Indicators', description: 'Register custom health checks', module: 'armature-core', status: 'completed' },
        { feature: 'Kubernetes Probes', description: 'K8s-compatible probe endpoints', module: 'armature-core', status: 'completed' },
        { feature: 'OpenTelemetry', description: 'Distributed tracing and metrics', module: 'armature-opentelemetry', status: 'completed' },
        { feature: 'Structured Logging', description: 'JSON logging with tracing integration', module: 'armature-core', status: 'completed' },
        { feature: 'Prometheus Metrics', description: '/metrics endpoint with custom metrics', module: 'armature-metrics', status: 'completed' },
        { feature: 'Request Metrics', description: 'Request count, latency, error rates', module: 'armature-metrics', status: 'completed' },
        { feature: 'Business Metrics', description: 'Custom metric registration', module: 'armature-metrics', status: 'completed' },
        { feature: 'Grafana Dashboards', description: 'Pre-built dashboard templates', module: 'docs/', status: 'medium' },
      ],
    },
    {
      name: 'Audit & Compliance',
      items: [
        { feature: 'Audit Logging', description: 'Track who did what, when', module: 'armature-audit', status: 'completed' },
        { feature: 'Request/Response Logging', description: 'Configurable payload logging', module: 'armature-audit', status: 'completed' },
        { feature: 'Data Masking', description: 'Mask sensitive data in logs (PII, passwords, credit cards)', module: 'armature-audit', status: 'completed' },
        { feature: 'Retention Policies', description: 'Auto-cleanup old audit logs', module: 'armature-audit', status: 'completed' },
      ],
    },
    {
      name: 'Resilience & Reliability',
      items: [
        { feature: 'Circuit Breaker', description: 'Open/Closed/Half-Open states, sliding window', module: 'armature-core', status: 'completed' },
        { feature: 'Retry with Backoff', description: 'Constant, linear, exponential, jitter strategies', module: 'armature-core', status: 'completed' },
        { feature: 'Bulkhead Pattern', description: 'Semaphore-based resource isolation', module: 'armature-core', status: 'completed' },
        { feature: 'Timeout Policies', description: 'Timeout configuration per endpoint', module: 'armature-core', status: 'completed' },
        { feature: 'Fallback Handlers', description: 'Graceful degradation with chains', module: 'armature-core', status: 'completed' },
        { feature: 'Connection Draining', description: 'Wait for in-flight requests on shutdown', module: 'armature-core', status: 'completed' },
        { feature: 'Shutdown Hooks', description: 'Custom cleanup on shutdown', module: 'armature-core', status: 'completed' },
        { feature: 'Lifecycle Hooks', description: 'OnApplicationShutdown, OnModuleInit', module: 'armature-core', status: 'completed' },
      ],
    },
    {
      name: 'API Features',
      items: [
        { feature: 'Pagination Helpers', description: 'Offset, cursor-based pagination', module: 'armature-core', status: 'completed' },
        { feature: 'Sorting Helpers', description: 'Multi-field sorting', module: 'armature-core', status: 'completed' },
        { feature: 'Filtering Helpers', description: 'Query parameter filtering', module: 'armature-core', status: 'completed' },
        { feature: 'Search Helpers', description: 'Full-text search integration', module: 'armature-core', status: 'completed' },
        { feature: 'Field Selection', description: 'Sparse fieldsets (GraphQL-like)', module: 'armature-core', status: 'completed' },
      ],
    },
    {
      name: 'File Storage',
      items: [
        { feature: 'Multipart Upload', description: 'Streaming file upload with constraints', module: 'armature-storage', status: 'completed' },
        { feature: 'File Validation', description: 'Type, size, extension validation', module: 'armature-storage', status: 'completed' },
        { feature: 'Local Storage', description: 'Filesystem storage with paths', module: 'armature-storage', status: 'completed' },
        { feature: 'AWS S3', description: 'Presigned URLs, server-side encryption', module: 'armature-storage', status: 'completed' },
        { feature: 'Google Cloud Storage', description: 'Signed URLs, resumable uploads', module: 'armature-storage', status: 'completed' },
        { feature: 'Azure Blob Storage', description: 'Azurite support, SAS tokens', module: 'armature-storage', status: 'completed' },
      ],
    },
    {
      name: 'Email',
      items: [
        { feature: 'SMTP Transport', description: 'TLS, STARTTLS, connection pooling', module: 'armature-mail', status: 'completed' },
        { feature: 'Email Templates', description: 'Handlebars-based HTML templates', module: 'armature-mail', status: 'completed' },
        { feature: 'SendGrid Integration', description: 'SendGrid API support', module: 'armature-mail', status: 'completed' },
        { feature: 'AWS SES', description: 'AWS Simple Email Service', module: 'armature-mail', status: 'completed' },
        { feature: 'Mailgun', description: 'Mailgun API support', module: 'armature-mail', status: 'completed' },
        { feature: 'Email Queue', description: 'Async sending, retries, dead letter queue', module: 'armature-mail', status: 'completed' },
      ],
    },
    {
      name: 'Messaging & Events',
      items: [
        { feature: 'RabbitMQ', description: 'Message broker integration', module: 'armature-messaging', status: 'completed' },
        { feature: 'Apache Kafka', description: 'Event streaming platform', module: 'armature-messaging', status: 'completed' },
        { feature: 'NATS', description: 'Cloud-native messaging', module: 'armature-messaging', status: 'completed' },
        { feature: 'AWS SQS/SNS', description: 'AWS messaging services', module: 'armature-messaging', status: 'completed' },
        { feature: 'Job Queue', description: 'Redis-based background jobs', module: 'armature-queue', status: 'completed' },
        { feature: 'Event Bus', description: 'In-process event publishing', module: 'armature-events', status: 'completed' },
        { feature: 'Event Sourcing', description: 'Event-sourced aggregates', module: 'armature-eventsourcing', status: 'completed' },
        { feature: 'CQRS', description: 'Command/Query separation', module: 'armature-cqrs', status: 'completed' },
      ],
    },
    {
      name: 'External APIs & Networking',
      items: [
        { feature: 'HTTP Client', description: 'Built-in client with retry and circuit breaker', module: 'armature-http-client', status: 'completed' },
        { feature: 'gRPC', description: 'Server and client support', module: 'armature-grpc', status: 'completed' },
        { feature: 'GraphQL Client', description: 'Client for federation and subscriptions', module: 'armature-graphql-client', status: 'completed' },
        { feature: 'WebSocket & SSE', description: 'Real-time bidirectional communication', module: 'armature-core', status: 'completed' },
        { feature: 'Webhooks', description: 'Webhook sending and receiving', module: 'armature-core', status: 'completed' },
      ],
    },
    {
      name: 'Authentication & Security',
      items: [
        { feature: 'JWT Authentication', description: 'Token signing, verification, refresh', module: 'armature-jwt', status: 'completed' },
        { feature: 'OAuth2/OIDC', description: 'Google, Microsoft, GitHub, Discord, and more', module: 'armature-auth', status: 'completed' },
        { feature: 'SAML 2.0', description: 'Enterprise SSO integration', module: 'armature-auth', status: 'completed' },
        { feature: 'API Key Management', description: 'Generation, rotation, scopes', module: 'armature-auth', status: 'completed' },
        { feature: 'Two-Factor Auth', description: 'TOTP/HOTP support', module: 'armature-auth', status: 'completed' },
        { feature: 'Passwordless Auth', description: 'Magic links, WebAuthn', module: 'armature-auth', status: 'completed' },
        { feature: 'Rate Limiting', description: 'Token bucket, sliding window', module: 'armature-ratelimit', status: 'completed' },
        { feature: 'Security Headers', description: 'CSP, HSTS, X-Frame-Options, etc.', module: 'armature-security', status: 'completed' },
        { feature: 'Request Signing', description: 'HMAC-SHA256 verification', module: 'armature-security', status: 'completed' },
      ],
    },
    {
      name: 'Multi-tenancy & Enterprise',
      items: [
        { feature: 'Tenant Isolation', description: 'Request-scoped tenant context', module: 'armature-tenancy', status: 'completed' },
        { feature: 'Database per Tenant', description: 'Separate database connections', module: 'armature-tenancy', status: 'completed' },
        { feature: 'Schema per Tenant', description: 'PostgreSQL schema isolation', module: 'armature-tenancy', status: 'completed' },
        { feature: 'Tenant Middleware', description: 'Auto tenant resolution', module: 'armature-tenancy', status: 'completed' },
        { feature: 'Feature Flags', description: 'Runtime toggling, A/B testing, gradual rollout', module: 'armature-features', status: 'completed' },
      ],
    },
    {
      name: 'Internationalization',
      items: [
        { feature: 'i18n Support', description: 'Message translation system', module: 'armature-i18n', status: 'high' },
        { feature: 'Locale Detection', description: 'Accept-Language header parsing', module: 'armature-i18n', status: 'high' },
        { feature: 'Pluralization', description: 'Plural rules support', module: 'armature-i18n', status: 'medium' },
        { feature: 'Date/Number Formatting', description: 'Locale-aware formatting', module: 'armature-i18n', status: 'medium' },
      ],
    },
    {
      name: 'Developer Experience',
      items: [
        { feature: 'Code Generation', description: 'Controllers, services, modules', module: 'armature-cli', status: 'completed' },
        { feature: 'Project Templates', description: 'Starter templates', module: 'armature-cli', status: 'completed' },
        { feature: 'Dev Server', description: 'Hot reloading development', module: 'armature-cli', status: 'completed' },
        { feature: 'Interactive REPL', description: 'Interactive Rust shell', module: 'armature-cli', status: 'completed' },
        { feature: 'Route Listing', description: 'armature routes command', module: 'armature-cli', status: 'completed' },
        { feature: 'Config Validation', description: 'armature config:check', module: 'armature-cli', status: 'completed' },
        { feature: 'OpenAPI Generation', description: 'Swagger/OpenAPI documentation', module: 'armature-openapi', status: 'completed' },
        { feature: 'API Playground', description: 'Interactive API testing UI', module: 'armature-openapi', status: 'completed' },
      ],
    },
    {
      name: 'Testing',
      items: [
        { feature: 'Unit Test Helpers', description: 'Mocks, spies, assertions', module: 'armature-testing', status: 'completed' },
        { feature: 'Integration Test Helpers', description: 'Database setup/teardown', module: 'armature-testing', status: 'completed' },
        { feature: 'Test Containers', description: 'Docker-based testing (Postgres, Redis, MongoDB)', module: 'armature-testing', status: 'completed' },
        { feature: 'Load Testing', description: 'Performance test utilities', module: 'armature-testing', status: 'completed' },
        { feature: 'Contract Testing', description: 'Pact-compatible consumer-driven contracts', module: 'armature-testing', status: 'completed' },
      ],
    },
    {
      name: 'Distributed Systems',
      items: [
        { feature: 'Distributed Locks', description: 'Redis-based locks with TTL and RAII', module: 'armature-distributed', status: 'completed' },
        { feature: 'Leader Election', description: 'Automatic failover with callbacks', module: 'armature-distributed', status: 'completed' },
        { feature: 'Service Discovery', description: 'Consul, etcd integration', module: 'armature-discovery', status: 'completed' },
        { feature: 'Request Correlation', description: 'Correlation ID propagation', module: 'armature-core', status: 'completed' },
      ],
    },
    {
      name: 'Caching',
      items: [
        { feature: 'Redis Cache', description: 'Distributed caching', module: 'armature-cache', status: 'completed' },
        { feature: 'Memcached', description: 'High-performance caching', module: 'armature-cache', status: 'completed' },
        { feature: 'Cache Decorators', description: '#[cache] method decorator', module: 'armature-cache', status: 'completed' },
        { feature: 'Tag-Based Invalidation', description: 'Bulk cache busting by tags', module: 'armature-cache', status: 'completed' },
        { feature: 'Multi-Tier Caching', description: 'L1/L2 cache layers with auto-promotion', module: 'armature-cache', status: 'completed' },
      ],
    },
    {
      name: 'Push Notifications',
      items: [
        { feature: 'Web Push', description: 'VAPID-based browser notifications', module: 'armature-push', status: 'completed' },
        { feature: 'Firebase Cloud Messaging', description: 'FCM for Android and web', module: 'armature-push', status: 'completed' },
        { feature: 'Apple Push Notifications', description: 'APNS for iOS', module: 'armature-push', status: 'completed' },
        { feature: 'Unified Push Service', description: 'Single API for all platforms', module: 'armature-push', status: 'completed' },
      ],
    },
    {
      name: 'Cloud Providers',
      items: [
        { feature: 'AWS SDK', description: 'S3, DynamoDB, SQS, SNS, Lambda, KMS, and more', module: 'armature-aws', status: 'completed' },
        { feature: 'Google Cloud SDK', description: 'Storage, Pub/Sub, Firestore, BigQuery', module: 'armature-gcp', status: 'completed' },
        { feature: 'Azure SDK', description: 'Blob, Queue, Cosmos, Service Bus, Key Vault', module: 'armature-azure', status: 'completed' },
        { feature: 'Centralized Redis', description: 'Shared Redis client for all crates', module: 'armature-redis', status: 'completed' },
      ],
    },
    {
      name: 'Serverless Deployment',
      items: [
        { feature: 'AWS Lambda', description: 'API Gateway, ALB, Function URLs', module: 'armature-lambda', status: 'completed' },
        { feature: 'Google Cloud Run', description: 'Container-based serverless', module: 'armature-cloudrun', status: 'completed' },
        { feature: 'Azure Functions', description: 'HTTP triggers, custom handlers', module: 'armature-azure-functions', status: 'completed' },
      ],
    },
    {
      name: 'Containerization & CI/CD',
      items: [
        { feature: 'Dockerfile Templates', description: 'Optimized Alpine-based, multi-stage builds', module: 'templates/', status: 'completed' },
        { feature: 'Docker Compose', description: 'Development environment with all services', module: 'templates/', status: 'completed' },
        { feature: 'Kubernetes Manifests', description: 'Deployment, Service, Ingress, HPA, PDB', module: 'templates/k8s/', status: 'completed' },
        { feature: 'Helm Charts', description: 'Production-ready Helm chart', module: 'templates/helm/', status: 'completed' },
        { feature: 'GitHub Actions', description: 'CI, Release, Docs workflows', module: '.github/', status: 'completed' },
        { feature: 'Jenkins Pipelines', description: 'Basic, Docker agent, multibranch', module: 'templates/jenkins/', status: 'completed' },
      ],
    },
  ];

  completedFeatures = [
    'Core: DI, Controllers, Modules, Routing, Middleware, Guards, Interceptors',
    'Request Handling: Timeouts, Size Limits, Extractors, Streaming',
    'API: Versioning, Content Negotiation, ETags, Pagination, Filtering',
    'Error Handling: Global Filters, RFC 7807, Error Correlation',
    'Auth: JWT, OAuth2 (10+ providers), SAML 2.0, 2FA, Passwordless',
    'Security: Rate Limiting, CSP, HSTS, Request Signing, CORS',
    'Caching: Redis, Memcached, Multi-tier, Tag Invalidation',
    'Messaging: RabbitMQ, Kafka, NATS, AWS SQS/SNS',
    'Event-Driven: Event Bus, Event Sourcing, CQRS',
    'Storage: Local, S3, GCS, Azure Blob, Multipart Upload',
    'Email: SMTP, SendGrid, SES, Mailgun, Email Queue',
    'Push: Web Push, FCM, APNS, Unified Push Service',
    'Observability: OpenTelemetry, Prometheus, Health Checks, Audit Logging',
    'Resilience: Circuit Breaker, Retry, Bulkhead, Timeout, Fallback',
    'Multi-tenancy: Tenant Isolation, DB/Schema per Tenant, Feature Flags',
    'Distributed: Locks, Leader Election, Service Discovery',
    'Testing: Unit, Integration, Test Containers, Load, Contract',
    'Cloud: AWS, GCP, Azure SDKs, Lambda, Cloud Run, Azure Functions',
    'DevOps: Docker, Kubernetes, Helm, GitHub Actions, Jenkins',
    'CLI: Code Gen, Templates, Dev Server, REPL, Route Listing',
  ];

  getStatusClass(status: string): string {
    switch (status) {
      case 'completed':
        return 'bg-emerald-900/50 text-emerald-400 border border-emerald-800';
      case 'critical':
        return 'bg-red-900/50 text-red-400 border border-red-800';
      case 'high':
        return 'bg-orange-900/50 text-orange-400 border border-orange-800';
      case 'medium':
        return 'bg-yellow-900/50 text-yellow-400 border border-yellow-800';
      case 'low':
        return 'bg-green-900/50 text-green-400 border border-green-800';
      default:
        return 'bg-stone-800 text-stone-400 border border-stone-700';
    }
  }

  getStatusIcon(status: string): string {
    switch (status) {
      case 'completed':
        return '✅';
      case 'critical':
        return '🔴';
      case 'high':
        return '🟠';
      case 'medium':
        return '🟡';
      case 'low':
        return '🟢';
      default:
        return '⚪';
    }
  }

  getCompletedCount(items: RoadmapItem[]): number {
    return items.filter((item) => item.status === 'completed').length;
  }

  getProgress(items: { feature: string; status: string }[]): number {
    const completed = items.filter((item) => item.status === 'completed').length;
    return Math.round((completed / items.length) * 100);
  }

  getTotalCompleted(): number {
    return this.categories.reduce((sum, cat) => sum + this.getCompletedCount(cat.items), 0);
  }

  getTotalFeatures(): number {
    return this.categories.reduce((sum, cat) => sum + cat.items.length, 0);
  }
}
