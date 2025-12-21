# armature-compression

HTTP response compression middleware for the Armature framework.

## Features

- **Multiple Algorithms** - gzip, brotli, zstd
- **Content Negotiation** - Accept-Encoding support
- **Streaming** - Compress as data is generated
- **Configurable** - Min size, compression levels
- **Auto Detection** - Skip already compressed content

## Installation

```toml
[dependencies]
armature-compression = "0.1"
```

## Quick Start

```rust
use armature_compression::CompressionMiddleware;
use armature_core::Application;

let app = Application::new()
    .with_middleware(CompressionMiddleware::default())
    .get("/data", handler);
```

## Configuration

```rust
let compression = CompressionMiddleware::new()
    .gzip(CompressionLevel::Default)
    .brotli(CompressionLevel::Best)
    .min_size(1024)  // Don't compress < 1KB
    .exclude_types(vec!["image/*", "video/*"]);
```

## Streaming Compression

```rust
use armature_compression::StreamingCompressor;

let compressor = StreamingCompressor::gzip();

// Compress chunks as they're generated
for chunk in data_stream {
    let compressed = compressor.compress_chunk(&chunk)?;
    response.write(compressed).await?;
}
```

## License

MIT OR Apache-2.0

