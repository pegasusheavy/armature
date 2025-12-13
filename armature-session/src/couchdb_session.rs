//! CouchDB session storage implementation.
//!
//! This module requires the `couchdb` feature flag.

use crate::config::SessionConfig;
use crate::error::{SessionError, SessionResult};
use crate::traits::{Session, SessionStore, generate_session_id};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// CouchDB document wrapper for sessions.
#[derive(Debug, Serialize, Deserialize)]
struct CouchDbSession {
    #[serde(rename = "_id")]
    id: String,
    #[serde(rename = "_rev", skip_serializing_if = "Option::is_none")]
    rev: Option<String>,
    #[serde(flatten)]
    session: Session,
}

/// CouchDB-backed session store.
///
/// # ⚠️ Important: Prefer Stateless Architecture
///
/// **Armature strongly recommends stateless architecture using JWT tokens.**
/// Use sessions only when absolutely necessary.
///
/// # Feature Flag
///
/// This requires the `couchdb` feature:
///
/// ```toml
/// [dependencies]
/// armature-session = { version = "0.1", features = ["couchdb"] }
/// ```
///
/// # Database Setup
///
/// Before using CouchDB sessions, create the database and a view for cleanup:
///
/// ```bash
/// # Create database
/// curl -X PUT http://localhost:5984/sessions
///
/// # Create design document with expiration view
/// curl -X PUT http://localhost:5984/sessions/_design/sessions \
///   -H "Content-Type: application/json" \
///   -d '{
///     "views": {
///       "by_expiration": {
///         "map": "function(doc) { if(doc.expires_at) emit(doc.expires_at, null); }"
///       }
///     }
///   }'
/// ```
///
/// # Examples
///
/// ```ignore
/// use armature_session::{CouchDbSessionStore, SessionConfig, SessionStore};
/// use std::time::Duration;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = SessionConfig::couchdb("http://localhost:5984", "sessions")?
///         .with_namespace("myapp")
///         .with_default_ttl(Duration::from_secs(3600))
///         .with_auth("admin", "password");
///
///     let store = CouchDbSessionStore::new(config).await?;
///
///     // Create a session
///     let mut session = store.create(None).await?;
///     session.set("user_id", 123)?;
///     store.save(&session).await?;
///
///     Ok(())
/// }
/// ```
pub struct CouchDbSessionStore {
    client: Client,
    config: SessionConfig,
    base_url: String,
}

impl CouchDbSessionStore {
    /// Create a new CouchDB session store.
    ///
    /// # Arguments
    ///
    /// * `config` - Session configuration
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use armature_session::{CouchDbSessionStore, SessionConfig};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = SessionConfig::couchdb("http://localhost:5984", "sessions")?;
    /// let store = CouchDbSessionStore::new(config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(config: SessionConfig) -> SessionResult<Self> {
        let database = config.database.as_ref().ok_or_else(|| {
            SessionError::Config("CouchDB database name is required".to_string())
        })?;

        let base_url = format!("{}/{}", config.url.trim_end_matches('/'), database);

        let client = Client::builder()
            .build()
            .map_err(|e| SessionError::Connection(e.to_string()))?;

        // Verify connection by checking database exists
        let url = format!("{}", base_url);
        let mut request = client.head(&url);

        if let (Some(username), Some(password)) = (&config.username, &config.password) {
            request = request.basic_auth(username, Some(password));
        }

        let response = request
            .send()
            .await
            .map_err(|e| SessionError::Connection(e.to_string()))?;

        if !response.status().is_success() {
            return Err(SessionError::Connection(format!(
                "Failed to connect to CouchDB database '{}': {}",
                database,
                response.status()
            )));
        }

        Ok(Self {
            client,
            config,
            base_url,
        })
    }

    /// Get the document ID for a session.
    fn doc_id(&self, session_id: &str) -> String {
        format!("{}:{}", self.config.namespace, session_id)
    }

    /// Build an authenticated request.
    fn request(&self, method: reqwest::Method, url: &str) -> reqwest::RequestBuilder {
        let mut request = self.client.request(method, url);

        if let (Some(username), Some(password)) = (&self.config.username, &self.config.password) {
            request = request.basic_auth(username, Some(password));
        }

        request
    }
}

