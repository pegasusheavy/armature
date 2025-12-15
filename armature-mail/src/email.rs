//! Email message types.

use serde::{Deserialize, Serialize};
use crate::{Address, Attachment, IntoAddress, MailError, Result};

/// Email message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email {
    /// Sender address.
    pub from: Option<Address>,
    /// Reply-to address.
    pub reply_to: Option<Address>,
    /// To recipients.
    pub to: Vec<Address>,
    /// CC recipients.
    pub cc: Vec<Address>,
    /// BCC recipients.
    pub bcc: Vec<Address>,
    /// Email subject.
    pub subject: Option<String>,
    /// Plain text body.
    pub text: Option<String>,
    /// HTML body.
    pub html: Option<String>,
    /// Attachments.
    pub attachments: Vec<Attachment>,
    /// Custom headers.
    pub headers: Vec<(String, String)>,
    /// Message ID.
    pub message_id: Option<String>,
    /// References (for threading).
    pub references: Vec<String>,
    /// In-Reply-To header.
    pub in_reply_to: Option<String>,
    /// Priority (1-5, 1 highest).
    pub priority: Option<u8>,
}

impl Email {
    /// Create a new empty email.
    pub fn new() -> Self {
        Self {
            from: None,
            reply_to: None,
            to: Vec::new(),
            cc: Vec::new(),
            bcc: Vec::new(),
            subject: None,
            text: None,
            html: None,
            attachments: Vec::new(),
            headers: Vec::new(),
            message_id: None,
            references: Vec::new(),
            in_reply_to: None,
            priority: None,
        }
    }

    /// Create a builder.
    pub fn builder() -> EmailBuilder {
        EmailBuilder::new()
    }

    /// Set the from address.
    pub fn from(mut self, from: impl IntoAddress) -> Self {
        self.from = from.into_address().ok();
        self
    }

    /// Set the reply-to address.
    pub fn reply_to(mut self, reply_to: impl IntoAddress) -> Self {
        self.reply_to = reply_to.into_address().ok();
        self
    }

    /// Add a to recipient.
    pub fn to(mut self, to: impl IntoAddress) -> Self {
        if let Ok(addr) = to.into_address() {
            self.to.push(addr);
        }
        self
    }

    /// Add multiple to recipients.
    pub fn to_many<I, A>(mut self, recipients: I) -> Self
    where
        I: IntoIterator<Item = A>,
        A: IntoAddress,
    {
        for r in recipients {
            if let Ok(addr) = r.into_address() {
                self.to.push(addr);
            }
        }
        self
    }

    /// Add a CC recipient.
    pub fn cc(mut self, cc: impl IntoAddress) -> Self {
        if let Ok(addr) = cc.into_address() {
            self.cc.push(addr);
        }
        self
    }

    /// Add a BCC recipient.
    pub fn bcc(mut self, bcc: impl IntoAddress) -> Self {
        if let Ok(addr) = bcc.into_address() {
            self.bcc.push(addr);
        }
        self
    }

    /// Set the subject.
    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    /// Set the plain text body.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Set the HTML body.
    pub fn html(mut self, html: impl Into<String>) -> Self {
        self.html = Some(html.into());
        self
    }

    /// Add an attachment.
    pub fn attach(mut self, attachment: Attachment) -> Self {
        self.attachments.push(attachment);
        self
    }

    /// Add a custom header.
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    /// Set the message ID.
    pub fn message_id(mut self, id: impl Into<String>) -> Self {
        self.message_id = Some(id.into());
        self
    }

    /// Set the in-reply-to header (for threading).
    pub fn in_reply_to(mut self, id: impl Into<String>) -> Self {
        self.in_reply_to = Some(id.into());
        self
    }

    /// Add a reference (for threading).
    pub fn reference(mut self, id: impl Into<String>) -> Self {
        self.references.push(id.into());
        self
    }

    /// Set the priority (1-5, 1 being highest).
    pub fn priority(mut self, priority: u8) -> Self {
        self.priority = Some(priority.clamp(1, 5));
        self
    }

    /// Set high priority.
    pub fn high_priority(self) -> Self {
        self.priority(1)
    }

    /// Set low priority.
    pub fn low_priority(self) -> Self {
        self.priority(5)
    }

