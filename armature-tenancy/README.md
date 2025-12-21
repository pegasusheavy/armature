# armature-tenancy

Multi-tenancy support for the Armature framework.

## Features

- **Tenant Isolation** - Data separation per tenant
- **Multiple Strategies** - Schema, database, row-level
- **Tenant Resolution** - Subdomain, header, path
- **Middleware** - Automatic tenant context
- **Database Routing** - Per-tenant connections

## Installation

```toml
[dependencies]
armature-tenancy = "0.1"
```

## Quick Start

```rust
use armature_tenancy::{TenantMiddleware, TenantResolver};

let tenant_middleware = TenantMiddleware::new(
    TenantResolver::subdomain()  // tenant.example.com
);

let app = Application::new()
    .with_middleware(tenant_middleware)
    .get("/data", |req| async move {
        let tenant = req.tenant()?;
        let data = fetch_tenant_data(&tenant.id).await?;
        Ok(HttpResponse::ok().json(data))
    });
```

## Tenant Resolution

### Subdomain

```rust
TenantResolver::subdomain() // tenant.example.com
```

### Header

```rust
TenantResolver::header("X-Tenant-ID")
```

### Path

```rust
TenantResolver::path_prefix() // /tenant/api/...
```

## Database Strategies

### Schema Isolation

```rust
let pool = TenantPool::schema_per_tenant(base_pool);
```

### Database Isolation

```rust
let pool = TenantPool::database_per_tenant(connections);
```

### Row-Level

```rust
let pool = TenantPool::row_level(pool, "tenant_id");
```

## License

MIT OR Apache-2.0

