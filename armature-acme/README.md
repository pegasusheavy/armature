# armature-acme

ACME/Let's Encrypt certificate automation for the Armature framework.

## Features

- **Auto Certificates** - Automatic TLS certificate provisioning
- **Let's Encrypt** - Built-in Let's Encrypt support
- **HTTP-01 Challenge** - Domain validation
- **Auto Renewal** - Automatic certificate renewal
- **Multiple Domains** - SAN certificate support

## Installation

```toml
[dependencies]
armature-acme = "0.1"
```

## Quick Start

```rust
use armature_acme::AcmeConfig;

let app = Application::new()
    .with_acme(AcmeConfig {
        domains: vec!["example.com", "www.example.com"],
        email: "admin@example.com",
        staging: false, // Use production Let's Encrypt
    })
    .get("/", handler);

app.listen_tls("0.0.0.0:443").await?;
```

## Staging Environment

```rust
// Use Let's Encrypt staging for testing
let config = AcmeConfig::staging(vec!["example.com"], "admin@example.com");
```

## Certificate Storage

```rust
let config = AcmeConfig::new(domains, email)
    .certificate_path("/etc/ssl/certs")
    .key_path("/etc/ssl/private");
```

## License

MIT OR Apache-2.0

