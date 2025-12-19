import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { DocPageComponent, DocPage } from '../../shared/doc-page.component';

@Component({
  selector: 'app-logging-guide',
  standalone: true,
  imports: [CommonModule, DocPageComponent],
  template: `<app-doc-page [page]="page"></app-doc-page>`
})
export class LoggingGuideComponent {
  page: DocPage = {
    title: 'Structured Logging',
    subtitle: 'Production-ready logging with structured output, multiple formats, log levels, and integration with tracing for distributed systems.',
    icon: '📝',
    badge: 'Observability',
    features: [
      {
        icon: '🎯',
        title: 'Structured Output',
        description: 'JSON logs for machine parsing, pretty logs for humans'
      },
      {
        icon: '🎨',
        title: 'Multiple Formats',
        description: 'JSON, compact, full, and pretty output modes'
      },
      {
        icon: '⚡',
        title: 'Zero-Cost Abstraction',
        description: 'Compile-time log level filtering'
      },
      {
        icon: '🔗',
        title: 'Tracing Integration',
        description: 'Built on tracing for distributed tracing support'
      }
    ],
    sections: [
      {
        id: 'quick-start',
        title: 'Quick Start',
        content: `<p>Initialize logging with sensible defaults:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `use armature::logging::{LogConfig, LogLevel, LogFormat};
use tracing::{info, debug, warn, error};

#[tokio::main]
async fn main() {
    // Initialize with defaults (INFO level, pretty format)
    let _guard = LogConfig::default().init();

    info!("Application starting");
    debug!(user_id = 123, "User logged in");
    warn!(latency_ms = 500, "Slow query detected");
    error!(error = %err, "Database connection failed");
}`
          }
        ]
      },
      {
        id: 'configuration',
        title: 'Configuration',
        content: `<p>Customize logging behavior with <code>LogConfig</code>:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `use armature::logging::{LogConfig, LogLevel, LogFormat};

let config = LogConfig::new()
    .level(LogLevel::Debug)
    .format(LogFormat::Json)
    .with_file(true)           // Include file:line
    .with_target(true)         // Include module path
    .with_thread_ids(true)     // Include thread IDs
    .with_ansi(false);         // Disable colors (for log files)

let _guard = config.init();`
          }
        ],
        subsections: [
          {
            id: 'log-levels',
            title: 'Log Levels',
            content: `<p>From most to least verbose:</p>
            <ul>
              <li><code>Trace</code> — Very detailed debugging info</li>
              <li><code>Debug</code> — Debugging information</li>
              <li><code>Info</code> — General information (default)</li>
              <li><code>Warn</code> — Warnings that may need attention</li>
              <li><code>Error</code> — Errors that need immediate attention</li>
            </ul>`,
            codeBlocks: [
              {
                language: 'rust',
                code: `// Set level from environment variable
let level = match std::env::var("LOG_LEVEL").as_deref() {
    Ok("trace") => LogLevel::Trace,
    Ok("debug") => LogLevel::Debug,
    Ok("warn") => LogLevel::Warn,
    Ok("error") => LogLevel::Error,
    _ => LogLevel::Info,
};

LogConfig::new().level(level).init();`
              }
            ]
          },
          {
            id: 'log-formats',
            title: 'Log Formats',
            content: `<p>Choose the format that fits your needs:</p>`,
            codeBlocks: [
              {
                language: 'rust',
                code: `// Pretty format (colorized, human-readable) - for development
LogConfig::new().format(LogFormat::Pretty).init();
// Output: 2024-01-15T10:30:00Z INFO  server User logged in user_id=123

// JSON format - for production (machine-parseable)
LogConfig::new().format(LogFormat::Json).init();
// Output: {"timestamp":"2024-01-15T10:30:00Z","level":"INFO","target":"server","message":"User logged in","user_id":123}

// Compact format - minimal output
LogConfig::new().format(LogFormat::Compact).init();
// Output: INFO server: User logged in

// Full format - verbose with all metadata
LogConfig::new().format(LogFormat::Full).init();`
              }
            ]
          }
        ]
      },
      {
        id: 'structured-logging',
        title: 'Structured Logging',
        content: `<p>Add context to your logs with structured fields:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `use tracing::{info, warn, error, instrument, Span};

// Basic structured fields
info!(user_id = 123, action = "login", "User authentication");

// With nested structures
info!(
    user.id = 123,
    user.email = "alice@example.com",
    "User profile updated"
);

// Using Display formatting
warn!(error = %err, "Operation failed");

// Using Debug formatting
debug!(request = ?req, "Incoming request");

// Empty message with just fields
info!(metric_name = "requests_total", value = 42);`
          }
        ]
      },
      {
        id: 'spans',
        title: 'Spans & Tracing',
        content: `<p>Use spans to track operations across your application:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            filename: 'spans.rs',
            code: `use tracing::{info_span, instrument, Span};

// Method-level instrumentation
#[instrument(skip(self), fields(user_id = %id))]
async fn get_user(&self, id: u64) -> Result<User, Error> {
    // All logs inside will include the span context
    info!("Fetching user from database");

    let user = self.db.find_user(id).await?;

    info!("User found");
    Ok(user)
}

// Manual span creation
async fn process_order(order: Order) -> Result<(), Error> {
    let span = info_span!(
        "process_order",
        order_id = %order.id,
        customer_id = %order.customer_id
    );

    let _guard = span.enter();

    info!("Starting order processing");
    // ... processing logic
    info!("Order processed successfully");

    Ok(())
}`
          }
        ]
      },
      {
        id: 'request-logging',
        title: 'Request Logging',
        content: `<p>Automatically log all HTTP requests with timing:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `use armature::middleware::RequestLoggingMiddleware;

// Add request logging middleware
let app = Application::create::<AppModule>().await
    .middleware(RequestLoggingMiddleware::new());

// Output for each request:
// INFO  request method="GET" path="/api/users" status=200 latency_ms=12
// WARN  request method="POST" path="/api/orders" status=500 latency_ms=1523 error="Database timeout"`
          }
        ]
      },
      {
        id: 'error-logging',
        title: 'Error Logging',
        content: `<p>Log errors with full context and stack traces:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `use tracing::error;

// Log error with context
if let Err(e) = process_payment(order).await {
    error!(
        error = %e,
        error_chain = ?e.source(),
        order_id = %order.id,
        amount = %order.total,
        "Payment processing failed"
    );

    return Err(Error::PaymentFailed(e));
}

// Using anyhow for error chains
use anyhow::Context;

let user = db.find_user(id)
    .await
    .context("Failed to find user")?;`
          }
        ]
      },
      {
        id: 'best-practices',
        title: 'Best Practices',
        content: `<ul>
          <li><strong>Use structured fields</strong> — <code>info!(user_id = 123, ...)</code> not <code>info!("user_id: {}", 123)</code></li>
          <li><strong>Be consistent with field names</strong> — Use snake_case, same names for same concepts</li>
          <li><strong>Log at appropriate levels</strong> — Debug for dev, Info for production events</li>
          <li><strong>Include request IDs</strong> — Trace requests across services</li>
          <li><strong>Don't log sensitive data</strong> — Passwords, tokens, PII</li>
          <li><strong>Use JSON in production</strong> — Easier to parse and aggregate</li>
          <li><strong>Set up log rotation</strong> — Prevent disk space issues</li>
        </ul>`
      }
    ],
    relatedDocs: [
      {
        id: 'debug-logging',
        title: 'Debug Logging',
        description: 'Development-focused logging and debugging tools'
      },
      {
        id: 'opentelemetry-guide',
        title: 'OpenTelemetry',
        description: 'Distributed tracing and observability'
      },
      {
        id: 'error-correlation',
        title: 'Error Tracking',
        description: 'Correlate errors across requests and services'
      }
    ],
    seeAlso: [
      { title: 'Metrics Guide', id: 'metrics-guide' },
      { title: 'Health Checks', id: 'health-check' },
      { title: 'Audit Logging', id: 'audit-guide' }
    ]
  };
}

