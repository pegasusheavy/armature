# armature-log

Logging utilities for the Armature framework.

## Features

- **JSON by Default** - Structured logging for production
- **Pretty Output** - Human-readable format for development
- **Environment Config** - Configure via `ARMATURE_*` env vars
- **Runtime Config** - Programmatic configuration API
- **Zero Dependencies** - Minimal footprint (optional tracing integration)

## Installation

```toml
[dependencies]
armature-log = "0.1"
```

## Quick Start

```rust
use armature_log::{info, debug, error};

fn main() {
    // Uses ARMATURE_DEBUG and ARMATURE_LOG_LEVEL env vars
    armature_log::init();

    info!("Application started");
    debug!("Debug message");
    error!("Something went wrong");
}
```

## Configuration

### Environment Variables

| Variable | Values | Default |
|----------|--------|---------|
| `ARMATURE_DEBUG` | `1`, `true` | `false` |
| `ARMATURE_LOG_LEVEL` | `trace`, `debug`, `info`, `warn`, `error` | `info` |
| `ARMATURE_LOG_FORMAT` | `json`, `pretty`, `compact` | `json` |
| `ARMATURE_LOG_COLOR` | `1`, `true` | auto-detect TTY |

### Programmatic

```rust
use armature_log::{configure, Format, Level};

configure()
    .format(Format::Pretty)
    .level(Level::Debug)
    .color(true)
    .apply();
```

### Presets

```rust
// Development: Pretty, Debug, Colors
armature_log::preset_development();

// Production: JSON, Info, No colors
armature_log::preset_production();
```

## License

MIT OR Apache-2.0

