//! Google Cloud Storage backend.

use async_trait::async_trait;
use bytes::Bytes;
use google_cloud_storage::client::Client;
use google_cloud_storage::http::objects::{
    delete::DeleteObjectRequest,
    download::Range,
    get::GetObjectRequest,
    list::ListObjectsRequest,
    upload::{Media, UploadObjectRequest, UploadType},
};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info};

use crate::{
    calculate_checksum, generate_unique_key, Result, Storage, StorageConfig, StorageError,
    StorageMetadata, UploadedFile,
};

/// Google Cloud Storage configuration.
#[derive(Debug, Clone)]
pub struct GcsConfig {
    /// GCS bucket name.
    pub bucket: String,
    /// GCP project ID (optional, uses default).
    pub project_id: Option<String>,
    /// Custom endpoint (for emulators).
    pub endpoint: Option<String>,
    /// Make uploaded objects publicly readable.
    pub public_access: bool,
    /// Common storage configuration.
    pub storage: StorageConfig,
    /// Signed URL duration.
    pub signed_url_duration: Duration,
}

impl Default for GcsConfig {
    fn default() -> Self {
        Self {
            bucket: String::new(),
            project_id: None,
            endpoint: None,
            public_access: false,
            storage: StorageConfig::default(),
            signed_url_duration: Duration::from_secs(3600), // 1 hour
        }
    }
}

impl GcsConfig {
    /// Create configuration for a bucket.
    pub fn new(bucket: impl Into<String>) -> Self {
        Self {
            bucket: bucket.into(),
            ..Default::default()
        }
    }

    /// Set the project ID.
    pub fn project_id(mut self, project_id: impl Into<String>) -> Self {
        self.project_id = Some(project_id.into());
        self
    }

    /// Set a custom endpoint (for emulators).
    pub fn endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = Some(endpoint.into());
        self
    }

    /// Enable public access for uploaded objects.
    pub fn public_access(mut self, public: bool) -> Self {
        self.public_access = public;
        self
    }

    /// Set the path prefix.
    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.storage.path_prefix = Some(prefix.into());
        self
    }

    /// Set signed URL duration.
    pub fn signed_url_duration(mut self, duration: Duration) -> Self {
        self.signed_url_duration = duration;
        self
    }
}

/// Google Cloud Storage backend.
pub struct GcsStorage {
    client: Client,
    config: GcsConfig,
}

impl GcsStorage {
    /// Create a new GCS storage backend.
    pub async fn new(config: GcsConfig) -> Result<Self> {
        use google_cloud_storage::client::ClientConfig;

        let client_config = ClientConfig::default()
            .with_auth()
            .await
            .map_err(|e| StorageError::Config(e.to_string()))?;

        let client = Client::new(client_config);

        info!(bucket = %config.bucket, "Initialized GCS storage");

        Ok(Self { client, config })
    }

    /// Create from an existing GCP client.
    pub fn from_client(client: Client, config: GcsConfig) -> Self {
        Self { client, config }
    }

    /// Create from armature-gcp services.
    pub fn from_gcp_services(
        services: &Arc<armature_gcp::GcpServices>,
        config: GcsConfig,
    ) -> Result<Self> {
        let client = services
            .storage()
            .map_err(|e| StorageError::Config(e.to_string()))?;
        Ok(Self { client, config })
    }

    /// Get the full GCS object name for a path.
    fn full_key(&self, key: &str) -> String {
        if let Some(prefix) = &self.config.storage.path_prefix {
            format!("{}/{}", prefix.trim_end_matches('/'), key)
        } else {
            key.to_string()
        }
    }

    /// Generate a key for a file.
    fn generate_key(&self, original_name: Option<&str>) -> String {
        if self.config.storage.generate_unique_names {
            generate_unique_key(original_name, self.config.storage.preserve_extension)
        } else {
            original_name
                .map(String::from)
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string())
        }
    }

    /// Get the public URL for a key.
    pub fn public_url(&self, key: &str) -> String {
        let full_key = self.full_key(key);
        format!(
            "https://storage.googleapis.com/{}/{}",
            self.config.bucket, full_key
        )
    }
}

#[async_trait]
impl Storage for GcsStorage {
    async fn put(&self, key: &str, data: Bytes) -> Result<StorageMetadata> {
        self.put_with_content_type(key, data, "application/octet-stream")
            .await
    }

    async fn put_with_content_type(
        &self,
        key: &str,
        data: Bytes,
        content_type: &str,
    ) -> Result<StorageMetadata> {
        // Check size limit
        if let Some(max_size) = self.config.storage.max_file_size {
            if data.len() as u64 > max_size {
                return Err(StorageError::TooLarge {
                    size: data.len() as u64,
                    limit: max_size,
                });
            }
        }

        let full_key = self.full_key(key);
        let size = data.len() as u64;

        // Calculate checksum if enabled
        let checksum = if self.config.storage.calculate_checksum {
            Some(calculate_checksum(&data))
        } else {
            None
        };

        // Upload object
        let upload_type = UploadType::Simple(Media::new(full_key.clone()));
        let request = UploadObjectRequest {
            bucket: self.config.bucket.clone(),
            ..Default::default()
        };

        self.client
            .upload_object(&request, data.to_vec(), &upload_type)
            .await
            .map_err(|e| StorageError::Storage(e.to_string()))?;

        debug!(key = %key, bucket = %self.config.bucket, size = size, "Uploaded to GCS");

        // Build metadata
        let mut metadata = StorageMetadata::new(key, size)
            .with_content_type(content_type)
            .with_url(self.public_url(key));

        if let Some(checksum) = checksum {
            metadata = metadata.with_checksum(checksum);
        }

        Ok(metadata)
    }

