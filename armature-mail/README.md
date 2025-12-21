# armature-mail

Email sending for the Armature framework.

## Features

- **SMTP Support** - Send via any SMTP server
- **Templates** - HTML email templates
- **Attachments** - File and inline attachments
- **Providers** - SendGrid, Mailgun, AWS SES
- **Async** - Non-blocking email sending

## Installation

```toml
[dependencies]
armature-mail = "0.1"
```

## Quick Start

```rust
use armature_mail::{Mailer, Email};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mailer = Mailer::smtp("smtp.example.com")
        .credentials("user", "password")
        .build()?;

    let email = Email::new()
        .from("sender@example.com")
        .to("recipient@example.com")
        .subject("Hello!")
        .body("This is a test email.");

    mailer.send(email).await?;
    Ok(())
}
```

## HTML Templates

```rust
let email = Email::new()
    .from("sender@example.com")
    .to("recipient@example.com")
    .subject("Welcome!")
    .html(render_template("welcome.html", &context)?);
```

## Providers

### SendGrid

```rust
let mailer = Mailer::sendgrid("API_KEY").build()?;
```

### AWS SES

```rust
let mailer = Mailer::ses(region).build()?;
```

## License

MIT OR Apache-2.0

