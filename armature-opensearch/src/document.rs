//! Document trait and helpers.

use serde::{de::DeserializeOwned, Serialize};

/// Trait for documents that can be indexed in OpenSearch.
///
/// # Example
///
/// ```rust
/// use armature_opensearch::Document;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Debug, Serialize, Deserialize)]
/// struct Product {
///     name: String,
///     price: f64,
///     category: String,
/// }
///
/// impl Document for Product {
///     fn index_name() -> &'static str {
///         "products"
///     }
/// }
/// ```
pub trait Document: Serialize + DeserializeOwned + Send + Sync {
    /// Returns the default index name for this document type.
    fn index_name() -> &'static str;

    /// Returns the document type (deprecated in OpenSearch, defaults to "_doc").
    fn doc_type() -> &'static str {
        "_doc"
    }

    /// Returns the routing key for this document (optional).
    fn routing(&self) -> Option<String> {
        None
    }
}

/// Document metadata returned from search results.
#[derive(Debug, Clone)]
pub struct DocumentMeta {
    /// Document ID.
    pub id: String,
    /// Index name.
    pub index: String,
    /// Document score (relevance).
    pub score: Option<f64>,
    /// Document version.
    pub version: Option<i64>,
    /// Sequence number.
    pub seq_no: Option<i64>,
    /// Primary term.
    pub primary_term: Option<i64>,
    /// Routing value.
    pub routing: Option<String>,
    /// Highlighted fields.
    pub highlight: Option<std::collections::HashMap<String, Vec<String>>>,
}

/// A document with its metadata.
#[derive(Debug, Clone)]
pub struct DocumentWithMeta<T> {
    /// The document data.
    pub doc: T,
    /// Document metadata.
    pub meta: DocumentMeta,
}