#[async_trait]
impl SessionStore for CouchDbSessionStore {
    async fn create(&self, ttl: Option<Duration>) -> SessionResult<Session> {
        let session_id = generate_session_id();
        let ttl = ttl.unwrap_or(self.config.default_ttl);

        // Enforce max TTL
        let ttl = if ttl > self.config.max_ttl {
            self.config.max_ttl
        } else {
            ttl
        };

        let session = Session::new(&session_id, ttl);

        // Save the session
        self.save(&session).await?;

        Ok(session)
    }

    async fn get(&self, session_id: &str) -> SessionResult<Option<Session>> {
        let doc_id = self.doc_id(session_id);
        let url = format!("{}/{}", self.base_url, doc_id);

        let response = self
            .request(reqwest::Method::GET, &url)
            .send()
            .await
            .map_err(|e| SessionError::Connection(e.to_string()))?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        if !response.status().is_success() {
            return Err(SessionError::CouchDb(format!(
                "Failed to get session: {}",
                response.status()
            )));
        }

        let doc: CouchDbSession = response
            .json()
            .await
            .map_err(|e| SessionError::Deserialization(e.to_string()))?;

        // Check if expired
        if doc.session.is_expired() {
            self.delete(session_id).await?;
            return Ok(None);
        }

        Ok(Some(doc.session))
    }

    async fn save(&self, session: &Session) -> SessionResult<()> {
        let doc_id = self.doc_id(&session.id);
        let url = format!("{}/{}", self.base_url, doc_id);

        // Try to get current revision if document exists
        let rev = {
            let response = self
                .request(reqwest::Method::GET, &url)
                .send()
                .await
                .map_err(|e| SessionError::Connection(e.to_string()))?;

            if response.status().is_success() {
                let doc: CouchDbSession = response
                    .json()
                    .await
                    .map_err(|e| SessionError::Deserialization(e.to_string()))?;
                doc.rev
            } else {
                None
            }
        };

        let doc = CouchDbSession {
            id: doc_id.clone(),
            rev,
            session: session.clone(),
        };

        let response = self
            .request(reqwest::Method::PUT, &url)
            .json(&doc)
            .send()
            .await
            .map_err(|e| SessionError::Connection(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SessionError::CouchDb(format!(
                "Failed to save session: {}",
                error_text
            )));
        }

        Ok(())
    }

    async fn delete(&self, session_id: &str) -> SessionResult<()> {
        let doc_id = self.doc_id(session_id);
        let url = format!("{}/{}", self.base_url, doc_id);

        // Get current revision
        let response = self
            .request(reqwest::Method::GET, &url)
            .send()
            .await
            .map_err(|e| SessionError::Connection(e.to_string()))?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(()); // Already deleted
        }

        let doc: CouchDbSession = response
            .json()
            .await
            .map_err(|e| SessionError::Deserialization(e.to_string()))?;

        if let Some(rev) = doc.rev {
            let delete_url = format!("{}?rev={}", url, rev);
            let response = self
                .request(reqwest::Method::DELETE, &delete_url)
                .send()
                .await
                .map_err(|e| SessionError::Connection(e.to_string()))?;

            if !response.status().is_success()
                && response.status() != reqwest::StatusCode::NOT_FOUND
            {
                return Err(SessionError::CouchDb(format!(
                    "Failed to delete session: {}",
                    response.status()
                )));
            }
        }

