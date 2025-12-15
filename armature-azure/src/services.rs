//! Azure services container with dynamic loading.

#[allow(unused_imports)]
use parking_lot::RwLock;
#[allow(unused_imports)]
use std::sync::Arc;
use tracing::info;

#[allow(unused_imports)]
use crate::{AzureConfig, AzureError, CredentialsSource, Result};

/// Container for Azure service clients.
///
/// Services are loaded lazily based on configuration.
/// Only enabled services are initialized.
pub struct AzureServices {
    config: AzureConfig,

    #[cfg(feature = "auth")]
    credential: Arc<dyn azure_core::auth::TokenCredential>,

    #[cfg(feature = "blob")]
    blob_service: RwLock<Option<azure_storage_blobs::prelude::BlobServiceClient>>,

    #[cfg(feature = "queue")]
    queue_service: RwLock<Option<azure_storage_queues::QueueServiceClient>>,

    #[cfg(feature = "cosmos")]
    cosmos: RwLock<Option<azure_data_cosmos::CosmosClient>>,

    #[cfg(feature = "keyvault")]
    keyvault: RwLock<Option<azure_security_keyvault::SecretClient>>,
}

impl AzureServices {
    /// Create a new Azure services container.
    pub async fn new(config: AzureConfig) -> Result<Arc<Self>> {
        #[cfg(feature = "auth")]
        let credential = Self::build_credential(&config).await?;

        info!(
            storage_account = ?config.storage_account,
            services = ?config.enabled_services,
            "Azure services initialized"
        );

        let services = Self {
            config,
            #[cfg(feature = "auth")]
            credential,
            #[cfg(feature = "blob")]
            blob_service: RwLock::new(None),
            #[cfg(feature = "queue")]
            queue_service: RwLock::new(None),
            #[cfg(feature = "cosmos")]
            cosmos: RwLock::new(None),
            #[cfg(feature = "keyvault")]
            keyvault: RwLock::new(None),
        };

        let services = Arc::new(services);

        // Pre-initialize enabled services
        services.initialize_enabled_services().await?;

        Ok(services)
    }

    /// Build Azure credential.
    #[cfg(feature = "auth")]
    async fn build_credential(
        config: &AzureConfig,
    ) -> Result<Arc<dyn azure_core::auth::TokenCredential>> {
        use azure_identity::*;

        match &config.credentials {
            CredentialsSource::DefaultCredential => {
                Ok(Arc::new(DefaultAzureCredential::default()))
            }
            CredentialsSource::Environment => {
                Ok(Arc::new(EnvironmentCredential::default()))
            }
            CredentialsSource::ManagedIdentity => {
                Ok(Arc::new(
                    ImdsManagedIdentityCredential::default(),
                ))
            }
            CredentialsSource::AzureCli => {
                Ok(Arc::new(AzureCliCredential::new()))
            }
            CredentialsSource::ServicePrincipal {
                tenant_id,
                client_id,
                client_secret,
            } => {
                Ok(Arc::new(
                    ClientSecretCredential::new(
                        azure_identity::TokenCredentialOptions::default(),
                        tenant_id.clone(),
                        client_id.clone(),
                        client_secret.clone(),
                    ),
                ))
            }
            CredentialsSource::ConnectionString(_) | CredentialsSource::StorageAccountKey { .. } => {
                // For storage-specific auth, we'll handle it at the client level
                Ok(Arc::new(DefaultAzureCredential::default()))
            }
        }
    }

