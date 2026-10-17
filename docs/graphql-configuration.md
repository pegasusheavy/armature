## GraphQL Configuration and Documentation

Armature's GraphQL module provides comprehensive configuration options for controlling playgrounds, documentation endpoints, and schema introspection.

---

## Table of Contents

- [Configuration](#configuration)
- [GraphQL Config Options](#graphql-config-options)
- [Development vs Production](#development-vs-production)
- [Schema Documentation](#schema-documentation)
- [Playground Options](#playground-options)
- [Security Considerations](#security-considerations)
- [Examples](#examples)

---

## Configuration

### Basic Configuration

```rust
use armature_graphql::GraphQLConfig;

// Default configuration: locked down (no playgrounds, no schema docs,
// no introspection; depth, complexity, directive and recursion limits on)
let config = GraphQLConfig::new("/graphql");

// Development configuration (developer tooling enabled)
let config = GraphQLConfig::development("/graphql");

// Production configuration: identical to `new()`, kept as a
// self-documenting constructor
let config = GraphQLConfig::production("/graphql");
```

`GraphQLConfig::default()` is `GraphQLConfig::new("/graphql")`. Developer
tooling is **opt-in**: use `development()` or the `with_*` builders to turn it
on.

### Secure Defaults

| Setting | `new()` / `default()` / `production()` | `development()` |
|---------|----------------------------------------|-----------------|
| `enable_playground` | `false` | `true` |
| `enable_graphiql` | `false` | `true` |
| `enable_schema_docs` | `false` | `true` |
| `enable_introspection` | `false` | `true` |
| `enable_tracing` | `false` | `true` |
| `max_depth` | `15` (`DEFAULT_MAX_DEPTH`) | `15` |
| `max_complexity` | `1000` (`DEFAULT_MAX_COMPLEXITY`) | `1000` |
| `max_directives` | `10` (`DEFAULT_MAX_DIRECTIVES`) | `10` |
| `max_recursive_depth` | `32` (`DEFAULT_MAX_RECURSIVE_DEPTH`) | `32` |

`SchemaBuilder`, `ProgrammaticSchemaBuilder` and `SubgraphSchemaBuilder` apply
`GraphQLConfig::default()` when no config is attached, so a schema built
without `.config(..)` is non-introspectable and depth/complexity limited. Attach
`.config(GraphQLConfig::development("/graphql"))` (or a config with
`with_introspection(true)`) to opt back in during development.

---

## GraphQL Config Options

### Available Options

```rust
pub struct GraphQLConfig {
    /// GraphQL endpoint path
    pub endpoint: String,

    /// Enable GraphQL Playground (interactive GraphQL IDE)
    pub enable_playground: bool,

    /// Playground endpoint path (if enabled)
    pub playground_endpoint: String,

    /// Enable GraphiQL (lighter alternative to Playground)
    pub enable_graphiql: bool,

    /// GraphiQL endpoint path (if enabled)
    pub graphiql_endpoint: String,

    /// Enable schema documentation endpoint
    pub enable_schema_docs: bool,

    /// Schema documentation endpoint path
    pub schema_docs_endpoint: String,

    /// Enable introspection queries (required for playgrounds and docs)
    pub enable_introspection: bool,

    /// Maximum query depth (0 = unlimited; default 15)
    pub max_depth: usize,

    /// Maximum query complexity (0 = unlimited; default 1000).
    /// Every aliased field costs at least 1, so this also bounds
    /// alias amplification.
    pub max_complexity: usize,

    /// Maximum directives per field (0 = unlimited; default 10)
    pub max_directives: usize,

    /// Maximum parser recursion depth (default 32; 0 falls back to 32)
    pub max_recursive_depth: usize,

    /// Enable query validation
    pub enable_validation: bool,

    /// Enable Apollo Tracing
    pub enable_tracing: bool,
}
```

### Builder Pattern

```rust
let config = GraphQLConfig::new("/api/graphql")
    .with_playground(true)
    .with_graphiql(false)
    .with_schema_docs(true)
    .with_introspection(true)
    .with_max_depth(10)
    .with_max_complexity(100)
    .with_max_directives(10)
    .with_max_recursive_depth(32)
    .with_validation(true)
    .with_tracing(false);
```

---

## Development vs Production

### Development Configuration

Enable all features for the best developer experience:

```rust
let config = GraphQLConfig::development("/graphql");

// Equivalent to:
let config = GraphQLConfig::new("/graphql")
    .with_playground(true)
    .with_graphiql(true)
    .with_schema_docs(true)
    .with_introspection(true)
    .with_tracing(true);
```

**Features enabled:**
- ✅ GraphQL Playground
- ✅ GraphiQL
- ✅ Schema documentation
- ✅ Introspection queries
- ✅ Apollo tracing

### Production Configuration

Playgrounds and introspection are already off by default:

```rust
let config = GraphQLConfig::production("/graphql");

// Identical to:
let config = GraphQLConfig::new("/graphql");
```

**Features disabled:**
- ❌ GraphQL Playground
- ❌ GraphiQL
- ❌ Schema documentation (can be enabled separately)
- ❌ Introspection queries

---

## Schema Documentation

### Documentation Endpoint

Armature generates beautiful, interactive schema documentation:

```rust
use armature_graphql::{generate_schema_docs_html, Schema};

let html = generate_schema_docs_html(
    &schema,
    "/graphql",      // GraphQL endpoint
    "My API"         // API title
);
```

### Features of Schema Documentation

1. **Interactive Schema Viewer**
   - Browse types, queries, mutations, subscriptions
   - Syntax highlighting for SDL
   - Copy schema to clipboard

2. **Getting Started Guide**
   - API endpoint information
   - Example queries
   - Integration instructions

3. **Example Queries**
   - Introspection queries
   - Type information queries
   - Common patterns

4. **Beautiful UI**
   - Modern, responsive design
   - Tabbed interface
   - Professional styling

### Accessing Documentation

Once enabled, access documentation at:
- **Schema Docs**: `http://localhost:3000/graphql/schema`
- **SDL Download**: `http://localhost:3000/graphql/schema.graphql`

These endpoints expose the full schema, so serve them only where introspection is
allowed. The `graphql-api` project template gates them on the same switch as
introspection: they return `404` unless `GRAPHQL_INTROSPECTION=true` (default `false`).

---

## Playground Options

### GraphQL Playground

Full-featured GraphQL IDE (off by default) with:
- Query editor with syntax highlighting
- Variable editor
- Response viewer
- Schema documentation sidebar
- Query history
- Multiple tabs

```rust
// Enable Playground
let config = GraphQLConfig::new("/graphql")
    .with_playground(true)
    .with_playground_endpoint("/graphql/playground");
```

**Access at**: `http://localhost:3000/graphql/playground`

### GraphiQL

Lighter alternative with:
- Query editor
- Variable support
- Schema explorer
- Query execution
- Faster load times

```rust
// Enable GraphiQL
let config = GraphQLConfig::new("/graphql")
    .with_graphiql(true)
    .with_graphiql_endpoint("/graphql/graphiql");
```

**Access at**: `http://localhost:3000/graphql/graphiql`

### Disabling Playgrounds

For production environments:

```rust
let config = GraphQLConfig::new("/graphql");
// Playgrounds are disabled by default; nothing to turn off
```

### Pinned Assets

The Playground and GraphiQL pages load exact, pinned versions from the CDN with
Subresource Integrity (`integrity="sha384-..."`) and `crossorigin="anonymous"`,
so a tampered or substituted CDN file is refused by the browser. The versions are
exposed as `GRAPHQL_PLAYGROUND_VERSION` (`1.7.28`), `GRAPHIQL_VERSION`
(`3.9.0`) and `GRAPHIQL_REACT_VERSION` (`18.3.1`). The endpoint interpolated
into the page is emitted as an escaped JavaScript string literal.

---

## Security Considerations

### Introspection Queries

Introspection allows clients to query the GraphQL schema structure. While useful for development, it can expose your API structure in production.

Introspection is disabled by default. Enable it only for development builds:

```rust
let config = if cfg!(debug_assertions) {
    GraphQLConfig::development("/graphql")
} else {
    GraphQLConfig::production("/graphql")
};
```

### Query Complexity Limits

Depth (15), complexity (1000), directives-per-field (10) and parser recursion
(32) are limited by default. Tighten them for your schema:

```rust
let config = GraphQLConfig::new("/graphql")
    .with_max_depth(10)          // Limit query depth
    .with_max_complexity(100)    // Limit overall complexity (also bounds aliases)
    .with_max_directives(5);     // Limit directives per field
```

Setting a limit to `0` removes it; avoid that for any internet-facing endpoint.

### Playground Access Control

Implement authentication for playgrounds in production:

```rust
#[get("/playground")]
async fn playground(&self, req: HttpRequest) -> Result<HttpResponse, Error> {
    // Check authentication
    if !is_authenticated(&req) {
        return Err(Error::Unauthorized("authentication required".into()));
    }

    // Only allow in development or for authorized users
    if !cfg!(debug_assertions) && !is_admin(&req) {
        return Err(Error::Forbidden("admin only".into()));
    }

    Ok(/* playground HTML */)
}
```

---

## Examples

### Example 1: Development Setup

```rust
use armature_graphql::GraphQLConfig;

let config = GraphQLConfig::development("/graphql");

// All features enabled for development
assert!(config.enable_playground);
assert!(config.enable_graphiql);
assert!(config.enable_schema_docs);
assert!(config.enable_introspection);
```

### Example 2: Production Setup

```rust
let config = GraphQLConfig::production("/graphql");

// Playgrounds disabled for security (same as `GraphQLConfig::new`)
assert!(!config.enable_playground);
assert!(!config.enable_graphiql);
assert!(!config.enable_introspection);
assert_eq!(config.max_depth, 15);
assert_eq!(config.max_complexity, 1000);
```

### Example 3: Custom Configuration

```rust
let config = GraphQLConfig::new("/api/v1/graphql")
    .with_playground(false)           // Disable Playground
    .with_graphiql(true)              // Enable GraphiQL
    .with_schema_docs(true)           // Enable documentation
    .with_introspection(true)         // Enable introspection
    .with_max_depth(15)               // Limit depth
    .with_max_complexity(200)         // Limit complexity
    .with_tracing(false);             // Disable tracing
```

### Example 4: Environment-Based Configuration

```rust
use std::env;

let config = match env::var("ENV").unwrap_or_default().as_str() {
    "production" => GraphQLConfig::production("/graphql"),
    "staging" => GraphQLConfig::new("/graphql")
        .with_playground(true)
        .with_introspection(true)
        .with_schema_docs(false),
    _ => GraphQLConfig::development("/graphql"),
};
```

### Example 5: Full Controller Implementation

```rust
use armature::prelude::*;
use armature_graphql::*;

#[controller("/graphql")]
struct GraphQLController {
    schema: Schema<Query, Mutation, Subscription>,
    config: GraphQLConfig,
}

impl GraphQLController {
    #[post("/")]
    async fn execute(&self, req: HttpRequest) -> Result<HttpResponse, Error> {
        // Execute GraphQL query
        // ...
    }

    #[get("/playground")]
    async fn playground(&self, _req: HttpRequest) -> Result<HttpResponse, Error> {
        if !self.config.enable_playground {
            return Err(Error::NotFound("not found".into()));
        }

        Ok(HttpResponse::html(graphql_playground_html(&self.config.endpoint)))
    }

    #[get("/schema")]
    async fn schema_docs(&self, _req: HttpRequest) -> Result<HttpResponse, Error> {
        if !self.config.enable_schema_docs {
            return Err(Error::NotFound("not found".into()));
        }

        let html = generate_schema_docs_html(&self.schema, &self.config.endpoint, "My API");
        Ok(HttpResponse::html(html))
    }
}
```

---

## Best Practices

1. **Use Environment-Based Config**
   - Development: All features enabled
   - Staging: Selective features
   - Production: the defaults (`GraphQLConfig::new` / `production`)

2. **Protect Sensitive Endpoints**
   - Add authentication to playgrounds
   - Use rate limiting
   - Monitor access logs

3. **Enable Documentation Selectively**
   - Public APIs: Enable schema docs
   - Internal APIs: Disable in production
   - Authenticated APIs: Require auth

4. **Set Complexity Limits**
   - Prevent denial-of-service attacks
   - Limit query depth
   - Monitor query complexity

5. **Use HTTPS in Production**
   - Encrypt GraphQL traffic
   - Protect sensitive data
   - Enable CORS appropriately

---

## Summary

Armature's GraphQL configuration provides:

✅ **Flexible playground options** (Playground, GraphiQL)
✅ **Interactive schema documentation**
✅ **Locked-down defaults** with opt-in development tooling
✅ **Security controls** (introspection, complexity limits)
✅ **Customizable endpoints**
✅ **Easy integration** with Armature framework

For complete examples, see:
- `examples/graphql_with_docs.rs`
- `examples/graphql_api.rs`
- `examples/graphql_programmatic.rs`