    async fn put_file(&self, file: &UploadedFile) -> Result<StorageMetadata> {
        let key = self.generate_key(file.name());
        let content_type = file
            .content_type_str()
            .unwrap_or_else(|| "application/octet-stream".to_string());

        let mut metadata = self
            .put_with_content_type(&key, file.data.clone(), &content_type)
            .await?;

        if let Some(name) = file.name() {
            metadata = metadata.with_original_name(name);
        }

        Ok(metadata)
    }

    async fn get(&self, key: &str) -> Result<Bytes> {
        let full_key = self.full_key(key);

        let data = self
            .client
            .download_object(
                &GetObjectRequest {
                    bucket: self.config.bucket.clone(),
                    object: full_key.clone(),
                    ..Default::default()
                },
                &Range::default(),
            )
            .await
            .map_err(|e| {
                let err_str = e.to_string();
                if err_str.contains("404") || err_str.contains("not found") {
                    StorageError::NotFound(key.to_string())
                } else {
                    StorageError::Storage(err_str)
                }
            })?;

        Ok(Bytes::from(data))
    }

    async fn head(&self, key: &str) -> Result<StorageMetadata> {
        let full_key = self.full_key(key);

        let object = self
            .client
            .get_object(&GetObjectRequest {
                bucket: self.config.bucket.clone(),
                object: full_key.clone(),
                ..Default::default()
            })
            .await
            .map_err(|e| {
                let err_str = e.to_string();
                if err_str.contains("404") || err_str.contains("not found") {
                    StorageError::NotFound(key.to_string())
                } else {
                    StorageError::Storage(err_str)
                }
            })?;

        let size = object.size as u64;
        let mut metadata = StorageMetadata::new(key, size).with_url(self.public_url(key));

        if let Some(ct) = &object.content_type {
            metadata = metadata.with_content_type(ct);
        }

        if let Some(md5) = &object.md5_hash {
            metadata = metadata.with_checksum(md5);
        }

        Ok(metadata)
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let full_key = self.full_key(key);

        self.client
            .delete_object(&DeleteObjectRequest {
                bucket: self.config.bucket.clone(),
                object: full_key.clone(),
                ..Default::default()
            })
            .await
            .map_err(|e| StorageError::Storage(e.to_string()))?;

        debug!(key = %key, bucket = %self.config.bucket, "Deleted from GCS");
        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        match self.head(key).await {
            Ok(_) => Ok(true),
            Err(StorageError::NotFound(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }

    async fn list(&self, prefix: Option<&str>) -> Result<Vec<StorageMetadata>> {
        let mut full_prefix = String::new();
        if let Some(p) = &self.config.storage.path_prefix {
            full_prefix.push_str(p);
            full_prefix.push('/');
        }
        if let Some(p) = prefix {
            full_prefix.push_str(p);
        }

        let request = ListObjectsRequest {
            bucket: self.config.bucket.clone(),
            prefix: if full_prefix.is_empty() {
                None
            } else {
                Some(full_prefix.clone())
            },
            ..Default::default()
        };

        let response = self
            .client
            .list_objects(&request)
            .await
            .map_err(|e| StorageError::Storage(e.to_string()))?;

        let mut results = Vec::new();

        for object in response.items.unwrap_or_default() {
            // Remove prefix to get the relative key
            let relative_key = if let Some(p) = &self.config.storage.path_prefix {
                object
                    .name
                    .strip_prefix(&format!("{}/", p))
                    .unwrap_or(&object.name)
                    .to_string()
            } else {
                object.name.clone()
            };

            let size = object.size as u64;
            let mut metadata =
                StorageMetadata::new(&relative_key, size).with_url(self.public_url(&relative_key));

            if let Some(ct) = &object.content_type {
                metadata = metadata.with_content_type(ct);
            }

            if let Some(md5) = &object.md5_hash {
                metadata = metadata.with_checksum(md5);
            }

            results.push(metadata);
        }

        Ok(results)
    }

    async fn copy(&self, from: &str, to: &str) -> Result<StorageMetadata> {
        let from_key = self.full_key(from);
        let to_key = self.full_key(to);

        use google_cloud_storage::http::objects::copy::CopyObjectRequest;

        self.client
            .copy_object(&CopyObjectRequest {
                source_bucket: self.config.bucket.clone(),
                source_object: from_key,
                destination_bucket: self.config.bucket.clone(),
                destination_object: to_key,
                ..Default::default()
            })
            .await
            .map_err(|e| StorageError::Storage(e.to_string()))?;

        self.head(to).await
    }

    async fn url(&self, key: &str) -> Result<Option<String>> {
        Ok(Some(self.public_url(key)))
    }

    async fn temporary_url(&self, key: &str, expires_in: Duration) -> Result<Option<String>> {
        let full_key = self.full_key(key);

        use google_cloud_storage::sign::{SignedURLMethod, SignedURLOptions};

        let url = self
            .client
            .signed_url(
                &self.config.bucket,
                &full_key,
                None,
                None,
                SignedURLOptions {
                    method: SignedURLMethod::GET,
                    expires: expires_in,
                    ..Default::default()
                },
            )
            .await
            .map_err(|e| StorageError::Storage(e.to_string()))?;

        Ok(Some(url))
    }
}
