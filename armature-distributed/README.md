# armature-distributed

Distributed system primitives for the Armature framework.

## Features

- **Distributed Locks** - Coordinated access control
- **Leader Election** - Single leader among instances
- **Rate Limiting** - Distributed rate limits
- **Caching** - Distributed cache invalidation
- **Circuit Breaker** - Shared circuit state

## Installation

```toml
[dependencies]
armature-distributed = "0.1"
```

## Quick Start

### Distributed Lock

```rust
use armature_distributed::DistributedLock;

let lock = DistributedLock::redis("redis://localhost:6379").await?;

// Acquire lock
let guard = lock.acquire("resource-key", Duration::from_secs(30)).await?;

// Do exclusive work...

// Lock released when guard is dropped
```

### Leader Election

```rust
use armature_distributed::LeaderElection;

let election = LeaderElection::redis("redis://localhost:6379").await?;

election.run("my-service", || async {
    // This only runs on the leader
    process_jobs().await
}).await?;
```

### Distributed Rate Limiter

```rust
use armature_distributed::DistributedRateLimiter;

let limiter = DistributedRateLimiter::redis(
    "redis://localhost:6379",
    100,  // requests
    Duration::from_secs(60),  // window
).await?;

if limiter.check("user:123").await? {
    // Allow request
}
```

## License

MIT OR Apache-2.0

