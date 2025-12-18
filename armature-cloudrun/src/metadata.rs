//! Cloud Run instance metadata.

use serde::{Deserialize, Serialize};

/// Instance metadata from the GCE metadata server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceMetadata {
    /// Instance ID.
    pub instance_id: Option<String>,
    /// Instance zone.
    pub zone: Option<String>,
    /// Project ID.
    pub project_id: Option<String>,
    /// Project number.
    pub project_number: Option<String>,
    /// Service account email.
    pub service_account: Option<String>,
    /// Instance region.
    pub region: Option<String>,
}

impl InstanceMetadata {
    /// Fetch instance metadata from the GCE metadata server.
    pub async fn fetch() -> Result<Self, crate::CloudRunError> {
        let client = reqwest::Client::new();

        async fn get_metadata(client: &reqwest::Client, path: &str) -> Option<String> {
            client
                .get(format!(
                    "http://metadata.google.internal/computeMetadata/v1/{}",
                    path
                ))
                .header("Metadata-Flavor", "Google")
                .send()
                .await
                .ok()?
                .text()
                .await
                .ok()
        }

        let zone = get_metadata(&client, "instance/zone").await;
        let region = zone.as_ref().map(|z| {
            // Zone is like "projects/123456789/zones/us-central1-a"
            // Extract region as "us-central1"
            z.rsplit('/')
                .next()
                .and_then(|z| {
                    z.rsplit('-')
                        .skip(1)
                        .collect::<Vec<_>>()
                        .into_iter()
                        .rev()
                        .collect::<Vec<_>>()
                        .join("-")
                        .into()
                })
                .unwrap_or_default()
        });

        Ok(Self {
            instance_id: get_metadata(&client, "instance/id").await,
            zone,
            project_id: get_metadata(&client, "project/project-id").await,
            project_number: get_metadata(&client, "project/numeric-project-id").await,
            service_account: get_metadata(&client, "instance/service-accounts/default/email").await,
            region,
        })
    }

    /// Create from environment variables (fallback when metadata server unavailable).
    pub fn from_env() -> Self {
        Self {
            instance_id: std::env::var("INSTANCE_ID").ok(),
            zone: std::env::var("CLOUD_RUN_ZONE").ok(),
            project_id: std::env::var("GOOGLE_CLOUD_PROJECT").ok(),
            project_number: std::env::var("GOOGLE_CLOUD_PROJECT_NUMBER").ok(),
            service_account: std::env::var("SERVICE_ACCOUNT").ok(),
            region: std::env::var("GOOGLE_CLOUD_REGION").ok(),
        }
    }
}

/// Service metadata for the Cloud Run service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetadata {
    /// Service name.
    pub name: String,
    /// Revision name.
    pub revision: Option<String>,
    /// Configuration name.
    pub configuration: Option<String>,
    /// Project ID.
    pub project_id: Option<String>,
    /// Region.
    pub region: Option<String>,
    /// Service URL.
    pub url: Option<String>,
}

impl ServiceMetadata {
    /// Create from environment variables.
    pub fn from_env() -> Option<Self> {
        let name = std::env::var("K_SERVICE").ok()?;

        Some(Self {
            name,
            revision: std::env::var("K_REVISION").ok(),
            configuration: std::env::var("K_CONFIGURATION").ok(),
            project_id: std::env::var("GOOGLE_CLOUD_PROJECT").ok(),
            region: std::env::var("GOOGLE_CLOUD_REGION").ok(),
            url: std::env::var("CLOUD_RUN_URL").ok(),
        })
    }

    /// Check if running on Cloud Run.
    pub fn is_cloud_run() -> bool {
        std::env::var("K_SERVICE").is_ok()
    }
}
