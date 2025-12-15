//! High-level mailer interface.

use std::sync::Arc;
use tracing::debug;

use crate::{
    Address, Email, MailError, Result, SmtpConfig, SmtpTransport,
    TemplateEngine, Transport,
};

/// Mailer configuration.
#[derive(Debug, Clone)]
pub struct MailerConfig {
    /// Default from address.
    pub default_from: Option<Address>,
    /// Default reply-to address.
    pub default_reply_to: Option<Address>,
    /// Retry count for failed sends.
    pub retry_count: u32,
    /// Retry delay.
    pub retry_delay: std::time::Duration,
}

impl Default for MailerConfig {
    fn default() -> Self {
        Self {
            default_from: None,
            default_reply_to: None,
            retry_count: 3,
            retry_delay: std::time::Duration::from_secs(1),
        }
    }
}

impl MailerConfig {
    /// Set the default from address.
    pub fn from(mut self, from: &str) -> Result<Self> {
        self.default_from = Some(Address::parse(from)?);
        Ok(self)
    }

    /// Set the default reply-to address.
    pub fn reply_to(mut self, reply_to: &str) -> Result<Self> {
        self.default_reply_to = Some(Address::parse(reply_to)?);
        Ok(self)
    }

    /// Set the retry count.
    pub fn retries(mut self, count: u32) -> Self {
        self.retry_count = count;
        self
    }
}

/// High-level mailer for sending emails.
pub struct Mailer {
    transport: Arc<dyn Transport>,
    config: MailerConfig,
    templates: Option<Arc<dyn TemplateEngine>>,
}

impl Mailer {
    /// Create a new mailer with an SMTP transport.
    pub async fn smtp(smtp_config: SmtpConfig) -> Result<Self> {
        let transport = SmtpTransport::new(smtp_config).await?;
        Ok(Self {
            transport: Arc::new(transport),
            config: MailerConfig::default(),
            templates: None,
        })
    }

    /// Create a new mailer with a custom transport.
    pub fn new(transport: impl Transport + 'static) -> Self {
        Self {
            transport: Arc::new(transport),
            config: MailerConfig::default(),
            templates: None,
        }
    }

    /// Set the mailer configuration.
    pub fn with_config(mut self, config: MailerConfig) -> Self {
        self.config = config;
        self
    }

    /// Set the default from address.
    pub fn default_from(mut self, from: &str) -> Result<Self> {
        self.config.default_from = Some(Address::parse(from)?);
        Ok(self)
    }

    /// Set a template engine.
    pub fn with_template_engine(mut self, engine: impl TemplateEngine + 'static) -> Self {
        self.templates = Some(Arc::new(engine));
        self
    }

    /// Load templates from a directory (requires handlebars feature).
    #[cfg(feature = "handlebars")]
    pub fn with_templates(mut self, path: impl AsRef<std::path::Path>) -> Result<Self> {
        let engine = crate::HandlebarsEngine::from_directory(path)?;
        self.templates = Some(Arc::new(engine));
        Ok(self)
    }

    /// Send an email.
    pub async fn send(&self, email: Email) -> Result<()> {
        let email = self.apply_defaults(email);
        self.send_with_retry(&email).await
    }

    /// Send an email using a template.
    pub async fn send_template(
        &self,
        template_name: &str,
        to: &str,
        context: serde_json::Value,
    ) -> Result<()> {
        let templates = self
            .templates
            .as_ref()
            .ok_or_else(|| MailError::Template("No template engine configured".to_string()))?;

        let rendered = templates.render(template_name, &context)?;
        let to_addr = Address::parse(to)?;

        let mut email = Email::new().to(to_addr);

        if let Some(subject) = rendered.subject {
            email = email.subject(subject);
        }
        if let Some(html) = rendered.html {
            email = email.html(html);
        }
        if let Some(text) = rendered.text {
            email = email.text(text);
        }

        self.send(email).await
    }

    /// Send a simple text email.
    pub async fn send_text(
        &self,
        to: &str,
        subject: &str,
        body: &str,
    ) -> Result<()> {
        let email = Email::new()
            .to(to)
            .subject(subject)
            .text(body);

        self.send(email).await
    }

    /// Send a simple HTML email.
    pub async fn send_html(
        &self,
        to: &str,
        subject: &str,
        html: &str,
    ) -> Result<()> {
        let email = Email::new()
            .to(to)
            .subject(subject)
            .html(html);

        self.send(email).await
    }

    /// Send to multiple recipients.
    pub async fn send_bulk(&self, emails: Vec<Email>) -> Vec<Result<()>> {
        let mut results = Vec::with_capacity(emails.len());

        for email in emails {
            results.push(self.send(email).await);
        }

        results
    }

    /// Check if the transport is healthy.
    pub async fn is_healthy(&self) -> bool {
        self.transport.is_healthy().await
    }

    /// Apply default configuration to an email.
    fn apply_defaults(&self, mut email: Email) -> Email {
        if email.from.is_none() {
            email.from = self.config.default_from.clone();
        }
        if email.reply_to.is_none() {
            email.reply_to = self.config.default_reply_to.clone();
        }
        email
    }

    /// Send with retry logic.
    async fn send_with_retry(&self, email: &Email) -> Result<()> {
        let mut last_error = None;

        for attempt in 0..=self.config.retry_count {
            if attempt > 0 {
                debug!(attempt, "Retrying email send");
                tokio::time::sleep(self.config.retry_delay).await;
            }

            match self.transport.send(email).await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    if !e.is_retryable() || attempt >= self.config.retry_count {
                        return Err(e);
                    }
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            MailError::Smtp("Unknown error after retries".to_string())
        }))
    }
}

/// Convenience functions for quick email sending.
impl Mailer {
    /// Create a welcome email.
    pub fn welcome_email(
        to: &str,
        name: &str,
        activation_link: &str,
    ) -> Email {
        Email::new()
            .to(to)
            .subject("Welcome! Please activate your account")
            .html(format!(
                r#"
                <h1>Welcome, {}!</h1>
                <p>Thank you for signing up. Please click the link below to activate your account:</p>
                <p><a href="{}">Activate Account</a></p>
                "#,
                name, activation_link
            ))
            .text(format!(
                "Welcome, {}!\n\nPlease visit this link to activate your account: {}",
                name, activation_link
            ))
    }

    /// Create a password reset email.
    pub fn password_reset_email(
        to: &str,
        reset_link: &str,
        expires_in: &str,
    ) -> Email {
        Email::new()
            .to(to)
            .subject("Password Reset Request")
            .html(format!(
                r#"
                <h1>Password Reset</h1>
                <p>We received a request to reset your password.</p>
                <p><a href="{}">Reset Password</a></p>
                <p>This link will expire in {}.</p>
                <p>If you didn't request this, please ignore this email.</p>
                "#,
                reset_link, expires_in
            ))
            .text(format!(
                "Password Reset\n\nVisit this link to reset your password: {}\n\nThis link will expire in {}.\n\nIf you didn't request this, please ignore this email.",
                reset_link, expires_in
            ))
    }

    /// Create a notification email.
    pub fn notification_email(
        to: &str,
        title: &str,
        message: &str,
    ) -> Email {
        Email::new()
            .to(to)
            .subject(title)
            .text(message)
            .html(format!("<h2>{}</h2><p>{}</p>", title, message))
    }
}

