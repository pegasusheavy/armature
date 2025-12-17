import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { DocPageComponent, DocPage } from '../../shared/doc-page.component';

@Component({
  selector: 'app-cache-guide',
  standalone: true,
  imports: [CommonModule, DocPageComponent],
  template: `<app-doc-page [page]="page"></app-doc-page>`
})
export class CacheGuideComponent {
  page: DocPage = {
    title: 'Caching Strategies',
    subtitle: 'Boost performance with intelligent caching. Support for in-memory, Redis, multi-tier caching, and tag-based invalidation.',
    icon: '🚀',
    badge: 'Performance',
    features: [
      {
        icon: '💾',
        title: 'Multiple Backends',
        description: 'In-memory, Redis, or custom backends'
      },
      {
        icon: '🏗️',
        title: 'Multi-Tier Caching',
        description: 'L1/L2 cache hierarchy for optimal performance'
      },
      {
        icon: '🏷️',
        title: 'Tag-Based Invalidation',
        description: 'Invalidate related entries with tags'
      },
      {
        icon: '⏱️',
        title: 'TTL & Stale-While-Revalidate',
        description: 'Flexible expiration strategies'
      }
    ],
    sections: [
      {
        id: 'installation',
        title: 'Installation',
        content: `<p>Enable caching features in your <code>Cargo.toml</code>:</p>`,
        codeBlocks: [
          {
            language: 'toml',
            filename: 'Cargo.toml',
            code: `[dependencies]
armature = { version = "0.1", features = ["cache"] }

# For Redis support
armature = { version = "0.1", features = ["cache", "redis"] }`
          }
        ]
      },
      {
        id: 'basic-caching',
        title: 'Basic Caching',
        content: `<p>Start with simple in-memory caching:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            filename: 'basic_cache.rs',
            code: `use armature_cache::{Cache, InMemoryCache};
use std::time::Duration;

// Create an in-memory cache
let cache = InMemoryCache::new();

// Store a value with TTL
cache.set("user:123", user, Duration::from_secs(300)).await?;

// Retrieve value
if let Some(user) = cache.get::<User>("user:123").await? {
    println!("Found user: {:?}", user);
}

// Delete value
cache.delete("user:123").await?;

// Check if key exists
if cache.exists("user:123").await? {
    println!("Key exists");
}`
          }
        ]
      },
      {
        id: 'cache-patterns',
        title: 'Caching Patterns',
        subsections: [
          {
            id: 'cache-aside',
            title: 'Cache-Aside Pattern',
            content: `<p>The most common pattern: check cache first, load from source if missing:</p>`,
            codeBlocks: [
              {
                language: 'rust',
                code: `async fn get_user(cache: &Cache, db: &Database, id: &str) -> Result<User, Error> {
    let cache_key = format!("user:{}", id);

    // Try cache first
    if let Some(user) = cache.get::<User>(&cache_key).await? {
        return Ok(user);
    }

    // Load from database
    let user = db.find_user(id).await?;

    // Store in cache for future requests
    cache.set(&cache_key, &user, Duration::from_secs(300)).await?;

    Ok(user)
}`
              }
            ]
          },
          {
            id: 'get-or-set',
            title: 'Get-Or-Set Pattern',
            content: `<p>Automatically populate cache if key doesn't exist:</p>`,
            codeBlocks: [
              {
                language: 'rust',
                code: `// Get value or compute and cache it
let user = cache.get_or_set(
    "user:123",
    Duration::from_secs(300),
    || async {
        db.find_user("123").await
    }
).await?;`
              }
            ]
          },
          {
            id: 'stale-while-revalidate',
            title: 'Stale-While-Revalidate',
            content: `<p>Return stale data immediately while refreshing in the background:</p>`,
            codeBlocks: [
              {
                language: 'rust',
                code: `use armature_cache::StaleWhileRevalidate;

let swr = StaleWhileRevalidate::new(cache)
    .ttl(Duration::from_secs(60))         // Fresh for 60s
    .stale_ttl(Duration::from_secs(300)); // Serve stale up to 5min

// Returns immediately (stale if needed), refreshes in background
let user = swr.get_or_refresh(
    "user:123",
    || async { db.find_user("123").await }
).await?;`
              }
            ]
          }
        ]
      },
      {
        id: 'tag-based-invalidation',
        title: 'Tag-Based Invalidation',
        content: `<p>Invalidate groups of related cache entries efficiently:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            filename: 'tagged_cache.rs',
            code: `use armature_cache::{TaggedCache, InMemoryCache};

let cache = TaggedCache::new(InMemoryCache::new());

// Cache with tags
cache.set_tagged(
    "user:123",
    &user,
    Duration::from_secs(300),
    &["users", "user:123", "team:456"]
).await?;

cache.set_tagged(
    "user:124",
    &user2,
    Duration::from_secs(300),
    &["users", "user:124", "team:456"]
).await?;

// Invalidate all users
cache.invalidate_tag("users").await?;

// Invalidate specific user's caches
cache.invalidate_tag("user:123").await?;

// Invalidate all team members' caches
cache.invalidate_tag("team:456").await?;`
          }
        ]
      },
      {
        id: 'multi-tier',
        title: 'Multi-Tier Caching',
        content: `<p>Combine fast local cache (L1) with shared distributed cache (L2):</p>`,
        codeBlocks: [
          {
            language: 'rust',
            filename: 'tiered_cache.rs',
            code: `use armature_cache::{TieredCache, InMemoryCache, RedisCache};

// L1: Fast local in-memory cache
let l1 = InMemoryCache::new()
    .max_size(10_000)
    .ttl(Duration::from_secs(60));

// L2: Shared Redis cache
let l2 = RedisCache::new("redis://localhost:6379").await?;

// Combined tiered cache
let cache = TieredCache::new(l1, l2);

// Get: checks L1 first, then L2, populates L1 on L2 hit
let user = cache.get::<User>("user:123").await?;

// Set: writes to both L1 and L2
cache.set("user:123", &user, Duration::from_secs(300)).await?;

// Benefits:
// - Sub-millisecond reads for hot data (L1)
// - Shared state across instances (L2)
// - Automatic cache warming on miss`
          }
        ]
      },
      {
        id: 'redis-cache',
        title: 'Redis Integration',
        content: `<p>Use Redis for distributed caching across multiple instances:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            filename: 'redis_cache.rs',
            code: `use armature_cache::RedisCache;

// Connect to Redis
let cache = RedisCache::new("redis://localhost:6379").await?;

// With connection pool
let cache = RedisCache::builder()
    .url("redis://localhost:6379")
    .pool_size(10)
    .connection_timeout(Duration::from_secs(5))
    .build()
    .await?;

// Cluster support
let cache = RedisCache::cluster(&[
    "redis://node1:6379",
    "redis://node2:6379",
    "redis://node3:6379",
]).await?;

// Sentinel support
let cache = RedisCache::sentinel(
    "mymaster",
    &["sentinel1:26379", "sentinel2:26379"]
).await?;`
          }
        ]
      },
      {
        id: 'controller-integration',
        title: 'Controller Integration',
        content: `<p>Inject cache service into controllers:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            filename: 'cached_controller.rs',
            code: `use armature::prelude::*;
use armature_cache::Cache;

#[controller("/api/users")]
#[derive(Clone)]
struct UserController {
    cache: Cache,
    db: DatabaseService,
}

impl UserController {
    #[get("/:id")]
    async fn get_user(&self, req: HttpRequest) -> Result<Json<User>, Error> {
        let id = req.param("id")?;
        let cache_key = format!("user:{}", id);

        // Try cache
        if let Some(user) = self.cache.get(&cache_key).await? {
            return Ok(Json(user));
        }

        // Load and cache
        let user = self.db.find_user(&id).await?;
        self.cache.set(&cache_key, &user, Duration::from_secs(300)).await?;

        Ok(Json(user))
    }

    #[put("/:id")]
    async fn update_user(&self, req: HttpRequest) -> Result<Json<User>, Error> {
        let id = req.param("id")?;
        let user: User = req.json().await?;

        // Update database
        self.db.update_user(&id, &user).await?;

        // Invalidate cache
        self.cache.delete(&format!("user:{}", id)).await?;

        Ok(Json(user))
    }
}`
          }
        ]
      },
      {
        id: 'best-practices',
        title: 'Best Practices',
        content: `<ul>
          <li><strong>Use meaningful keys</strong> — <code>user:123:profile</code> not <code>key1</code></li>
          <li><strong>Set appropriate TTLs</strong> — Balance freshness vs. performance</li>
          <li><strong>Use tags for relationships</strong> — Easy invalidation of related data</li>
          <li><strong>Consider cache stampedes</strong> — Use locking or stale-while-revalidate</li>
          <li><strong>Monitor hit rates</strong> — Aim for 90%+ hit rate on hot data</li>
          <li><strong>Size limits</strong> — Set max memory limits to prevent OOM</li>
          <li><strong>Serialize efficiently</strong> — Use compact serialization (MessagePack, bincode)</li>
        </ul>`
      }
    ],
    relatedDocs: [
      {
        id: 'redis-guide',
        title: 'Redis Integration',
        description: 'Deep dive into Redis features and configuration'
      },
      {
        id: 'response-caching',
        title: 'HTTP Response Caching',
        description: 'Cache-Control headers and browser caching'
      },
      {
        id: 'etag-conditional',
        title: 'ETags & Conditional Requests',
        description: 'Optimize bandwidth with conditional requests'
      }
    ],
    seeAlso: [
      { title: 'Performance Benchmarks', id: 'armature-vs-nodejs' },
      { title: 'Health Checks', id: 'health-check' },
      { title: 'Logging Guide', id: 'logging-guide' }
    ]
  };
}

