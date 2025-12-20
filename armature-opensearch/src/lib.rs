//! OpenSearch integration for the Armature framework.
//!
//! This crate provides a high-level client for OpenSearch with support for:
//! - Document indexing, searching, and management
//! - Index lifecycle management
//! - Bulk operations with streaming support
//! - Query DSL builder
//! - AWS OpenSearch Service authentication
//!
//! # Example
//!
//! ```rust,no_run
//! use armature_opensearch::{OpenSearchClient, OpenSearchConfig, Document};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, Serialize, Deserialize)]
//! struct Article {
//!     title: String,
//!     body: String,
//!     tags: Vec<String>,
//! }
//!
//! impl Document for Article {
//!     fn index_name() -> &'static str {
//!         "articles"
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create client
//!     let config = OpenSearchConfig::new("http://localhost:9200");
//!     let client = OpenSearchClient::new(config)?;
//!
//!     // Index a document
//!     let article = Article {
//!         title: "Hello OpenSearch".to_string(),
//!         body: "Getting started with full-text search.".to_string(),
//!         tags: vec!["tutorial".to_string(), "search".to_string()],
//!     };
//!
//!     client.index("article-1", &article).await?;
//!
//!     // Search
//!     let results: Vec<Article> = client
//!         .search()
//!         .index("articles")
//!         .query_string("hello")
//!         .execute()
//!         .await?;
//!
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

mod client;
mod config;
mod document;
mod error;
mod index;
mod query;
mod search;

#[cfg(feature = "bulk-stream")]
mod bulk;

pub use client::OpenSearchClient;
pub use config::{OpenSearchConfig, TlsConfig};
pub use document::Document;
pub use error::{OpenSearchError, Result};
pub use index::{IndexManager, IndexSettings, Mapping, MappingField, FieldType};
pub use query::{Query, QueryBuilder, BoolQuery, MatchQuery, TermQuery, RangeQuery};
pub use search::{SearchBuilder, SearchResult, Hit, Aggregation, AggregationResult};

#[cfg(feature = "bulk-stream")]
pub use bulk::{BulkOperation, BulkResponse, BulkItem};

/// Prelude for common imports.
pub mod prelude {
    pub use crate::{
        OpenSearchClient, OpenSearchConfig, Document,
        OpenSearchError, Result,
        Query, QueryBuilder, SearchBuilder,
    };
}

