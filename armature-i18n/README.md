# armature-i18n

Internationalization (i18n) support for the Armature framework.

## Features

- **Message Translation** - Fluent-based translations
- **Locale Detection** - Accept-Language header parsing
- **Pluralization** - CLDR plural rules
- **Date/Number Formatting** - Locale-aware formatting
- **Locale Negotiation** - Best-match locale selection

## Installation

```toml
[dependencies]
armature-i18n = "0.1"
```

## Quick Start

```rust
use armature_i18n::{I18n, Locale};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let i18n = I18n::new()
        .add_bundle("en", include_str!("locales/en.ftl"))
        .add_bundle("es", include_str!("locales/es.ftl"));

    // Simple translation
    let msg = i18n.t("en", "hello")?;

    // With arguments
    let msg = i18n.t_args("en", "greeting", &[("name", "World")])?;

    // Pluralization
    let msg = i18n.t_plural("en", "items", 5)?;

    Ok(())
}
```

## Fluent Files

```ftl
# locales/en.ftl
hello = Hello!
greeting = Hello, { $name }!
items = { $count ->
    [one] { $count } item
   *[other] { $count } items
}
```

## Locale Detection

```rust
use armature_i18n::Locale;

// Parse Accept-Language header
let locales = Locale::from_accept_language("en-US,en;q=0.9,es;q=0.8");

// Negotiate best locale
let best = i18n.negotiate(&locales, &["en", "es", "fr"]);
```

## License

MIT OR Apache-2.0

