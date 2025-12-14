//! etcd service discovery implementation

use crate::service::{DiscoveryError, ServiceDiscovery, ServiceInstance};
use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose};
use serde_json;
use tracing::{debug, info};

/// etcd service discovery client
pub struct EtcdDiscovery {
    base_url: String,
    prefix: String,
    client: reqwest::Client,
}

impl EtcdDiscovery {
    /// Create new etcd discovery client
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use armature_discovery::EtcdDiscovery;
    ///
    /// let etcd = EtcdDiscovery::new("http://localhost:2379", "/services")?;
    /// ```
    pub fn new(base_url: impl Into<String>, prefix: impl Into<String>) -> Result<Self, DiscoveryError> {
        Ok(Self {
            base_url: base_url.into(),
            prefix: prefix.into(),
            client: reqwest::Client::new(),
        })
    }

    fn service_key(&self, service_id: &str) -> String {
        format!("{}/{}", self.prefix, service_id)
    }

    fn service_name_prefix(&self, service_name: &str) -> String {
        format!("{}/{}/", self.prefix, service_name)
    }
}

#[async_trait]
impl ServiceDiscovery for EtcdDiscovery {
    async fn register(&self, service: &ServiceInstance) -> Result<(), DiscoveryError> {
        let url = format!("{}/v3/kv/put", self.base_url);
        let key = self.service_key(&service.id);
        let value = serde_json::to_string(service)
            .map_err(|e| DiscoveryError::InvalidConfiguration(e.to_string()))?;

        // Base64 encode key and value for etcd v3 API
        let key_b64 = general_purpose::STANDARD.encode(key.as_bytes());
        let value_b64 = general_purpose::STANDARD.encode(value.as_bytes());

        let payload = serde_json::json!({
            "key": key_b64,
            "value": value_b64,
        });

        let response = self.client.post(&url).json(&payload).send().await?;

        if response.status().is_success() {
            info!("Registered service {} with etcd", service.id);
            Ok(())
        } else {
            let error = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(DiscoveryError::RegistrationFailed(error))
        }
    }

    async fn deregister(&self, service_id: &str) -> Result<(), DiscoveryError> {
        let url = format!("{}/v3/kv/deleterange", self.base_url);
        let key = self.service_key(service_id);

        let key_b64 = general_purpose::STANDARD.encode(key.as_bytes());

        let payload = serde_json::json!({
            "key": key_b64,
        });

        let response = self.client.post(&url).json(&payload).send().await?;

        if response.status().is_success() {
            info!("Deregistered service {} from etcd", service_id);
            Ok(())
        } else {
            let error = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(DiscoveryError::DeregistrationFailed(error))
        }
    }

    async fn discover(&self, service_name: &str) -> Result<Vec<ServiceInstance>, DiscoveryError> {
        let url = format!("{}/v3/kv/range", self.base_url);
        let prefix = self.service_name_prefix(service_name);

        let key_b64 = general_purpose::STANDARD.encode(prefix.as_bytes());

        let payload = serde_json::json!({
            "key": key_b64,
            "range_end": general_purpose::STANDARD.encode(format!("{}~", prefix).as_bytes()),
        });

        let response = self.client.post(&url).json(&payload).send().await?;

        if !response.status().is_success() {
            return Err(DiscoveryError::ServiceNotFound(service_name.to_string()));
        }

        #[derive(serde::Deserialize)]
        struct EtcdResponse {
            kvs: Option<Vec<EtcdKV>>,
        }

        #[derive(serde::Deserialize)]
        struct EtcdKV {
            value: String,
        }

        let etcd_response: EtcdResponse = response.json().await?;

        let instances: Result<Vec<ServiceInstance>, _> = etcd_response
            .kvs
            .unwrap_or_default()
            .into_iter()
            .map(|kv| {
                let value_bytes = general_purpose::STANDARD.decode(&kv.value)
                    .map_err(|e| DiscoveryError::InvalidConfiguration(e.to_string()))?;
                let value_str = String::from_utf8(value_bytes)
                    .map_err(|e| DiscoveryError::InvalidConfiguration(e.to_string()))?;
                serde_json::from_str(&value_str)
                    .map_err(|e| DiscoveryError::InvalidConfiguration(e.to_string()))
            })
            .collect();

        let instances = instances?;

        if instances.is_empty() {
            Err(DiscoveryError::ServiceNotFound(service_name.to_string()))
        } else {
            debug!("Discovered {} instances of service {}", instances.len(), service_name);
            Ok(instances)
        }
    }

    async fn get_service(&self, service_id: &str) -> Result<ServiceInstance, DiscoveryError> {
        let url = format!("{}/v3/kv/range", self.base_url);
        let key = self.service_key(service_id);

        let key_b64 = general_purpose::STANDARD.encode(key.as_bytes());

        let payload = serde_json::json!({
            "key": key_b64,
        });

        let response = self.client.post(&url).json(&payload).send().await?;

        if !response.status().is_success() {
            return Err(DiscoveryError::ServiceNotFound(service_id.to_string()));
        }

        #[derive(serde::Deserialize)]
        struct EtcdResponse {
            kvs: Option<Vec<EtcdKV>>,
        }

        #[derive(serde::Deserialize)]
        struct EtcdKV {
            value: String,
        }

        let etcd_response: EtcdResponse = response.json().await?;

        let kv = etcd_response
            .kvs
            .and_then(|mut kvs| kvs.pop())
            .ok_or_else(|| DiscoveryError::ServiceNotFound(service_id.to_string()))?;

        let value_bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &kv.value)
            .map_err(|e| DiscoveryError::InvalidConfiguration(e.to_string()))?;
        let value_str = String::from_utf8(value_bytes)
            .map_err(|e| DiscoveryError::InvalidConfiguration(e.to_string()))?;
        let instance: ServiceInstance = serde_json::from_str(&value_str)
            .map_err(|e| DiscoveryError::InvalidConfiguration(e.to_string()))?;

        Ok(instance)
    }

    async fn list_services(&self) -> Result<Vec<String>, DiscoveryError> {
        // For etcd, we would need to scan all keys under the prefix
        // This is a simplified implementation
        Ok(Vec::new())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_etcd_discovery_creation() {
        let etcd = EtcdDiscovery::new("http://localhost:2379", "/services");
        assert!(etcd.is_ok());
    }
}

