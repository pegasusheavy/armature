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
        { feature: 'Global Exception Filters', description: 'Centralized error transformation', module: 'armature-core', status: 'critical' },
        { feature: 'Problem Details (RFC 7807)', description: 'Standardized error response format', module: 'armature-core', status: 'high' },
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
      ],
    },
    {
      name: 'Resilience & Reliability',
      items: [
        { feature: 'Circuit Breaker', description: 'Prevent cascade failures', module: 'armature-resilience', status: 'critical' },
        { feature: 'Retry with Backoff', description: 'Configurable retry strategies', module: 'armature-resilience', status: 'high' },
        { feature: 'Bulkhead Pattern', description: 'Resource isolation', module: 'armature-resilience', status: 'high' },
        { feature: 'Connection Draining', description: 'Wait for in-flight requests', module: 'armature-core', status: 'critical' },
        { feature: 'Shutdown Hooks', description: 'Custom cleanup on shutdown', module: 'armature-core', status: 'high' },
        { feature: 'Lifecycle Hooks', description: 'OnApplicationShutdown', module: 'armature-core', status: 'completed' },
      ],
    },
    {
      name: 'API Features',
      items: [
        { feature: 'Pagination Helpers', description: 'Offset, cursor-based pagination', module: 'armature-core', status: 'critical' },
        { feature: 'Sorting Helpers', description: 'Multi-field sorting', module: 'armature-core', status: 'high' },
        { feature: 'Filtering Helpers', description: 'Query parameter filtering', module: 'armature-core', status: 'high' },
        { feature: 'Multipart Upload', description: 'File upload handling', module: 'armature-storage', status: 'critical' },
        { feature: 'S3 Integration', description: 'AWS S3 file storage', module: 'armature-storage', status: 'high' },
        { feature: 'GCS Integration', description: 'Google Cloud Storage', module: 'armature-storage', status: 'high' },
      ],
    },
    {
      name: 'Communication & Integration',
      items: [
        { feature: 'SMTP Integration', description: 'Email sending via SMTP', module: 'armature-mail', status: 'critical' },
        { feature: 'Email Templates', description: 'HTML email with templates', module: 'armature-mail', status: 'high' },
        { feature: 'SendGrid Integration', description: 'SendGrid API support', module: 'armature-mail', status: 'high' },
        { feature: 'RabbitMQ Integration', description: 'RabbitMQ message broker', module: 'armature-rabbitmq', status: 'high' },
        { feature: 'Kafka Integration', description: 'Apache Kafka support', module: 'armature-kafka', status: 'high' },
        { feature: 'HTTP Client', description: 'Built-in HTTP client with retry', module: 'armature-http-client', status: 'high' },
        { feature: 'Job Queue', description: 'Redis-based job queue', module: 'armature-queue', status: 'completed' },
      ],
    },
    {
      name: 'Security',
      items: [
        { feature: 'API Key Management', description: 'API key generation/rotation', module: 'armature-auth', status: 'high' },
        { feature: 'Two-Factor Auth (2FA)', description: 'TOTP/HOTP support', module: 'armature-auth', status: 'high' },
        { feature: 'JWT Authentication', description: 'JWT token management', module: 'armature-jwt', status: 'completed' },
        { feature: 'OAuth2/OIDC', description: 'Google, Microsoft, etc.', module: 'armature-auth', status: 'completed' },
        { feature: 'CORS Improvements', description: 'More granular CORS control', module: 'armature-security', status: 'critical' },
        { feature: 'Security Headers', description: 'Basic security headers', module: 'armature-security', status: 'completed' },
        { feature: 'Rate Limiting', description: 'Token bucket, sliding window', module: 'armature-ratelimit', status: 'completed' },
      ],
    },
    {
      name: 'Enterprise Features',
      items: [
        { feature: 'Tenant Isolation', description: 'Request-scoped tenant context', module: 'armature-tenancy', status: 'high' },
        { feature: 'Feature Flags', description: 'Toggle features at runtime', module: 'armature-features', status: 'high' },
        { feature: 'i18n Support', description: 'Message translation', module: 'armature-i18n', status: 'high' },
        { feature: 'Audit Logging', description: 'Track who did what, when', module: 'armature-audit', status: 'high' },
      ],
    },
    {
      name: 'Developer Experience',
      items: [
        { feature: 'Code Generation', description: 'Controllers, services, modules', module: 'armature-cli', status: 'completed' },
        { feature: 'Project Templates', description: 'Starter templates', module: 'armature-cli', status: 'completed' },
        { feature: 'Dev Server', description: 'Hot reloading development', module: 'armature-cli', status: 'completed' },
        { feature: 'OpenAPI Generation', description: 'Swagger/OpenAPI docs', module: 'armature-openapi', status: 'completed' },
        { feature: 'API Playground', description: 'Interactive API testing UI', module: 'armature-openapi', status: 'high' },
        { feature: 'Route List', description: 'armature routes - list all routes', module: 'armature-cli', status: 'medium' },
        { feature: 'Unit Test Helpers', description: 'Mocks, spies, assertions', module: 'armature-testing', status: 'completed' },
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