    /// Validate the email.
    pub fn validate(&self) -> Result<()> {
        if self.from.is_none() {
            return Err(MailError::MissingField("from"));
        }
        if self.to.is_empty() && self.cc.is_empty() && self.bcc.is_empty() {
            return Err(MailError::MissingField("to/cc/bcc"));
        }
        if self.subject.is_none() {
            return Err(MailError::MissingField("subject"));
        }
        if self.text.is_none() && self.html.is_none() {
            return Err(MailError::MissingField("text/html body"));
        }
        Ok(())
    }

    /// Build a lettre message.
    pub(crate) fn to_lettre(&self) -> Result<lettre::Message> {
        self.validate()?;

        let from = self.from.as_ref().unwrap().to_mailbox()?;

        let mut builder = lettre::Message::builder()
            .from(from)
            .subject(self.subject.as_deref().unwrap_or_default());

        // Add recipients
        for addr in &self.to {
            builder = builder.to(addr.to_mailbox()?);
        }
        for addr in &self.cc {
            builder = builder.cc(addr.to_mailbox()?);
        }
        for addr in &self.bcc {
            builder = builder.bcc(addr.to_mailbox()?);
        }

        // Add reply-to
        if let Some(reply_to) = &self.reply_to {
            builder = builder.reply_to(reply_to.to_mailbox()?);
        }

        // Add message ID
        if let Some(msg_id) = &self.message_id {
            builder = builder.message_id(Some(msg_id.clone()));
        }

        // Add in-reply-to
        if let Some(in_reply_to) = &self.in_reply_to {
            builder = builder.in_reply_to(in_reply_to.clone());
        }

        // Add references
        for reference in &self.references {
            builder = builder.references(reference.clone());
        }

        // Build body
        let body = match (&self.html, &self.text) {
            (Some(html), Some(text)) => {
                lettre::message::MultiPart::alternative_plain_html(text.clone(), html.clone())
            }
            (Some(html), None) => {
                lettre::message::MultiPart::alternative_plain_html(String::new(), html.clone())
            }
            (None, Some(text)) => {
                lettre::message::MultiPart::alternative_plain_html(text.clone(), String::new())
            }
            (None, None) => unreachable!(), // Validated above
        };

        // Add attachments if any
        let body = if self.attachments.is_empty() {
            body
        } else {
            let mut mixed = lettre::message::MultiPart::mixed().multipart(body);
            for attachment in &self.attachments {
                let content_type = attachment
                    .content_type
                    .parse()
                    .unwrap_or(lettre::message::header::ContentType::TEXT_PLAIN);

                // Note: Content-ID for inline attachments requires using SinglePart directly
                // with custom headers in newer lettre versions
                let att = lettre::message::Attachment::new(attachment.filename.clone())
                    .body(attachment.data.clone(), content_type);

                mixed = mixed.singlepart(att);
            }
            mixed
        };

        builder
            .multipart(body)
            .map_err(|e| MailError::Smtp(e.to_string()))
    }
}

impl Default for Email {
    fn default() -> Self {
        Self::new()
    }
}

/// Email builder with validation.
#[derive(Default)]
pub struct EmailBuilder {
    email: Email,
}

impl EmailBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the from address.
    pub fn from(mut self, from: &str) -> Result<Self> {
        self.email.from = Some(Address::parse(from)?);
        Ok(self)
    }

    /// Set the to address.
    pub fn to(mut self, to: &str) -> Result<Self> {
        self.email.to.push(Address::parse(to)?);
        Ok(self)
    }

    /// Set the subject.
    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.email.subject = Some(subject.into());
        self
    }

    /// Set the text body.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.email.text = Some(text.into());
        self
    }

    /// Set the HTML body.
    pub fn html(mut self, html: impl Into<String>) -> Self {
        self.email.html = Some(html.into());
        self
    }

    /// Add an attachment.
    pub fn attach(mut self, attachment: Attachment) -> Self {
        self.email.attachments.push(attachment);
        self
    }

    /// Build and validate the email.
    pub fn build(self) -> Result<Email> {
        self.email.validate()?;
        Ok(self.email)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_builder() {
        let email = Email::new()
            .from("sender@example.com")
            .to("recipient@example.com")
            .subject("Test")
            .text("Hello, world!");

        assert!(email.validate().is_ok());
    }

    #[test]
    fn test_email_missing_from() {
        let email = Email::new()
            .to("recipient@example.com")
            .subject("Test")
            .text("Hello");

        assert!(email.validate().is_err());
    }
}