        Ok(())
    }

    async fn exists(&self, session_id: &str) -> SessionResult<bool> {
        match self.get(session_id).await? {
            Some(session) => Ok(!session.is_expired()),
            None => Ok(false),
        }
    }

    async fn extend(&self, session_id: &str, ttl: Duration) -> SessionResult<()> {
        if let Some(mut session) = self.get(session_id).await? {
            // Enforce max TTL
            let ttl = if ttl > self.config.max_ttl {
                self.config.max_ttl
            } else {
                ttl
            };

            session.extend(ttl);
            self.save(&session).await?;
        }

        Ok(())
    }

    async fn touch(&self, session_id: &str) -> SessionResult<()> {
        if let Some(mut session) = self.get(session_id).await? {
            session.touch();
            self.save(&session).await?;
        }

        Ok(())
    }

    async fn clear_all(&self) -> SessionResult<()> {
        // Get all documents with our namespace prefix
        let url = format!(
            "{}/_all_docs?startkey=\"{}:\"&endkey=\"{}:\\ufff0\"&include_docs=true",
            self.base_url, self.config.namespace, self.config.namespace
        );

        let response = self
            .request(reqwest::Method::GET, &url)
            .send()
            .await
            .map_err(|e| SessionError::Connection(e.to_string()))?;

        if !response.status().is_success() {
            return Err(SessionError::CouchDb(format!(
                "Failed to list sessions: {}",
                response.status()
            )));
        }

        #[derive(Deserialize)]
        struct AllDocsResponse {
            rows: Vec<DocRow>,
        }

        #[derive(Deserialize)]
        struct DocRow {
            id: String,
            value: DocValue,
        }

        #[derive(Deserialize)]
        struct DocValue {
            rev: String,
        }

        let docs: AllDocsResponse = response
            .json()
            .await
            .map_err(|e| SessionError::Deserialization(e.to_string()))?;

        // Delete each document
        for row in docs.rows {
            let delete_url = format!("{}/{}?rev={}", self.base_url, row.id, row.value.rev);
            let _ = self
                .request(reqwest::Method::DELETE, &delete_url)
                .send()
                .await;
        }

        Ok(())
    }

    async fn count(&self) -> SessionResult<usize> {
        let url = format!(
            "{}/_all_docs?startkey=\"{}:\"&endkey=\"{}:\\ufff0\"",
            self.base_url, self.config.namespace, self.config.namespace
        );

        let response = self
            .request(reqwest::Method::GET, &url)
            .send()
            .await
            .map_err(|e| SessionError::Connection(e.to_string()))?;

        if !response.status().is_success() {
            return Err(SessionError::CouchDb(format!(
                "Failed to count sessions: {}",
                response.status()
            )));
        }

        #[derive(Deserialize)]
        struct AllDocsResponse {
            total_rows: usize,
        }

        let docs: AllDocsResponse = response
            .json()
            .await
            .map_err(|e| SessionError::Deserialization(e.to_string()))?;

        Ok(docs.total_rows)
    }

    async fn cleanup_expired(&self) -> SessionResult<usize> {
        let now = chrono::Utc::now().to_rfc3339();
        let url = format!(
            "{}/_design/sessions/_view/by_expiration?endkey=\"{}\"&include_docs=true",
            self.base_url, now
        );

        let response = self
            .request(reqwest::Method::GET, &url)
            .send()
            .await
            .map_err(|e| SessionError::Connection(e.to_string()))?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            // View doesn't exist, skip cleanup
            return Ok(0);
        }

        if !response.status().is_success() {
            return Err(SessionError::CouchDb(format!(
                "Failed to query expired sessions: {}",
                response.status()
            )));
        }

        #[derive(Deserialize)]
        struct ViewResponse {
            rows: Vec<ViewRow>,
        }

        #[derive(Deserialize)]
        struct ViewRow {
            doc: Option<CouchDbSession>,
        }

        let view: ViewResponse = response
            .json()
            .await
            .map_err(|e| SessionError::Deserialization(e.to_string()))?;

        let mut deleted = 0;
        for row in view.rows {
            if let Some(doc) = row.doc {
                if let Some(rev) = doc.rev {
                    let delete_url = format!("{}/{}?rev={}", self.base_url, doc.id, rev);
                    if self
                        .request(reqwest::Method::DELETE, &delete_url)
                        .send()
                        .await
                        .is_ok()
                    {
                        deleted += 1;
                    }
                }
            }
        }

        Ok(deleted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doc_id_generation() {
        let config = SessionConfig::couchdb("http://localhost:5984", "sessions").unwrap();
        let store_result = tokio_test::block_on(async {
            // Can't test without actual CouchDB, just test config
            config.session_key("test-id")
        });
        assert!(store_result.starts_with("session:"));
    }
}

