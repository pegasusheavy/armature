# armature-discovery

Service discovery for the Armature framework.

## Features

- **Service Registration** - Register services on startup
- **Service Discovery** - Find other services
- **Health Checks** - Automatic health monitoring
- **Load Balancing** - Client-side load balancing
- **Multiple Backends** - Consul, etcd, Kubernetes

## Installation

```toml
[dependencies]
armature-discovery = "0.1"
```

## Quick Start

```rust
use armature_discovery::{Discovery, ConsulBackend};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let discovery = Discovery::consul("http://localhost:8500").await?;

    // Register this service
    discovery.register("my-service", "http://localhost:3000").await?;

    // Discover other services
    let instances = discovery.discover("user-service").await?;

    // Get one instance (load balanced)
    let instance = discovery.get_instance("user-service").await?;

    Ok(())
}
```

## Backends

### Consul

```rust
let discovery = Discovery::consul("http://localhost:8500").await?;
```

### etcd

```rust
let discovery = Discovery::etcd("http://localhost:2379").await?;
```

### Kubernetes

```rust
let discovery = Discovery::kubernetes().await?;
```

## License

MIT OR Apache-2.0

