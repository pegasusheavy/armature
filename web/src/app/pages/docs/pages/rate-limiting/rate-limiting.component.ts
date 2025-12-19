import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { DocPageComponent, DocPage } from '../../shared/doc-page.component';

@Component({
  selector: 'app-rate-limiting',
  standalone: true,
  imports: [CommonModule, DocPageComponent],
  template: `<app-doc-page [page]="page"></app-doc-page>`
})
export class RateLimitingComponent {
  page: DocPage = {
    title: 'Rate Limiting',
    subtitle: 'Protect your API from abuse with configurable rate limiting using token bucket and sliding window algorithms.',
    icon: '🚦',
    badge: 'Security',
    features: [
      { icon: '🪣', title: 'Token Bucket', description: 'Smooth traffic with burst allowance' },
      { icon: '📊', title: 'Sliding Window', description: 'Precise request counting' },
      { icon: '🔑', title: 'Per-Key Limits', description: 'Rate limit by IP, user, or API key' },
      { icon: '📡', title: 'Distributed', description: 'Redis-backed for multi-instance' }
    ],
    sections: [
      {
        id: 'basic-usage',
        title: 'Basic Rate Limiting',
        content: `<p>Add rate limiting to routes with the <code>#[rate_limit]</code> decorator:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `use armature::prelude::*;
use armature_ratelimit::*;

#[controller("/api")]
#[derive(Default, Clone)]
struct ApiController;

impl ApiController {
    #[get("/data")]
    #[rate_limit(requests = 100, window = "1m")]  // 100 requests per minute
    async fn get_data(&self) -> Json<Data> {
        ...
    }

    #[post("/submit")]
    #[rate_limit(requests = 10, window = "1h")]   // 10 requests per hour
    async fn submit(&self, body: Json<Submission>) -> StatusCode {
        ...
    }
}`
          }
        ]
      },
      {
        id: 'algorithms',
        title: 'Rate Limiting Algorithms',
        content: `<p>Choose the algorithm that fits your use case:</p>`,
        subsections: [
          {
            id: 'token-bucket',
            title: 'Token Bucket',
            content: `<p>Allows bursts while maintaining average rate:</p>`,
            codeBlocks: [
              {
                language: 'rust',
                code: `#[rate_limit(
    algorithm = "token_bucket",
    capacity = 100,        // Max tokens (burst size)
    refill_rate = 10,      // Tokens added per second
)]
async fn endpoint(&self) -> StatusCode {
    // Allows burst of 100 requests, then 10/second sustained
    ...
}`
              }
            ]
          },
          {
            id: 'sliding-window',
            title: 'Sliding Window',
            content: `<p>Counts requests in a rolling time window:</p>`,
            codeBlocks: [
              {
                language: 'rust',
                code: `#[rate_limit(
    algorithm = "sliding_window",
    requests = 100,
    window = "1m",
)]
async fn endpoint(&self) -> StatusCode {
    // Exactly 100 requests allowed in any 60-second window
    ...
}`
              }
            ]
          },
          {
            id: 'fixed-window',
            title: 'Fixed Window',
            content: `<p>Resets counter at fixed intervals:</p>`,
            codeBlocks: [
              {
                language: 'rust',
                code: `#[rate_limit(
    algorithm = "fixed_window",
    requests = 1000,
    window = "1h",
)]
async fn endpoint(&self) -> StatusCode {
    // 1000 requests per hour, resets on the hour
    ...
}`
              }
            ]
          }
        ]
      },
      {
        id: 'key-strategies',
        title: 'Rate Limit Keys',
        content: `<p>Rate limit by different identifiers:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `// By IP address (default)
#[rate_limit(requests = 100, window = "1m", key = "ip")]

// By authenticated user
#[rate_limit(requests = 1000, window = "1m", key = "user")]

// By API key
#[rate_limit(requests = 10000, window = "1m", key = "api_key")]

// Custom key extractor
#[rate_limit(requests = 100, window = "1m", key_fn = "extract_tenant_id")]

fn extract_tenant_id(req: &HttpRequest) -> Option<String> {
    req.header("X-Tenant-ID").map(|h| h.to_string())
}`
          }
        ]
      },
      {
        id: 'redis-backend',
        title: 'Distributed Rate Limiting',
        content: `<p>Use Redis for rate limiting across multiple instances:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `use armature_ratelimit::{RateLimiter, RedisBackend};

// Configure Redis-backed rate limiter
let rate_limiter = RateLimiter::builder()
    .backend(RedisBackend::new("redis://localhost:6379")?)
    .default_limit(100, Duration::from_secs(60))
    .build();

// Register with the application
Application::new()
    .configure(|app| {
        app.rate_limiter(rate_limiter);
    })
    .run()
    .await`
          }
        ]
      },
      {
        id: 'response-headers',
        title: 'Rate Limit Headers',
        content: `<p>Clients receive headers indicating their rate limit status:</p>`,
        codeBlocks: [
          {
            language: 'bash',
            code: `HTTP/1.1 200 OK
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1699574400
Retry-After: 45

# When rate limited (429 Too Many Requests):
HTTP/1.1 429 Too Many Requests
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1699574400
Retry-After: 45`
          }
        ]
      },
      {
        id: 'tiered-limits',
        title: 'Tiered Rate Limits',
        content: `<p>Different limits for different user tiers:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `#[rate_limit(
    tiers = [
        { role = "free", requests = 100, window = "1h" },
        { role = "pro", requests = 1000, window = "1h" },
        { role = "enterprise", requests = 10000, window = "1h" },
    ],
    tier_fn = "get_user_tier"
)]
async fn api_endpoint(&self) -> Json<Data> { ... }

fn get_user_tier(req: &HttpRequest) -> &str {
    req.extensions()
        .get::<User>()
        .map(|u| u.subscription_tier.as_str())
        .unwrap_or("free")
}`
          }
        ]
      },
      {
        id: 'best-practices',
        title: 'Best Practices',
        content: `<ul>
          <li><strong>Start conservative</strong> — You can always increase limits</li>
          <li><strong>Use Redis in production</strong> — In-memory doesn't work with multiple instances</li>
          <li><strong>Return informative headers</strong> — Help clients implement backoff</li>
          <li><strong>Consider user tiers</strong> — Paying customers expect higher limits</li>
          <li><strong>Log rate limit events</strong> — Detect abuse patterns</li>
          <li><strong>Exempt health checks</strong> — Don't rate limit monitoring endpoints</li>
        </ul>`
      }
    ],
    relatedDocs: [
      { id: 'redis-guide', title: 'Redis', description: 'Distributed rate limiting backend' },
      { id: 'security-guide', title: 'Security', description: 'Other security measures' }
    ],
    seeAlso: [
      { title: 'Authentication', id: 'auth-guide' },
      { title: 'API Versioning', id: 'api-versioning' }
    ]
  };
}

