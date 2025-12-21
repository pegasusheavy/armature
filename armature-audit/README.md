# armature-audit

Audit logging and compliance for the Armature framework.

## Features

- **Audit Trail** - Track all changes
- **User Actions** - Who did what, when
- **Data Changes** - Before/after values
- **Compliance** - GDPR, HIPAA, SOC 2 support
- **Multiple Sinks** - Database, file, external services

## Installation

```toml
[dependencies]
armature-audit = "0.1"
```

## Quick Start

```rust
use armature_audit::{AuditLog, AuditEntry};

let audit = AuditLog::new()
    .sink(DatabaseSink::new(pool))
    .sink(FileSink::new("/var/log/audit.log"));

// Log an action
audit.log(AuditEntry {
    actor: "user123",
    action: "user.update",
    resource: "user/456",
    changes: json!({
        "before": {"email": "old@example.com"},
        "after": {"email": "new@example.com"}
    }),
}).await?;
```

## Middleware

```rust
let app = Application::new()
    .with_middleware(AuditMiddleware::new(audit))
    .put("/users/:id", update_user);
```

## Query Audit Logs

```rust
let logs = audit.query()
    .actor("user123")
    .action("user.*")
    .since(Utc::now() - Duration::days(30))
    .execute()
    .await?;
```

## License

MIT OR Apache-2.0

