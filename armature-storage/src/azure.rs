//! Azure Blob Storage backend.

use async_trait::async_trait;
use azure_storage::prelude::*;
use azure_storage_blobs::prelude::*;
use bytes::Bytes;
use futures::StreamExt;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info};

use crate::{
    calculate_checksum, generate_unique_key, Result, Storage, StorageConfig, StorageError,
    StorageMetadata, UploadedFile,
};

/// Azure Blob Storage configuration.
#[derive(Debug, Clone)]
pub struct AzureBlobConfig {
    /// Storage account name.
    pub account: String,
    /// Container name.
    pub container: String,
    /// Access key (if not using default credentials).
    pub access_key: Option<String>,
    /// Connection string (alternative to account/key).
    pub connection_string: Option<String>,
    /// Custom endpoint (for Azurite emulator).
    pub endpoint: Option<String>,
    /// Use Azurite emulator.
    pub use_emulator: bool,
    /// Common storage configuration.
    pub storage: StorageConfig,
    /// SAS token duration.
    pub sas_duration: Duration,
}

impl Default for AzureBlobConfig {
    fn default() -> Self {
        Self {
            account: String::new(),
            container: String::new(),
            access_key: None,
            connection_string: None,
            endpoint: None,
            use_emulator: false,
            storage: StorageConfig::default(),
            sas_duration: Duration::from_secs(3600), // 1 hour
        }
    }
}

impl AzureBlobConfig {
    /// Create configuration for a container.
    pub fn new(account: impl Into<String>, container: impl Into<String>) -> Self {
        Self {
            account: account.into(),
            container: container.into(),
            ..Default::default()
        }
    }

    /// Set the access key.
    pub fn access_key(mut self, key: impl Into<String>) -> Self {
        self.access_key = Some(key.into());
        self
    }

    /// Set the connection string.
    pub fn connection_string(mut self, conn_str: impl Into<String>) -> Self {
        self.connection_string = Some(conn_str.into());
        self
    }

    /// Set a custom endpoint.
    pub fn endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = Some(endpoint.into());
        self
    }

    /// Use Azurite emulator.
    pub fn emulator(mut self) -> Self {
        self.use_emulator = true;
        self
    }

    /// Set the path prefix.
    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.storage.path_prefix = Some(prefix.into());
        self
    }

    /// Set SAS token duration.
    pub fn sas_duration(mut self, duration: Duration) -> Self {
        self.sas_duration = duration;
        self
    }
}

/// Azure Blob Storage backend.
pub struct AzureBlobStorage {
    container_client: ContainerClient,
    config: AzureBlobConfig,
}

impl AzureBlobStorage {
    /// Create a new Azure Blob storage backend.
    pub async fn new(config: AzureBlobConfig) -> Result<Self> {
        let storage_credentials = if config.use_emulator {
            // Azurite default credentials
            StorageCredentials::access_key(
                "devstoreaccount1",
                "Eby8vdM02xNOcqFlqUwJPLlmEtlCDXJ1OUzFT50uSRZ6IFsuFq2UVErCz4I6tq/K1SZFPTOtr/KBHBeksoGMGw==",
            )
        } else if let Some(conn_str) = &config.connection_string {
            StorageCredentials::connection_string(conn_str)
                .map_err(|e| StorageError::Config(e.to_string()))?
        } else if let Some(key) = &config.access_key {
            StorageCredentials::access_key(&config.account, key)
        } else {
            // Use default Azure credential
            StorageCredentials::token_credential(Arc::new(
                azure_identity::DefaultAzureCredential::default(),
            ))
        };

        let blob_service = if config.use_emulator {
            BlobServiceClient::builder(&config.account, storage_credentials)
                .blob_service_url("http://127.0.0.1:10000/devstoreaccount1")
                .build()
        } else if let Some(endpoint) = &config.endpoint {
            BlobServiceClient::builder(&config.account, storage_credentials)
                .blob_service_url(endpoint)
                .build()
        } else {
            BlobServiceClient::new(&config.account, storage_credentials)
        };

        let container_client = blob_service.container_client(&config.container);

        info!(
            account = %config.account,
            container = %config.container,
            "Initialized Azure Blob storage"
        );

        Ok(Self {
            container_client,
            config,
        })
    }

    /// Create from armature-azure services.
    pub fn from_azure_services(
        services: &Arc<armature_azure::AzureServices>,
        config: AzureBlobConfig,
    ) -> Result<Self> {
        let blob_service = services
            .blob_service()
            .map_err(|e| StorageError::Config(e.to_string()))?;
        let container_client = blob_service.container_client(&config.container);

        Ok(Self {
            container_client,
            config,
        })
    }

