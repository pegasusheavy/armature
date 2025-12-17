import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { DocPageComponent, DocPage } from '../../shared/doc-page.component';

@Component({
  selector: 'app-redis-guide',
  standalone: true,
  imports: [CommonModule, DocPageComponent],
  template: `<app-doc-page [page]="page"></app-doc-page>`
})
export class RedisGuideComponent {
  page: DocPage = {
    title: 'Redis Integration',
    subtitle: 'Centralized Redis client with connection pooling, pub/sub, cluster support, and shared usage across all Armature features.',
    icon: '🔴',
    badge: 'Data',
    features: [
      { icon: '🏊', title: 'Connection Pool', description: 'Efficient connection management' },
      { icon: '📡', title: 'Pub/Sub', description: 'Real-time messaging' },
      { icon: '🔒', title: 'TLS Support', description: 'Secure connections' },
      { icon: '🌐', title: 'Cluster Mode', description: 'High availability setup' }
    ],
    sections: [
      {
        id: 'setup',
        title: 'Basic Setup',
        content: `<p>Add Redis to your project and configure the connection:</p>`,
        codeBlocks: [
          {
            language: 'toml',
            filename: 'Cargo.toml',
            code: `[dependencies]
armature = { version = "0.1", features = ["redis"] }`
          },
          {
            language: 'rust',
            filename: 'main.rs',
            code: `use armature::prelude::*;
use armature_redis::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create Redis client
    let redis = RedisClient::builder()
        .url("redis://localhost:6379")
        .pool_size(10)
        .build()
        .await?;

    Application::new()
        .register(redis)
        .run()
        .await
}`
          }
        ]
      },
      {
        id: 'basic-operations',
        title: 'Basic Operations',
        content: `<p>Perform common Redis operations:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `#[injectable]
#[derive(Clone)]
pub struct MyService {
    redis: RedisClient,
}

impl MyService {
    // String operations
    pub async fn cache_value(&self, key: &str, value: &str) -> Result<()> {
        self.redis.set(key, value).await?;
        Ok(())
    }

    pub async fn get_value(&self, key: &str) -> Result<Option<String>> {
        self.redis.get(key).await
    }

    // With expiration
    pub async fn cache_with_ttl(&self, key: &str, value: &str, ttl: Duration) -> Result<()> {
        self.redis.set_ex(key, value, ttl).await?;
        Ok(())
    }

    // Hash operations
    pub async fn store_user(&self, user_id: &str, user: &User) -> Result<()> {
        self.redis.hset_multiple(
            format!("user:{}", user_id),
            &[
                ("name", &user.name),
                ("email", &user.email),
            ]
        ).await?;
        Ok(())
    }
}`
          }
        ]
      },
      {
        id: 'pub-sub',
        title: 'Pub/Sub Messaging',
        content: `<p>Use Redis pub/sub for real-time messaging:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `// Publisher
#[injectable]
pub struct NotificationService {
    redis: RedisClient,
}

impl NotificationService {
    pub async fn broadcast(&self, channel: &str, message: &str) -> Result<()> {
        self.redis.publish(channel, message).await?;
        Ok(())
    }
}

// Subscriber
pub async fn start_subscriber(redis: RedisClient) {
    let mut pubsub = redis.subscribe(&["notifications", "alerts"]).await.unwrap();

    while let Some(msg) = pubsub.next().await {
        match msg {
            Ok(message) => {
                println!("Channel: {}, Message: {}",
                    message.channel,
                    message.payload
                );
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}`
          }
        ]
      },
      {
        id: 'cluster',
        title: 'Cluster Configuration',
        content: `<p>Connect to a Redis cluster for high availability:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `let redis = RedisClient::builder()
    .cluster(&[
        "redis://node1:6379",
        "redis://node2:6379",
        "redis://node3:6379",
    ])
    .pool_size(10)
    .build()
    .await?;`
          }
        ]
      },
      {
        id: 'sentinel',
        title: 'Sentinel Configuration',
        content: `<p>Use Redis Sentinel for automatic failover:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `let redis = RedisClient::builder()
    .sentinel(&[
        "redis://sentinel1:26379",
        "redis://sentinel2:26379",
        "redis://sentinel3:26379",
    ])
    .master_name("mymaster")
    .pool_size(10)
    .build()
    .await?;`
          }
        ]
      },
      {
        id: 'tls',
        title: 'TLS/SSL Connections',
        content: `<p>Secure your Redis connections with TLS:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `let redis = RedisClient::builder()
    .url("rediss://localhost:6379")  // Note: rediss:// for TLS
    .tls(TlsConfig {
        ca_cert: Some("/path/to/ca.crt"),
        client_cert: Some("/path/to/client.crt"),
        client_key: Some("/path/to/client.key"),
        verify_hostname: true,
    })
    .build()
    .await?;`
          }
        ]
      },
      {
        id: 'shared-usage',
        title: 'Shared Across Features',
        content: `<p>The Redis client is shared across all Armature features:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `// All these features use the same Redis connection pool:

// Caching
let cache = Cache::new(redis.clone());

// Sessions
let sessions = SessionStore::new(redis.clone());

// Rate limiting
let rate_limiter = RateLimiter::new(redis.clone());

// Job queues
let queue = JobQueue::new(redis.clone());

// Distributed locks
let lock = DistributedLock::new(redis.clone());`
          }
        ]
      },
      {
        id: 'best-practices',
        title: 'Best Practices',
        content: `<ul>
          <li><strong>Use connection pooling</strong> — Don't create new connections per request</li>
          <li><strong>Set appropriate TTLs</strong> — Prevent memory bloat</li>
          <li><strong>Use pipelines for bulk ops</strong> — Reduce round trips</li>
          <li><strong>Monitor memory usage</strong> — Set maxmemory policy</li>
          <li><strong>Use TLS in production</strong> — Encrypt data in transit</li>
          <li><strong>Implement retry logic</strong> — Handle transient failures</li>
        </ul>`
      }
    ],
    relatedDocs: [
      { id: 'cache-improvements', title: 'Caching', description: 'Redis-backed caching' },
      { id: 'queue-guide', title: 'Job Queues', description: 'Background job processing' }
    ],
    seeAlso: [
      { title: 'Session Management', id: 'session-guide' },
      { title: 'Rate Limiting', id: 'rate-limiting' }
    ]
  };
}

