# armature-ferron

Ferron reverse proxy integration for the Armature framework.

## Features

- **Reverse Proxy** - Route requests to backends
- **Load Balancing** - Distribute traffic
- **SSL Termination** - Handle TLS at proxy
- **Health Checks** - Monitor backend health

## Installation

```toml
[dependencies]
armature-ferron = "0.1"
```

## Quick Start

```rust
use armature_ferron::FerronProxy;

let proxy = FerronProxy::new()
    .backend("http://localhost:8001")
    .backend("http://localhost:8002")
    .health_check("/health")
    .build();

proxy.listen("0.0.0.0:80").await?;
```

## License

MIT OR Apache-2.0