    /// Get the full blob name for a path.
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
        if self.config.use_emulator {
            format!(
                "http://127.0.0.1:10000/devstoreaccount1/{}/{}",
                self.config.container, full_key
            )
        } else {
            format!(
                "https://{}.blob.core.windows.net/{}/{}",
                self.config.account, self.config.container, full_key
            )
        }
    }

    /// Get a blob client for a key.
    fn blob_client(&self, key: &str) -> BlobClient {
        let full_key = self.full_key(key);
        self.container_client.blob_client(&full_key)
    }
}

#[async_trait]
impl Storage for AzureBlobStorage {
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

        let size = data.len() as u64;

        // Calculate checksum if enabled
        let checksum = if self.config.storage.calculate_checksum {
            Some(calculate_checksum(&data))
        } else {
            None
        };

        // Upload blob
        let blob_client = self.blob_client(key);

        blob_client
            .put_block_blob(data)
            .content_type(content_type)
            .await
            .map_err(|e| StorageError::Storage(e.to_string()))?;

        debug!(
            key = %key,
            container = %self.config.container,
            size = size,
            "Uploaded to Azure Blob"
        );

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
        let blob_client = self.blob_client(key);

        let response = blob_client.get_content().await.map_err(|e| {
            let err_str = e.to_string();
            if err_str.contains("BlobNotFound") || err_str.contains("404") {
                StorageError::NotFound(key.to_string())
            } else {
                StorageError::Storage(err_str)
            }
        })?;

        Ok(Bytes::from(response))
    }

    async fn head(&self, key: &str) -> Result<StorageMetadata> {
        let blob_client = self.blob_client(key);

        let properties = blob_client.get_properties().await.map_err(|e| {
            let err_str = e.to_string();
            if err_str.contains("BlobNotFound") || err_str.contains("404") {
                StorageError::NotFound(key.to_string())
            } else {
                StorageError::Storage(err_str)
            }
        })?;

        let size = properties.blob.properties.content_length;
        let mut metadata = StorageMetadata::new(key, size).with_url(self.public_url(key));

        if let Some(ct) = &properties.blob.properties.content_type {
            metadata = metadata.with_content_type(ct);
        }

        if let Some(md5) = &properties.blob.properties.content_md5 {
            metadata = metadata.with_checksum(&base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                md5.as_slice(),
            ));
        }

        Ok(metadata)
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let blob_client = self.blob_client(key);

        blob_client
            .delete()
            .await
            .map_err(|e| StorageError::Storage(e.to_string()))?;

        debug!(
            key = %key,
            container = %self.config.container,
            "Deleted from Azure Blob"
        );
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

        let mut results = Vec::new();
        let mut stream = self
            .container_client
            .list_blobs()
            .prefix(&full_prefix)
            .into_stream();

        while let Some(response) = stream.next().await {
            let response = response.map_err(|e| StorageError::Storage(e.to_string()))?;

            for blob in response.blobs.blobs() {
                // Remove prefix to get the relative key
                let relative_key = if let Some(p) = &self.config.storage.path_prefix {
                    blob.name
                        .strip_prefix(&format!("{}/", p))
                        .unwrap_or(&blob.name)
                        .to_string()
                } else {
                    blob.name.clone()
                };

                let size = blob.properties.content_length;
                let mut metadata = StorageMetadata::new(&relative_key, size)
                    .with_url(self.public_url(&relative_key));

                if let Some(ct) = &blob.properties.content_type {
                    metadata = metadata.with_content_type(ct);
                }

                if let Some(md5) = &blob.properties.content_md5 {
                    metadata = metadata.with_checksum(&base64::Engine::encode(
                        &base64::engine::general_purpose::STANDARD,
                        md5.as_slice(),
                    ));
                }

                results.push(metadata);
            }
        }

        Ok(results)
    }

    async fn copy(&self, from: &str, to: &str) -> Result<StorageMetadata> {
        let from_client = self.blob_client(from);
        let to_client = self.blob_client(to);

        let source_url = from_client
            .url()
            .map_err(|e| StorageError::Storage(e.to_string()))?;

        to_client
            .copy(&source_url)
            .await
            .map_err(|e| StorageError::Storage(e.to_string()))?;

        self.head(to).await
    }

    async fn url(&self, key: &str) -> Result<Option<String>> {
        Ok(Some(self.public_url(key)))
    }

    async fn temporary_url(&self, key: &str, expires_in: Duration) -> Result<Option<String>> {
        // Azure SAS token generation requires account key
        // For now, return the public URL
        // Full SAS implementation would require storing the account key
        Ok(Some(self.public_url(key)))
    }
}
