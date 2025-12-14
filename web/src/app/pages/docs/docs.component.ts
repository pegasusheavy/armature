import { Component, OnInit, signal, effect } from '@angular/core';
import { CommonModule } from '@angular/common';
import { ActivatedRoute, Router, RouterModule } from '@angular/router';
import { HttpClient } from '@angular/common/http';
import { marked } from 'marked';
import { DomSanitizer, SafeHtml } from '@angular/platform-browser';

interface DocMetadata {
  id: string;
  title: string;
  filename: string;
  category: string;
}

@Component({
  selector: 'app-docs',
  standalone: true,
  imports: [CommonModule, RouterModule],
  templateUrl: './docs.component.html',
  styleUrls: ['./docs.component.scss'],
})
export class DocsComponent implements OnInit {
  content = signal<SafeHtml>('');
  loading = signal(true);
  error = signal<string | null>(null);
  currentDoc = signal<DocMetadata | null>(null);

  // Documentation metadata - all lowercase with hyphens
  docs: DocMetadata[] = [
    // Getting Started
    { id: 'readme', title: 'Documentation Index', filename: 'README.md', category: 'Getting Started' },
    { id: 'di-guide', title: 'Dependency Injection', filename: 'di-guide.md', category: 'Getting Started' },
    { id: 'config-guide', title: 'Configuration', filename: 'config-guide.md', category: 'Getting Started' },
    { id: 'project-templates', title: 'Project Templates', filename: 'project-templates.md', category: 'Getting Started' },
    { id: 'lifecycle-hooks', title: 'Lifecycle Hooks', filename: 'lifecycle-hooks.md', category: 'Getting Started' },
    { id: 'macro-overview', title: 'Macro Overview', filename: 'macro-overview.md', category: 'Getting Started' },

    // Authentication & Security
    { id: 'auth-guide', title: 'Authentication', filename: 'auth-guide.md', category: 'Authentication & Security' },
    { id: 'oauth2-providers', title: 'OAuth2 Providers', filename: 'oauth2-providers-guide.md', category: 'Authentication & Security' },
    { id: 'security-guide', title: 'Security Guide', filename: 'security-guide.md', category: 'Authentication & Security' },
    { id: 'session-guide', title: 'Session Management', filename: 'session-guide.md', category: 'Authentication & Security' },
    { id: 'rate-limiting', title: 'Rate Limiting', filename: 'rate-limiting-guide.md', category: 'Authentication & Security' },

    // Core Features
    { id: 'guards-interceptors', title: 'Guards & Interceptors', filename: 'guards-interceptors.md', category: 'Core Features' },
    { id: 'use-guard', title: '#[use_guard] Decorator', filename: 'use-guard-guide.md', category: 'Core Features' },
    { id: 'use-middleware', title: '#[use_middleware] Decorator', filename: 'use-middleware-guide.md', category: 'Core Features' },
    { id: 'request-extractors', title: 'Request Extractors', filename: 'request-extractors.md', category: 'Core Features' },

    // Routing
    { id: 'route-groups', title: 'Route Groups', filename: 'guides/route-groups-guide.md', category: 'Routing' },
    { id: 'route-constraints', title: 'Route Constraints', filename: 'guides/route-constraints-guide.md', category: 'Routing' },

    // HTTP & Networking
    { id: 'https-guide', title: 'HTTPS Setup', filename: 'https-guide.md', category: 'HTTP & Networking' },
    { id: 'acme-certificates', title: 'ACME Certificates', filename: 'acme-certificates.md', category: 'HTTP & Networking' },
    { id: 'compression', title: 'Compression', filename: 'compression.md', category: 'HTTP & Networking' },
    { id: 'http-status-errors', title: 'HTTP Status & Errors', filename: 'http-status-errors.md', category: 'HTTP & Networking' },
    { id: 'websocket-sse', title: 'WebSocket & SSE', filename: 'websocket-sse-guide.md', category: 'HTTP & Networking' },
    { id: 'webhooks', title: 'Webhooks', filename: 'webhooks.md', category: 'HTTP & Networking' },
    { id: 'request-timeouts', title: 'Request Timeouts', filename: 'request-timeouts-guide.md', category: 'HTTP & Networking' },
    { id: 'streaming-responses', title: 'Streaming Responses', filename: 'streaming-responses-guide.md', category: 'HTTP & Networking' },

    // API Features
    { id: 'api-versioning', title: 'API Versioning', filename: 'api-versioning-guide.md', category: 'API Features' },
    { id: 'content-negotiation', title: 'Content Negotiation', filename: 'content-negotiation-guide.md', category: 'API Features' },
    { id: 'response-caching', title: 'Response Caching', filename: 'response-caching-guide.md', category: 'API Features' },
    { id: 'etag-conditional', title: 'ETags & Conditional Requests', filename: 'etag-conditional-requests-guide.md', category: 'API Features' },
    { id: 'pagination-filtering', title: 'Pagination & Filtering', filename: 'guides/pagination-filtering-guide.md', category: 'API Features' },

    // GraphQL
    { id: 'graphql-guide', title: 'GraphQL Guide', filename: 'graphql-guide.md', category: 'GraphQL' },
    { id: 'graphql-config', title: 'GraphQL Configuration', filename: 'graphql-configuration.md', category: 'GraphQL' },

    // OpenAPI
    { id: 'openapi-guide', title: 'OpenAPI Guide', filename: 'openapi-guide.md', category: 'OpenAPI' },

    // Background Processing
    { id: 'queue-guide', title: 'Job Queues', filename: 'queue-guide.md', category: 'Background Processing' },
    { id: 'cron-guide', title: 'Cron Jobs', filename: 'cron-guide.md', category: 'Background Processing' },
    { id: 'graceful-shutdown', title: 'Graceful Shutdown', filename: 'guides/graceful-shutdown-guide.md', category: 'Background Processing' },

    // Observability
    { id: 'logging-guide', title: 'Logging', filename: 'logging-guide.md', category: 'Observability' },
    { id: 'debug-logging', title: 'Debug Logging', filename: 'debug-logging-guide.md', category: 'Observability' },
    { id: 'opentelemetry-guide', title: 'OpenTelemetry', filename: 'opentelemetry-guide.md', category: 'Observability' },
    { id: 'health-check', title: 'Health Checks', filename: 'health-check-guide.md', category: 'Observability' },
    { id: 'error-correlation', title: 'Error Correlation', filename: 'error-correlation-guide.md', category: 'Observability' },
    { id: 'metrics-guide', title: 'Prometheus Metrics', filename: 'guides/metrics-guide.md', category: 'Observability' },
    { id: 'audit-guide', title: 'Audit Logging', filename: 'guides/audit-guide.md', category: 'Observability' },

    // Caching
    { id: 'cache-improvements', title: 'Cache Improvements', filename: 'guides/cache-improvements-guide.md', category: 'Caching' },

    // Architecture
    { id: 'stateless-architecture', title: 'Stateless Architecture', filename: 'stateless-architecture.md', category: 'Architecture' },
    { id: 'server-integration', title: 'Server Integration', filename: 'server-integration.md', category: 'Architecture' },

    // Advanced Security
    { id: 'security-advanced', title: 'Advanced Security', filename: 'guides/security-advanced-guide.md', category: 'Advanced Features' },

    // Macros
    { id: 'macros-guide', title: 'Macros In-Depth', filename: 'guides/macros-guide.md', category: 'Macros' },

    // Error Handling
    { id: 'error-transformation', title: 'Error Transformation', filename: 'error-transformation-guide.md', category: 'Error Handling' },

    // Testing & Quality
    { id: 'testing-guide', title: 'Testing Utilities', filename: 'guides/testing-guide.md', category: 'Testing & Quality' },
    { id: 'testing-coverage', title: 'Testing Coverage', filename: 'testing-coverage.md', category: 'Testing & Quality' },
    { id: 'testing-documentation', title: 'Testing Documentation', filename: 'testing-documentation.md', category: 'Testing & Quality' },
    { id: 'documentation-testing', title: 'Documentation Testing', filename: 'documentation-testing.md', category: 'Testing & Quality' },

    // Benchmarks
    { id: 'armature-vs-nodejs', title: 'Armature vs Node.js', filename: 'guides/armature-vs-nodejs-benchmark.md', category: 'Benchmarks' },
    { id: 'armature-vs-nextjs', title: 'Armature vs Next.js', filename: 'guides/armature-vs-nextjs-benchmark.md', category: 'Benchmarks' },
  ];

