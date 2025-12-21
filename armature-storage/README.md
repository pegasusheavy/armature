# armature-storage

Object storage for the Armature framework.

## Features

- **Multiple Providers** - S3, Azure Blob, GCS, local filesystem
- **Unified API** - Same interface for all providers
- **Streaming** - Stream large files
- **Presigned URLs** - Secure temporary access
- **Multipart Upload** - Large file uploads

## Installation

```toml
[dependencies]
armature-storage = "0.1"
```

## Quick Start

```rust
use armature_storage::{Storage, S3Storage};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let storage = S3Storage::new("my-bucket", "us-east-1").await?;

    // Upload
    storage.put("files/doc.pdf", bytes).await?;

    // Download
    let data = storage.get("files/doc.pdf").await?;

    // Delete
    storage.delete("files/doc.pdf").await?;

    // Presigned URL
    let url = storage.presigned_url("files/doc.pdf", Duration::from_secs(3600))?;

    Ok(())
}
```

## Providers

### AWS S3

```rust
let storage = S3Storage::new("bucket", "region").await?;
```

### Azure Blob

```rust
let storage = AzureBlobStorage::new("container", "connection_string").await?;
```

### Google Cloud Storage

```rust
let storage = GcsStorage::new("bucket").await?;
```

### Local Filesystem

```rust
let storage = LocalStorage::new("/data/uploads");
```

## License

MIT OR Apache-2.0

