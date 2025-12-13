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
      quarter: 'Q1',
      items: [
        { feature: 'Health Check Module', status: 'completed' },
        { feature: 'Request Timeout & Size Limits', status: 'completed' },
        { feature: 'Global Exception Filters', status: 'critical' },
        { feature: 'Pagination Helpers', status: 'critical' },
        { feature: 'Circuit Breaker', status: 'critical' },
        { feature: 'Connection Draining (Graceful Shutdown)', status: 'critical' },
        { feature: 'Multipart Upload', status: 'critical' },
        { feature: 'SMTP Integration', status: 'critical' },
      ],
    },
    {
      name: 'Phase 2: Enterprise Features',
      quarter: 'Q2',
      items: [
        { feature: 'Prometheus Metrics', status: 'high' },
        { feature: 'API Versioning', status: 'completed' },
        { feature: 'Audit Logging', status: 'high' },
        { feature: 'Multi-tenancy', status: 'high' },
        { feature: 'Feature Flags', status: 'high' },
        { feature: 'i18n Support', status: 'high' },
      ],
    },
    {
      name: 'Phase 3: Advanced Capabilities',
      quarter: 'Q3',
      items: [
        { feature: 'RabbitMQ/Kafka Integration', status: 'high' },
        { feature: 'Event Bus', status: 'high' },
        { feature: 'S3/GCS Storage', status: 'high' },
        { feature: 'HTTP Client with Retry', status: 'high' },
        { feature: 'Distributed Locks', status: 'high' },
        { feature: 'Request Correlation', status: 'completed' },
      ],
    },
    {
      name: 'Phase 4: Developer Experience',
      quarter: 'Q4',
      items: [
        { feature: 'Admin Dashboard', status: 'medium' },
        { feature: 'VS Code Extension', status: 'medium' },
        { feature: 'Test Containers', status: 'medium' },
        { feature: 'gRPC Support', status: 'medium' },
        { feature: 'Push Notifications', status: 'medium' },
        { feature: 'Advanced Caching', status: 'medium' },
      ],
    },
  ];

  categories: RoadmapCategory[] = [
    {
      name: 'Request/Response Handling',
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
        { feature: 'Route Groups', description: 'Group routes with shared middleware/guards', module: 'armature-core', status: 'high' },
        { feature: 'Route Constraints', description: 'Parameter validation at route level', module: 'armature-core', status: 'high' },
        { feature: '#[use_middleware] Decorator', description: 'Apply middleware via decorator syntax', module: 'armature-macro', status: 'completed' },
        { feature: '#[use_guard] Decorator', description: 'Apply guards via decorator syntax', module: 'armature-macro', status: 'completed' },
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
      name: 'Health Checks & Observability',
      items: [
        { feature: 'Health Check Module', description: '/health, /ready, /live endpoints', module: 'armature-core', status: 'completed' },
        { feature: 'Custom Health Indicators', description: 'Register custom health checks', module: 'armature-core', status: 'completed' },
        { feature: 'Kubernetes Probes', description: 'K8s-compatible probe endpoints', module: 'armature-core', status: 'completed' },
        { feature: 'OpenTelemetry', description: 'Distributed tracing and metrics', module: 'armature-opentelemetry', status: 'completed' },
        { feature: 'Logging', description: 'Structured logging', module: 'armature-core', status: 'completed' },
        { feature: 'Prometheus Metrics', description: '/metrics endpoint with custom metrics', module: 'armature-metrics', status: 'critical' },
        { feature: 'Request Metrics', description: 'Request count, latency, error rates', module: 'armature-metrics', status: 'high' },
        { feature: 'Business Metrics', description: 'Custom metric registration', module: 'armature-metrics', status: 'high' },
      ],
    },
    {
      name: 'Audit & Compliance',
      items: [
        { feature: 'Audit Logging', description: 'Track who did what, when', module: 'armature-audit', status: 'high' },
        { feature: 'Request/Response Logging', description: 'Configurable payload logging', module: 'armature-audit', status: 'high' },
        { feature: 'Data Masking', description: 'Mask sensitive data in logs', module: 'armature-audit', status: 'medium' },
        { feature: 'Retention Policies', description: 'Auto-cleanup old audit logs', module: 'armature-audit', status: 'medium' },
      ],
    },
    {
      name: 'Resilience & Reliability',
      items: [
        { feature: 'Circuit Breaker', description: 'Prevent cascade failures', module: 'armature-resilience', status: 'critical' },
        { feature: 'Retry with Backoff', description: 'Configurable retry strategies', module: 'armature-resilience', status: 'high' },
        { feature: 'Bulkhead Pattern', description: 'Resource isolation', module: 'armature-resilience', status: 'high' },
        { feature: 'Timeout Policies', description: 'Timeout configuration per endpoint', module: 'armature-resilience', status: 'high' },
        { feature: 'Fallback Handlers', description: 'Graceful degradation', module: 'armature-resilience', status: 'medium' },
        { feature: 'Connection Draining', description: 'Wait for in-flight requests', module: 'armature-core', status: 'critical' },
        { feature: 'Shutdown Hooks', description: 'Custom cleanup on shutdown', module: 'armature-core', status: 'high' },
        { feature: 'Health Status Update', description: 'Mark unhealthy during shutdown', module: 'armature-core', status: 'high' },
        { feature: 'Lifecycle Hooks', description: 'OnApplicationShutdown', module: 'armature-core', status: 'completed' },
      ],
    },
    {
      name: 'Pagination & Filtering',
      items: [
        { feature: 'Pagination Helpers', description: 'Offset, cursor-based pagination', module: 'armature-core', status: 'critical' },
        { feature: 'Sorting Helpers', description: 'Multi-field sorting', module: 'armature-core', status: 'high' },
        { feature: 'Filtering Helpers', description: 'Query parameter filtering', module: 'armature-core', status: 'high' },
        { feature: 'Search Helpers', description: 'Full-text search integration', module: 'armature-core', status: 'medium' },
        { feature: 'Field Selection', description: 'Sparse fieldsets (GraphQL-like)', module: 'armature-core', status: 'medium' },
      ],
    },
    {
      name: 'File Handling',
      items: [
        { feature: 'Multipart Upload', description: 'File upload handling', module: 'armature-storage', status: 'critical' },
        { feature: 'File Validation', description: 'Type, size, extension validation', module: 'armature-storage', status: 'critical' },
        { feature: 'S3 Integration', description: 'AWS S3 file storage', module: 'armature-storage', status: 'high' },
        { feature: 'GCS Integration', description: 'Google Cloud Storage', module: 'armature-storage', status: 'high' },
        { feature: 'Azure Blob', description: 'Azure Blob Storage', module: 'armature-storage', status: 'high' },
        { feature: 'Local Storage', description: 'Filesystem storage with paths', module: 'armature-storage', status: 'medium' },
      ],
    },
    {
      name: 'Email',
      items: [
        { feature: 'SMTP Integration', description: 'Email sending via SMTP', module: 'armature-mail', status: 'critical' },
        { feature: 'Email Templates', description: 'HTML email with templates', module: 'armature-mail', status: 'high' },
        { feature: 'SendGrid Integration', description: 'SendGrid API support', module: 'armature-mail', status: 'high' },
        { feature: 'AWS SES Integration', description: 'AWS SES support', module: 'armature-mail', status: 'high' },
        { feature: 'Mailgun Integration', description: 'Mailgun API support', module: 'armature-mail', status: 'medium' },
        { feature: 'Email Queue', description: 'Async email sending', module: 'armature-mail', status: 'medium' },
      ],
    },
    {
      name: 'Messaging',
      items: [
        { feature: 'RabbitMQ Integration', description: 'RabbitMQ message broker', module: 'armature-messaging', status: 'completed' },
        { feature: 'Kafka Integration', description: 'Apache Kafka support', module: 'armature-messaging', status: 'completed' },
        { feature: 'NATS Integration', description: 'NATS messaging', module: 'armature-messaging', status: 'completed' },
        { feature: 'AWS SQS/SNS', description: 'AWS messaging services', module: 'armature-messaging', status: 'completed' },
        { feature: 'Job Queue', description: 'Redis-based job queue', module: 'armature-queue', status: 'completed' },
      ],
    },
    {
      name: 'External APIs',
      items: [
        { feature: 'HTTP Client', description: 'Built-in HTTP client with retry', module: 'armature-http-client', status: 'high' },
        { feature: 'gRPC Support', description: 'gRPC server and client', module: 'armature-grpc', status: 'medium' },
        { feature: 'GraphQL Client', description: 'GraphQL client for federation', module: 'armature-graphql-client', status: 'medium' },
      ],
    },
    {
      name: 'Authentication',
      items: [
        { feature: 'API Key Management', description: 'API key generation/rotation', module: 'armature-auth', status: 'high' },
        { feature: 'Two-Factor Auth (2FA)', description: 'TOTP/HOTP support', module: 'armature-auth', status: 'high' },
        { feature: 'Passwordless Auth', description: 'Magic links, WebAuthn', module: 'armature-auth', status: 'medium' },
        { feature: 'Social Auth Extensions', description: 'More OAuth providers', module: 'armature-auth', status: 'medium' },
        { feature: 'JWT Authentication', description: 'JWT token management', module: 'armature-jwt', status: 'completed' },
        { feature: 'OAuth2/OIDC', description: 'Google, Microsoft, etc.', module: 'armature-auth', status: 'completed' },
        { feature: 'SAML 2.0', description: 'Enterprise SSO', module: 'armature-auth', status: 'completed' },
      ],
    },
    {
      name: 'Security Headers & Protection',
      items: [
        { feature: 'CORS Improvements', description: 'More granular CORS control', module: 'armature-security', status: 'critical' },
        { feature: 'CSP Headers', description: 'Content Security Policy', module: 'armature-security', status: 'high' },
        { feature: 'HSTS', description: 'HTTP Strict Transport Security', module: 'armature-security', status: 'high' },
        { feature: 'Request Signing', description: 'HMAC request verification', module: 'armature-security', status: 'medium' },
        { feature: 'Security Headers', description: 'Basic security headers', module: 'armature-security', status: 'completed' },
        { feature: 'Rate Limiting', description: 'Token bucket, sliding window', module: 'armature-ratelimit', status: 'completed' },
      ],
    },
    {
      name: 'Multi-tenancy',
      items: [
        { feature: 'Tenant Isolation', description: 'Request-scoped tenant context', module: 'armature-tenancy', status: 'high' },
        { feature: 'Database per Tenant', description: 'Separate database connections', module: 'armature-tenancy', status: 'high' },
        { feature: 'Schema per Tenant', description: 'PostgreSQL schema isolation', module: 'armature-tenancy', status: 'high' },
        { feature: 'Tenant Middleware', description: 'Auto tenant resolution', module: 'armature-tenancy', status: 'medium' },
        { feature: 'Tenant-aware Caching', description: 'Cache key prefixing', module: 'armature-tenancy', status: 'medium' },
      ],
    },
    {
      name: 'Feature Flags',
      items: [
        { feature: 'Feature Flags', description: 'Toggle features at runtime', module: 'armature-features', status: 'high' },
        { feature: 'LaunchDarkly Integration', description: 'LaunchDarkly support', module: 'armature-features', status: 'high' },
        { feature: 'A/B Testing', description: 'Experiment framework', module: 'armature-features', status: 'medium' },
        { feature: 'Gradual Rollout', description: 'Percentage-based rollout', module: 'armature-features', status: 'medium' },
      ],
    },
    {
      name: 'Internationalization',
      items: [
        { feature: 'i18n Support', description: 'Message translation', module: 'armature-i18n', status: 'high' },
        { feature: 'Locale Detection', description: 'Accept-Language parsing', module: 'armature-i18n', status: 'high' },
        { feature: 'Pluralization', description: 'Plural rules support', module: 'armature-i18n', status: 'medium' },
        { feature: 'Date/Number Formatting', description: 'Locale-aware formatting', module: 'armature-i18n', status: 'medium' },
      ],
    },
    {
      name: 'CLI Improvements',
      items: [
        { feature: 'REPL', description: 'Interactive Rust REPL', module: 'armature-cli', status: 'high' },
        { feature: 'Route List', description: 'armature routes - list all routes', module: 'armature-cli', status: 'medium' },
        { feature: 'Config Validation', description: 'armature config:check', module: 'armature-cli', status: 'medium' },
        { feature: 'Code Generation', description: 'Controllers, services, modules', module: 'armature-cli', status: 'completed' },
        { feature: 'Project Templates', description: 'Starter templates', module: 'armature-cli', status: 'completed' },
        { feature: 'Dev Server', description: 'Hot reloading development', module: 'armature-cli', status: 'completed' },
      ],
    },
    {
      name: 'Documentation & Tooling',
      items: [
        { feature: 'API Playground', description: 'Interactive API testing UI', module: 'armature-openapi', status: 'high' },
        { feature: 'OpenAPI Generation', description: 'Swagger/OpenAPI docs', module: 'armature-openapi', status: 'completed' },
      ],
    },
    {
      name: 'Testing',
      items: [
        { feature: 'Integration Test Helpers', description: 'Database setup/teardown', module: 'armature-testing', status: 'high' },
        { feature: 'Test Containers', description: 'Docker-based testing', module: 'armature-testing', status: 'high' },
        { feature: 'Load Testing', description: 'Performance test utilities', module: 'armature-testing', status: 'medium' },
        { feature: 'Contract Testing', description: 'Pact/consumer-driven contracts', module: 'armature-testing', status: 'medium' },
        { feature: 'Unit Test Helpers', description: 'Mocks, spies, assertions', module: 'armature-testing', status: 'completed' },
      ],
    },
    {
      name: 'Event-Driven Architecture',
      items: [
        { feature: 'Event Bus', description: 'In-process event publishing', module: 'armature-events', status: 'high' },
        { feature: 'Event Handlers', description: 'Decorator-based event handling', module: 'armature-events', status: 'high' },
        { feature: 'Event Sourcing', description: 'Event-sourced aggregates', module: 'armature-eventsourcing', status: 'medium' },
        { feature: 'CQRS Support', description: 'Command/Query separation', module: 'armature-cqrs', status: 'medium' },
      ],
    },
    {
      name: 'Distributed Systems',
      items: [
        { feature: 'Distributed Locks', description: 'Redis-based distributed locks', module: 'armature-distributed', status: 'high' },
        { feature: 'Request Correlation', description: 'Correlation ID propagation', module: 'armature-core', status: 'completed' },
        { feature: 'Leader Election', description: 'Distributed leader election', module: 'armature-distributed', status: 'medium' },
        { feature: 'Service Discovery', description: 'Consul/etcd integration', module: 'armature-discovery', status: 'medium' },
      ],
    },
    {
      name: 'Caching Improvements',
      items: [
        { feature: 'Cache Decorators', description: '#[cache] method decorator', module: 'armature-cache', status: 'high' },
        { feature: 'Cache Invalidation', description: 'Tag-based invalidation', module: 'armature-cache', status: 'high' },
        { feature: 'Multi-tier Caching', description: 'L1/L2 cache layers', module: 'armature-cache', status: 'medium' },
        { feature: 'Redis Cache', description: 'Redis caching', module: 'armature-cache', status: 'completed' },
        { feature: 'Memcached Cache', description: 'Memcached caching', module: 'armature-cache', status: 'completed' },
      ],
    },
    {
      name: 'Containerization',
      items: [
        { feature: 'Dockerfile Templates', description: 'Optimized Dockerfiles', module: 'templates/', status: 'high' },
        { feature: 'Docker Compose', description: 'Development environment', module: 'templates/', status: 'high' },
        { feature: 'Kubernetes Manifests', description: 'K8s deployment templates', module: 'templates/', status: 'medium' },
        { feature: 'Helm Charts', description: 'Helm chart templates', module: 'templates/', status: 'medium' },
      ],
    },
    {
      name: 'CI/CD',
      items: [
        { feature: 'GitHub Actions', description: 'CI/CD workflow templates', module: '.github/', status: 'high' },
        { feature: 'Jenkins', description: 'Jenkinsfile templates', module: 'templates/', status: 'medium' },
      ],
    },
    {
      name: 'Cloud Providers',
      items: [
        { feature: 'AWS Lambda', description: 'Serverless deployment', module: 'armature-lambda', status: 'medium' },
        { feature: 'Google Cloud Run', description: 'GCR deployment', module: 'armature-cloudrun', status: 'medium' },
        { feature: 'Azure Functions', description: 'Azure serverless', module: 'armature-azure-functions', status: 'medium' },
      ],
    },
  ];

  completedFeatures = [
    'DI, Controllers, Modules, Routing, Middleware, Guards, Interceptors',
    'API Versioning, Content Negotiation, ETags/Conditional Requests',
    'Streaming Responses, Response Caching Headers',
    'Error Correlation, Request Correlation',
    'Health Checks (liveness/readiness/full)',
    'Request Timeout (#[timeout] decorator)',
    'Request Size Limits (#[body_limit] decorator)',
    'JWT, OAuth2 (Google, Microsoft, Cognito, Okta, Auth0), SAML 2.0',
    'Redis Cache, Memcached Cache, Session Storage',
    'Job Queues, Cron Jobs',
    'GraphQL, OpenAPI/Swagger, WebSocket, SSE, Webhooks',
    'Rate Limiting, HTTPS/TLS, ACME Certificates, Security Headers',
    'OpenTelemetry, Structured Logging',
    'CLI, Code Generation, Project Templates',
    'Compression, #[use_middleware], #[use_guard] decorators',
    'Test Utilities, Validation Framework',
  ];

  getStatusClass(status: string): string {
    switch (status) {
      case 'completed':
        return 'bg-emerald-100 text-emerald-800';
      case 'critical':
        return 'bg-red-100 text-red-800';
      case 'high':
        return 'bg-orange-100 text-orange-800';
      case 'medium':
        return 'bg-yellow-100 text-yellow-800';
      case 'low':
        return 'bg-green-100 text-green-800';
      default:
        return 'bg-gray-100 text-gray-800';
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
}

