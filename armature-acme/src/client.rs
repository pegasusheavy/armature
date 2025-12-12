/// ACME client implementation
use crate::{challenge::*, config::*, directory::*, error::*};
use std::path::Path;

/// ACME client for certificate management
///
/// This client handles the ACME protocol for obtaining and renewing
/// SSL/TLS certificates from providers like Let's Encrypt.
///
/// # Example
///
/// ```no_run
/// use armature_acme::{AcmeClient, AcmeConfig};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = AcmeConfig::lets_encrypt_staging(
///     vec!["admin@example.com".to_string()],
///     vec!["example.com".to_string()],
/// ).with_accept_tos(true);
///
/// let client = AcmeClient::new(config).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct AcmeClient {
    config: AcmeConfig,
    directory: Directory,
    #[allow(dead_code)]
    http_client: reqwest::Client,
    account_url: Option<String>,
}

impl AcmeClient {
    /// Create a new ACME client
    ///
    /// This initializes the client and fetches the ACME directory.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use armature_acme::{AcmeClient, AcmeConfig};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = AcmeConfig::lets_encrypt_staging(
    ///     vec!["admin@example.com".to_string()],
    ///     vec!["example.com".to_string()],
    /// );
    ///
    /// let client = AcmeClient::new(config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(config: AcmeConfig) -> Result<Self> {
        let http_client = reqwest::Client::builder()
            .use_rustls_tls()
            .build()
            .map_err(|e| AcmeError::Internal(e.to_string()))?;

        let directory = Self::fetch_directory(&http_client, &config.directory_url).await?;

