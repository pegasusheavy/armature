# armature-config

Configuration management for the Armature framework.

## Features

- **Multiple Sources** - Environment, files, remote
- **File Formats** - TOML, YAML, JSON
- **Type-Safe** - Deserialize into typed structs
- **Hot Reload** - Watch for config changes
- **Secrets** - Vault, AWS Secrets Manager integration

## Installation

```toml
[dependencies]
armature-config = "0.1"
```

## Quick Start

```rust
use armature_config::{Config, Environment};
use serde::Deserialize;

#[derive(Deserialize)]
struct AppConfig {
    database_url: String,
    port: u16,
    debug: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config: AppConfig = Config::builder()
        .add_source(Environment::with_prefix("APP"))
        .add_source(File::with_name("config"))
        .build()?
        .try_deserialize()?;

    println!("Port: {}", config.port);
    Ok(())
}
```

## Environment Variables

```bash
APP_DATABASE_URL=postgres://localhost/mydb
APP_PORT=3000
APP_DEBUG=true
```

## Config Files

```toml
# config.toml
database_url = "postgres://localhost/mydb"
port = 3000
debug = true
```

## License

MIT OR Apache-2.0

