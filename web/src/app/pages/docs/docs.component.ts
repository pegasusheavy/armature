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

  // Documentation metadata
  docs: DocMetadata[] = [
    // Getting Started
    { id: 'readme', title: 'Getting Started', filename: 'README.md', category: 'Getting Started' },
    {
      id: 'di-guide',
      title: 'Dependency Injection',
      filename: 'DI_GUIDE.md',
      category: 'Getting Started',
    },
    {
      id: 'config-guide',
      title: 'Configuration',
      filename: 'CONFIG_GUIDE.md',
      category: 'Getting Started',
    },

    // Core Features
    {
      id: 'auth-guide',
      title: 'Authentication',
      filename: 'AUTH_GUIDE.md',
      category: 'Core Features',
    },
    {
      id: 'oauth2-guide',
      title: 'OAuth2 Providers',
      filename: 'OAUTH2_PROVIDERS_GUIDE.md',
      category: 'Core Features',
    },
    {
      id: 'guards-interceptors',
      title: 'Guards & Interceptors',
      filename: 'GUARDS_INTERCEPTORS.md',
      category: 'Core Features',
    },
    {
      id: 'csrf-protection',
      title: 'CSRF Protection',
      filename: 'CSRF_PROTECTION.md',
      category: 'Core Features',
    },
    {
      id: 'lifecycle-hooks',
      title: 'Lifecycle Hooks',
      filename: 'LIFECYCLE_HOOKS.md',
      category: 'Core Features',
    },

    // HTTP & Networking
    {
      id: 'https-guide',
      title: 'HTTPS Setup',
      filename: 'HTTPS_GUIDE.md',
      category: 'HTTP & Networking',
    },
    {
      id: 'acme-certificates',
      title: 'ACME Certificates',
      filename: 'ACME_CERTIFICATES.md',
      category: 'HTTP & Networking',
    },
    {
      id: 'http-status-errors',
      title: 'HTTP Status & Errors',
      filename: 'HTTP_STATUS_ERRORS.md',
      category: 'HTTP & Networking',
    },
    {
      id: 'websocket-sse',
      title: 'WebSocket & SSE',
      filename: 'WEBSOCKET_SSE_GUIDE.md',
      category: 'HTTP & Networking',
    },

    // SSR Frameworks
    {
      id: 'angular-ssr',
      title: 'Angular SSR',
      filename: 'ANGULAR_SSR_GUIDE.md',
      category: 'SSR Frameworks',
    },
    { id: 'vue-ssr', title: 'Vue SSR', filename: 'VUE_SSR_GUIDE.md', category: 'SSR Frameworks' },
    {
      id: 'hmr-guide',
      title: 'Hot Module Reload',
      filename: 'HMR_GUIDE.md',
      category: 'SSR Frameworks',
    },
    {
      id: 'hmr-quick-start',
      title: 'HMR Quick Start',
      filename: 'HMR_QUICK_START.md',
      category: 'SSR Frameworks',
    },

    // GraphQL
    {
      id: 'graphql-guide',
      title: 'GraphQL Guide',
      filename: 'GRAPHQL_GUIDE.md',
      category: 'GraphQL',
    },
    {
      id: 'graphql-config',
      title: 'GraphQL Configuration',
      filename: 'GRAPHQL_CONFIGURATION.md',
      category: 'GraphQL',
    },

    // OpenAPI
    {
      id: 'openapi-guide',
      title: 'OpenAPI Guide',
      filename: 'OPENAPI_GUIDE.md',
      category: 'OpenAPI',
    },

    // Background Processing
    {
      id: 'queue-guide',
      title: 'Job Queues',
      filename: 'QUEUE_GUIDE.md',
      category: 'Background Processing',
    },
    {
      id: 'cron-guide',
      title: 'Cron Jobs',
      filename: 'CRON_GUIDE.md',
      category: 'Background Processing',
    },

    // Observability
    {
      id: 'logging-guide',
      title: 'Logging',
      filename: 'LOGGING_GUIDE.md',
      category: 'Observability',
    },
    {
      id: 'debug-logging',
      title: 'Debug Logging',
      filename: 'DEBUG_LOGGING_GUIDE.md',
      category: 'Observability',
    },
    {
      id: 'opentelemetry-guide',
      title: 'OpenTelemetry',
      filename: 'OPENTELEMETRY_GUIDE.md',
      category: 'Observability',
    },

    // Architecture
    {
      id: 'stateless-architecture',
      title: 'Stateless Architecture',
      filename: 'STATELESS_ARCHITECTURE.md',
      category: 'Architecture',
    },
    {
      id: 'server-integration',
      title: 'Server Integration',
      filename: 'SERVER_INTEGRATION.md',
      category: 'Architecture',
    },
    {
      id: 'parallel-processing',
      title: 'Parallel Processing',
      filename: 'PARALLEL_PROCESSING_GUIDE.md',
      category: 'Architecture',
    },
    {
      id: 'multithreading-opportunities',
      title: 'Multithreading',
      filename: 'MULTITHREADING_OPPORTUNITIES.md',
      category: 'Architecture',
    },

    // Testing & Quality
    {
      id: 'testing-coverage',
      title: 'Testing Coverage',
      filename: 'TESTING_COVERAGE.md',
      category: 'Testing & Quality',
    },
    {
      id: 'testing-documentation',
      title: 'Testing Documentation',
      filename: 'TESTING_DOCUMENTATION.md',
      category: 'Testing & Quality',
    },
    {
      id: 'doc-test-coverage',
      title: 'Doc Test Coverage',
      filename: 'DOC_TEST_COVERAGE_STATUS.md',
      category: 'Testing & Quality',
    },
    {
      id: 'documentation-testing',
      title: 'Documentation Testing',
      filename: 'DOCUMENTATION_TESTING.md',
      category: 'Testing & Quality',
    },
    {
      id: 'documentation-testing-report',
      title: 'Testing Final Report',
      filename: 'DOCUMENTATION_TESTING_FINAL_REPORT.md',
      category: 'Testing & Quality',
    },

    // Security
    {
      id: 'security-guide',
      title: 'Security Guide',
      filename: 'SECURITY_GUIDE.md',
      category: 'Security',
    },
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
        .get(`/docs/${doc.filename}`, { responseType: 'text' })
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