    /// Initialize all enabled services.
    async fn initialize_enabled_services(&self) -> Result<()> {
        for service in &self.config.enabled_services {
            match service.as_str() {
                #[cfg(feature = "blob")]
                "blob" => {
                    self.init_blob()?;
                }
                #[cfg(feature = "queue")]
                "queue" => {
                    self.init_queue()?;
                }
                #[cfg(feature = "cosmos")]
                "cosmos" => {
                    self.init_cosmos()?;
                }
                #[cfg(feature = "keyvault")]
                "keyvault" => {
                    self.init_keyvault()?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Get the configuration.
    pub fn config(&self) -> &AzureConfig {
        &self.config
    }

    // Service initializers

    #[cfg(feature = "blob")]
    fn init_blob(&self) -> Result<()> {
        use azure_storage_blobs::prelude::*;
        use azure_storage::prelude::*;

        let mut client = self.blob_service.write();
        if client.is_none() {
            let account = self.config.storage_account.as_ref()
                .ok_or(AzureError::StorageAccountNotSpecified)?;

            let storage_credentials = match &self.config.credentials {
                CredentialsSource::ConnectionString(conn_str) => {
                    StorageCredentials::connection_string(conn_str)
                        .map_err(|e| AzureError::Config(e.to_string()))?
                }
                CredentialsSource::StorageAccountKey { account_name: _, account_key } => {
                    StorageCredentials::access_key(account.clone(), account_key.clone())
                }
                _ => {
                    // Use token credential
                    StorageCredentials::token_credential(self.credential.clone())
                }
            };

            let blob_client = if self.config.use_emulator {
                // Azurite emulator
                BlobServiceClient::builder(account, storage_credentials)
                    .blob_service_url(format!("http://127.0.0.1:10000/{}", account))
                    .build()
            } else {
                BlobServiceClient::new(account, storage_credentials)
            };

            *client = Some(blob_client);
            info!(account = %account, "Blob Storage client initialized");
        }
        Ok(())
    }

    #[cfg(feature = "queue")]
    fn init_queue(&self) -> Result<()> {
        use azure_storage_queues::*;
        use azure_storage::prelude::*;

        let mut client = self.queue_service.write();
        if client.is_none() {
            let account = self.config.storage_account.as_ref()
                .ok_or(AzureError::StorageAccountNotSpecified)?;

            let storage_credentials = match &self.config.credentials {
                CredentialsSource::ConnectionString(conn_str) => {
                    StorageCredentials::connection_string(conn_str)
                        .map_err(|e| AzureError::Config(e.to_string()))?
                }
                CredentialsSource::StorageAccountKey { account_name: _, account_key } => {
                    StorageCredentials::access_key(account.clone(), account_key.clone())
                }
                _ => {
                    StorageCredentials::token_credential(self.credential.clone())
                }
            };

            let queue_client = QueueServiceClient::new(account, storage_credentials);
            *client = Some(queue_client);
            info!(account = %account, "Queue Storage client initialized");
        }
        Ok(())
    }

    #[cfg(feature = "cosmos")]
    fn init_cosmos(&self) -> Result<()> {
        use azure_data_cosmos::prelude::*;

        let mut client = self.cosmos.write();
        if client.is_none() {
            let endpoint = self.config.cosmos_endpoint.as_ref()
                .ok_or_else(|| AzureError::Config("Cosmos DB endpoint not specified".to_string()))?;

            let cosmos_client = CosmosClient::new(
                endpoint,
                self.credential.clone(),
                CosmosOptions::default(),
            );

            *client = Some(cosmos_client);
            info!(endpoint = %endpoint, "Cosmos DB client initialized");
        }
        Ok(())
    }

    #[cfg(feature = "keyvault")]
    fn init_keyvault(&self) -> Result<()> {
        use azure_security_keyvault::prelude::*;

        let mut client = self.keyvault.write();
        if client.is_none() {
            let vault_url = self.config.keyvault_url.as_ref()
                .ok_or_else(|| AzureError::Config("Key Vault URL not specified".to_string()))?;

            let kv_client = SecretClient::new(vault_url, self.credential.clone())
                .map_err(|e| AzureError::Config(e.to_string()))?;

            *client = Some(kv_client);
            info!(vault = %vault_url, "Key Vault client initialized");
        }
        Ok(())
    }

    // Service accessors

    /// Get the Blob Service client.
    #[cfg(feature = "blob")]
    pub fn blob_service(&self) -> Result<azure_storage_blobs::prelude::BlobServiceClient> {
        if !self.config.is_enabled("blob") {
            return Err(AzureError::not_configured("blob"));
        }

        self.blob_service
            .read()
            .clone()
            .ok_or_else(|| AzureError::Service("Blob client not initialized".to_string()))
    }

    #[cfg(not(feature = "blob"))]
    pub fn blob_service(&self) -> Result<()> {
        Err(AzureError::not_enabled("blob"))
    }

    /// Get the Queue Service client.
    #[cfg(feature = "queue")]
    pub fn queue_service(&self) -> Result<azure_storage_queues::QueueServiceClient> {
        if !self.config.is_enabled("queue") {
            return Err(AzureError::not_configured("queue"));
        }

        self.queue_service
            .read()
            .clone()
            .ok_or_else(|| AzureError::Service("Queue client not initialized".to_string()))
    }

    #[cfg(not(feature = "queue"))]
    pub fn queue_service(&self) -> Result<()> {
        Err(AzureError::not_enabled("queue"))
    }

    /// Get the Cosmos DB client.
    #[cfg(feature = "cosmos")]
    pub fn cosmos(&self) -> Result<azure_data_cosmos::CosmosClient> {
        if !self.config.is_enabled("cosmos") {
            return Err(AzureError::not_configured("cosmos"));
        }

        self.cosmos
            .read()
            .clone()
            .ok_or_else(|| AzureError::Service("Cosmos client not initialized".to_string()))
    }

    #[cfg(not(feature = "cosmos"))]
    pub fn cosmos(&self) -> Result<()> {
        Err(AzureError::not_enabled("cosmos"))
    }

    /// Get the Key Vault client.
    #[cfg(feature = "keyvault")]
    pub fn keyvault(&self) -> Result<azure_security_keyvault::SecretClient> {
        if !self.config.is_enabled("keyvault") {
            return Err(AzureError::not_configured("keyvault"));
        }

        self.keyvault
            .read()
            .clone()
            .ok_or_else(|| AzureError::Service("Key Vault client not initialized".to_string()))
    }

    #[cfg(not(feature = "keyvault"))]
    pub fn keyvault(&self) -> Result<()> {
        Err(AzureError::not_enabled("keyvault"))
    }
}

