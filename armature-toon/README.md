# armature-toon

TOON (Token-Oriented Object Notation) integration for the Armature framework.

## Features

- **LLM Optimized** - 30-60% fewer tokens than JSON
- **Serde Compatible** - Works with existing types
- **HTTP Integration** - Request/response handling
- **Content Negotiation** - Auto-detect TOON requests

## Installation

```toml
[dependencies]
armature-toon = "0.1"
```

## Quick Start

```rust
use armature_toon::{to_string, from_str, Toon};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct User {
    name: String,
    age: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let user = User { name: "Alice".into(), age: 30 };

    // Serialize to TOON
    let toon = to_string(&user)?;

    // Deserialize from TOON
    let parsed: User = from_str(&toon)?;

    Ok(())
}
```

## HTTP Integration

```rust
use armature_toon::ToonResponseExt;

async fn handler(req: HttpRequest) -> Result<HttpResponse, Error> {
    let user = User { name: "Alice".into(), age: 30 };
    HttpResponse::toon(user)
}
```

## Token Comparison

| Format | Tokens | Savings |
|--------|--------|---------|
| JSON   | 100    | -       |
| TOON   | 45     | 55%     |

## License

MIT OR Apache-2.0