        Ok(Self {
            config,
            directory,
            http_client,
            account_url: None,
        })
    }

    /// Fetch the ACME directory
    async fn fetch_directory(client: &reqwest::Client, url: &str) -> Result<Directory> {
        let response = client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(AcmeError::InvalidDirectory(format!(
                "Failed to fetch directory: {}",
                response.status()
            )));
        }

        let directory = response.json().await?;
        Ok(directory)
    }

    /// Register or retrieve an existing account
    ///
    /// This creates a new ACME account or retrieves an existing one.
    /// The account credentials are stored in the configured account directory.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use armature_acme::{AcmeClient, AcmeConfig};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let config = AcmeConfig::lets_encrypt_staging(
    /// #     vec!["admin@example.com".to_string()],
    /// #     vec!["example.com".to_string()],
    /// # ).with_accept_tos(true);
    /// # let mut client = AcmeClient::new(config).await?;
    /// client.register_account().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn register_account(&mut self) -> Result<()> {
        // In a real implementation, this would:
        // 1. Generate or load an account key pair
        // 2. Create a JWS-signed request to newAccount
        // 3. Store the account URL and credentials

        tracing::info!("Registering ACME account");

        // Placeholder - real implementation would make HTTP requests
        self.account_url = Some(self.directory.new_account.clone());

        Ok(())
    }

    /// Order a new certificate
    ///
    /// This creates a new certificate order for the configured domains.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use armature_acme::{AcmeClient, AcmeConfig};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let config = AcmeConfig::lets_encrypt_staging(
    /// #     vec!["admin@example.com".to_string()],
    /// #     vec!["example.com".to_string()],
    /// # ).with_accept_tos(true);
    /// # let mut client = AcmeClient::new(config).await?;
    /// # client.register_account().await?;
    /// let order_url = client.create_order().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_order(&self) -> Result<String> {
        if self.account_url.is_none() {
            return Err(AcmeError::InvalidAccount(
                "Account not registered".to_string(),
            ));
        }

        tracing::info!(
            "Creating certificate order for domains: {:?}",
            self.config.domains
        );

        // Placeholder - real implementation would:
        // 1. Create OrderCreate request
        // 2. Sign with account key
        // 3. POST to newOrder endpoint
        // 4. Return order URL

        Ok(self.directory.new_order.clone())
    }

    /// Get challenges for an order
    ///
    /// Retrieves the challenges that need to be completed for domain validation.
    pub async fn get_challenges(&self, _order_url: &str) -> Result<Vec<Http01Challenge>> {
        tracing::info!("Fetching challenges");

        // Placeholder - real implementation would:
        // 1. GET the order
        // 2. GET each authorization
        // 3. Extract challenges
        // 4. Return formatted challenge data

        Ok(Vec::new())
    }

    /// Notify ACME server that challenge is ready
    ///
    /// This tells the ACME server to validate the challenge.
    pub async fn notify_challenge_ready(&self, _challenge_url: &str) -> Result<()> {
        tracing::info!("Notifying challenge ready");

        // Placeholder - real implementation would:
        // 1. POST to challenge URL
        // 2. Wait for validation
        // 3. Check status

        Ok(())
    }

    /// Finalize the order and retrieve the certificate
    ///
    /// This generates a CSR, submits it for signing, and retrieves the certificate.
    pub async fn finalize_order(&self, _order_url: &str) -> Result<(String, String)> {
        tracing::info!("Finalizing order");

        // Placeholder - real implementation would:
        // 1. Generate CSR
        // 2. POST to finalize URL
        // 3. Poll for certificate
        // 4. Download certificate
        // 5. Return certificate and private key

        Ok((String::new(), String::new()))
    }

    /// Complete certificate ordering process
    ///
    /// This is a high-level method that handles the entire certificate
    /// ordering process including challenges, validation, and finalization.
    ///
    /// # Returns
    ///
    /// Returns a tuple of (certificate_pem, private_key_pem)
    pub async fn order_certificate(&mut self) -> Result<(String, String)> {
        // Register account if not already registered
        if self.account_url.is_none() {
            self.register_account().await?;
        }

        // Create order
        let order_url = self.create_order().await?;

        // Get challenges
        let _challenges = self.get_challenges(&order_url).await?;

        // Note: In a real implementation, the user would need to:
        // 1. Set up HTTP server for HTTP-01 challenges
        // 2. Add DNS records for DNS-01 challenges
        // 3. Configure TLS for TLS-ALPN-01 challenges

        // Finalize and get certificate
        self.finalize_order(&order_url).await
    }

    /// Check if certificate needs renewal
    ///
    /// Returns true if the certificate should be renewed based on
    /// the configured renewal threshold.
    pub async fn should_renew(&self, cert_path: impl AsRef<Path>) -> Result<bool> {
        let cert_path = cert_path.as_ref();

        if !cert_path.exists() {
            return Ok(true);
        }

        // Placeholder - real implementation would:
        // 1. Load certificate
        // 2. Parse expiration date
        // 3. Check if within renewal window

        Ok(false)
    }

    /// Save certificate and private key to files
    pub async fn save_certificate(
        &self,
        cert_pem: &str,
        key_pem: &str,
    ) -> Result<(String, String)> {
        use std::fs;

        fs::create_dir_all(&self.config.cert_dir)?;

        let cert_path = self.config.cert_dir.join("cert.pem");
        let key_path = self.config.cert_dir.join("key.pem");

        fs::write(&cert_path, cert_pem)?;
        fs::write(&key_path, key_pem)?;

        Ok((
            cert_path.to_string_lossy().to_string(),
            key_path.to_string_lossy().to_string(),
        ))
    }

    /// Get the ACME directory
    pub fn directory(&self) -> &Directory {
        &self.directory
    }

    /// Get the configuration
    pub fn config(&self) -> &AcmeConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_directory_invalid_url() {
        let client = reqwest::Client::new();
        let result = AcmeClient::fetch_directory(&client, "https://invalid.example.com").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_client_config_access() {
        let config = AcmeConfig::lets_encrypt_staging(
            vec!["admin@example.com".to_string()],
            vec!["example.com".to_string()],
        );

        let domains = config.domains.clone();

        // Can't easily test full client creation without network access
        assert_eq!(domains, vec!["example.com"]);
    }
}
