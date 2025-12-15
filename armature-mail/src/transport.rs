//! Email transport implementations.

use async_trait::async_trait;
use lettre::{
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
};
use std::time::Duration;
use tracing::{debug, info};

use crate::{Email, MailError, Result};

/// Email transport trait.
#[async_trait]
pub trait Transport: Send + Sync {
    /// Send an email.
    async fn send(&self, email: &Email) -> Result<()>;

    /// Check if the transport is healthy.
    async fn is_healthy(&self) -> bool {
        true
    }
}

/// SMTP security mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub enum SmtpSecurity {
    /// No encryption (port 25, not recommended).
    None,
    /// STARTTLS upgrade (port 587).
    #[default]
    StartTls,
    /// Implicit TLS (port 465).
    Tls,
}


/// SMTP configuration.
#[derive(Debug, Clone)]
pub struct SmtpConfig {
    /// SMTP server host.
    pub host: String,
    /// SMTP server port.
    pub port: u16,
    /// Security mode.
    pub security: SmtpSecurity,
    /// Username for authentication.
    pub username: Option<String>,
    /// Password for authentication.
    pub password: Option<String>,
    /// Connection timeout.
    pub timeout: Duration,
    /// Maximum connections in pool.
    pub pool_size: u32,
}

impl SmtpConfig {
    /// Create a new SMTP configuration.
    pub fn new(host: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            port: 587,
            security: SmtpSecurity::StartTls,
            username: None,
            password: None,
            timeout: Duration::from_secs(30),
            pool_size: 4,
        }
    }

    /// Set credentials.
    pub fn credentials(mut self, username: impl Into<String>, password: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self.password = Some(password.into());
        self
    }

    /// Set the port.
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Use STARTTLS security (port 587).
    pub fn starttls(mut self) -> Self {
        self.security = SmtpSecurity::StartTls;
        self.port = 587;
        self
    }

    /// Use implicit TLS security (port 465).
    pub fn tls(mut self) -> Self {
        self.security = SmtpSecurity::Tls;
        self.port = 465;
        self
    }

    /// Use no encryption (not recommended).
    pub fn insecure(mut self) -> Self {
        self.security = SmtpSecurity::None;
        self.port = 25;
        self
    }

    /// Set the connection timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the connection pool size.
    pub fn pool_size(mut self, size: u32) -> Self {
        self.pool_size = size;
        self
    }

    /// Create configuration for common providers.
    pub fn gmail(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self::new("smtp.gmail.com")
            .port(587)
            .starttls()
            .credentials(username, password)
    }

    /// Create configuration for Outlook/Office365.
    pub fn outlook(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self::new("smtp.office365.com")
            .port(587)
            .starttls()
            .credentials(username, password)
    }

    /// Create configuration for Amazon SES.
    pub fn amazon_ses(
        region: &str,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        Self::new(format!("email-smtp.{}.amazonaws.com", region))
            .port(587)
            .starttls()
            .credentials(username, password)
    }

    /// Create configuration for Mailgun.
    pub fn mailgun(domain: &str, api_key: impl Into<String>) -> Self {
        Self::new("smtp.mailgun.org")
            .port(587)
            .starttls()
            .credentials(format!("postmaster@{}", domain), api_key)
    }

    /// Create configuration for SendGrid.
    pub fn sendgrid(api_key: impl Into<String>) -> Self {
        Self::new("smtp.sendgrid.net")
            .port(587)
            .starttls()
            .credentials("apikey", api_key)
    }

    /// Create configuration for Postmark.
    pub fn postmark(api_key: impl Into<String>) -> Self {
        let key = api_key.into();
        Self::new("smtp.postmarkapp.com")
            .port(587)
            .starttls()
            .credentials(key.clone(), key)
    }
}

/// SMTP transport.
pub struct SmtpTransport {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    config: SmtpConfig,
}

impl SmtpTransport {
    /// Create a new SMTP transport.
    pub async fn new(config: SmtpConfig) -> Result<Self> {
        let mut builder = match config.security {
            SmtpSecurity::None => {
                AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.host)
            }
            SmtpSecurity::StartTls => {
                AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.host)?
            }
            SmtpSecurity::Tls => {
                AsyncSmtpTransport::<Tokio1Executor>::relay(&config.host)?
            }
        };

        builder = builder
            .port(config.port)
            .timeout(Some(config.timeout));

        if let (Some(username), Some(password)) = (&config.username, &config.password) {
            builder = builder.credentials(Credentials::new(username.clone(), password.clone()));
        }

        let transport = builder.build();

        info!(
            host = %config.host,
            port = config.port,
            security = ?config.security,
            "SMTP transport initialized"
        );

        Ok(Self { transport, config })
    }

    /// Get the configuration.
    pub fn config(&self) -> &SmtpConfig {
        &self.config
    }

    /// Test the SMTP connection.
    pub async fn test_connection(&self) -> Result<bool> {
        self.transport
            .test_connection()
            .await
            .map_err(MailError::from)
    }
}

#[async_trait]
impl Transport for SmtpTransport {
    async fn send(&self, email: &Email) -> Result<()> {
        let message = email.to_lettre()?;

        debug!(
            to = ?email.to.iter().map(|a| &a.email).collect::<Vec<_>>(),
            subject = ?email.subject,
            "Sending email via SMTP"
        );

        self.transport.send(message).await?;

        debug!("Email sent successfully");
        Ok(())
    }

    async fn is_healthy(&self) -> bool {
        self.test_connection().await.unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smtp_config_builder() {
        let config = SmtpConfig::new("smtp.example.com")
            .port(587)
            .starttls()
            .credentials("user", "pass");

        assert_eq!(config.host, "smtp.example.com");
        assert_eq!(config.port, 587);
        assert_eq!(config.security, SmtpSecurity::StartTls);
        assert_eq!(config.username.as_deref(), Some("user"));
    }

    #[test]
    fn test_provider_configs() {
        let gmail = SmtpConfig::gmail("user@gmail.com", "pass");
        assert_eq!(gmail.host, "smtp.gmail.com");

        let sendgrid = SmtpConfig::sendgrid("api-key");
        assert_eq!(sendgrid.host, "smtp.sendgrid.net");
        assert_eq!(sendgrid.username.as_deref(), Some("apikey"));
    }
}