  // Group docs by category
  docsByCategory = signal<{ [key: string]: DocMetadata[] }>({});

  constructor(
    private route: ActivatedRoute,
    private router: Router,
    private http: HttpClient,
    private sanitizer: DomSanitizer
  ) {
    // Configure marked options
    marked.setOptions({
      gfm: true,
      breaks: true,
    });

    // Group docs by category
    const grouped: { [key: string]: DocMetadata[] } = {};
    this.docs.forEach((doc) => {
      if (!grouped[doc.category]) {
        grouped[doc.category] = [];
      }
      grouped[doc.category].push(doc);
    });
    this.docsByCategory.set(grouped);

    // Watch for route changes
    effect(() => {
      const docId = this.route.snapshot.paramMap.get('id');
      if (docId) {
        this.loadDoc(docId);
      }
    });
  }

  ngOnInit() {
    this.route.paramMap.subscribe((params) => {
      const docId = params.get('id');
      if (docId) {
        this.loadDoc(docId);
      } else {
        // Default to README
        this.router.navigate(['/docs/readme']);
      }
    });
  }

  async loadDoc(docId: string) {
    this.loading.set(true);
    this.error.set(null);

    const doc = this.docs.find((d) => d.id === docId);
    if (!doc) {
      this.error.set('Documentation not found');
      this.loading.set(false);
      return;
    }

    this.currentDoc.set(doc);

    try {
      const markdown = await this.http
        .get(`docs/${doc.filename}`, { responseType: 'text' })
        .toPromise();

      if (markdown) {
        const html = await marked.parse(markdown);
        this.content.set(this.sanitizer.bypassSecurityTrustHtml(html as string));
      }
    } catch (err) {
      this.error.set('Failed to load documentation');
      console.error('Error loading doc:', err);
    } finally {
      this.loading.set(false);
    }
  }

  getCategories(): string[] {
    return Object.keys(this.docsByCategory());
  }
}
