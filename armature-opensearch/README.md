# armature-opensearch

OpenSearch integration for the Armature framework.

## Features

- **Document CRUD** - Index, get, update, delete documents
- **Search** - Full-text and structured queries
- **Query DSL** - Type-safe query builder
- **Bulk Operations** - Efficient batch processing
- **Index Management** - Create, configure, delete indices
- **AWS OpenSearch** - AWS authentication support

## Installation

```toml
[dependencies]
armature-opensearch = "0.1"
```

## Quick Start

```rust
use armature_opensearch::{OpenSearchClient, Document, Query};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OpenSearchClient::new("http://localhost:9200").await?;

    // Index a document
    client.index("products", "1", &product).await?;

    // Search
    let results = client.search::<Product>("products")
        .query(Query::match_field("name", "laptop"))
        .size(10)
        .execute()
        .await?;

    // Bulk operations
    client.bulk()
        .index("products", "2", &product2)
        .delete("products", "old-id")
        .execute()
        .await?;

    Ok(())
}
```

## Query DSL

```rust
let query = Query::bool()
    .must(Query::match_field("category", "electronics"))
    .filter(Query::range("price").gte(100).lte(500))
    .should(Query::term("featured", true));
```

## License

MIT OR Apache-2.0

