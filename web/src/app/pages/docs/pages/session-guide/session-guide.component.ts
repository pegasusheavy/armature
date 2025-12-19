import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { DocPageComponent, DocPage } from '../../shared/doc-page.component';

@Component({
  selector: 'app-session-guide',
  standalone: true,
  imports: [CommonModule, DocPageComponent],
  template: `<app-doc-page [page]="page"></app-doc-page>`
})
export class SessionGuideComponent {
  page: DocPage = {
    title: 'Session Management',
    subtitle: 'Secure server-side sessions with Redis or database storage, automatic expiration, and CSRF protection.',
    icon: '🔐',
    badge: 'Security',
    features: [
      { icon: '💾', title: 'Redis Storage', description: 'Fast distributed sessions' },
      { icon: '⏰', title: 'Auto-expiration', description: 'Configurable TTL' },
      { icon: '🛡️', title: 'CSRF Protection', description: 'Built-in token validation' },
      { icon: '🔄', title: 'Regeneration', description: 'Prevent fixation attacks' }
    ],
    sections: [
      {
        id: 'setup',
        title: 'Basic Setup',
        content: `<p>Enable session middleware with Redis storage:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `use armature::prelude::*;
use armature_session::*;

Application::new()
    .middleware(SessionMiddleware::new(
        SessionConfig::builder()
            .store(RedisStore::new("redis://localhost:6379")?)
            .cookie_name("session_id")
            .ttl(Duration::from_hours(24))
            .secure(true)  // HTTPS only
            .http_only(true)
            .same_site(SameSite::Strict)
            .build()
    ))
    .run()
    .await`
          }
        ]
      },
      {
        id: 'usage',
        title: 'Using Sessions',
        content: `<p>Access and modify session data in handlers:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `#[controller("/api")]
struct ApiController;

impl ApiController {
    #[post("/login")]
    async fn login(&self, session: Session, body: Json<LoginRequest>) -> Result<Json<User>, Error> {
        // Authenticate user...
        let user = authenticate(&body.email, &body.password).await?;

        // Store user in session
        session.set("user_id", user.id).await?;
        session.set("role", &user.role).await?;

        // Regenerate session ID to prevent fixation
        session.regenerate().await?;

        Ok(Json(user))
    }

    #[get("/profile")]
    async fn profile(&self, session: Session) -> Result<Json<User>, Error> {
        // Get user from session
        let user_id: i64 = session.get("user_id").await?
            .ok_or(Error::Unauthorized)?;

        let user = find_user(user_id).await?;
        Ok(Json(user))
    }

    #[post("/logout")]
    async fn logout(&self, session: Session) -> StatusCode {
        // Destroy session
        session.destroy().await.ok();
        StatusCode::OK
    }
}`
          }
        ]
      },
      {
        id: 'flash-messages',
        title: 'Flash Messages',
        content: `<p>Store one-time messages between requests:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `#[post("/submit")]
async fn submit(&self, session: Session) -> Redirect {
    // Do something...

    // Store flash message (cleared after next read)
    session.flash("success", "Your form was submitted!").await?;

    Redirect::to("/thank-you")
}

#[get("/thank-you")]
async fn thank_you(&self, session: Session) -> Html<String> {
    // Get and clear flash message
    let message = session.get_flash::<String>("success").await?;

    Html(format!("<h1>{}</h1>", message.unwrap_or_default()))
}`
          }
        ]
      },
      {
        id: 'csrf',
        title: 'CSRF Protection',
        content: `<p>Protect forms against cross-site request forgery:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `// Enable CSRF protection
Application::new()
    .middleware(CsrfMiddleware::new(
        CsrfConfig::builder()
            .token_length(32)
            .cookie_name("csrf_token")
            .header_name("X-CSRF-Token")
            .excluded_paths(vec!["/api/webhooks"])
            .build()
    ))

// In handlers, get the token
#[get("/form")]
async fn show_form(&self, csrf: CsrfToken) -> Html<String> {
    Html(format!(r#"
        <form method="POST" action="/submit">
            <input type="hidden" name="_csrf" value="{}">
            <button type="submit">Submit</button>
        </form>
    "#, csrf.token()))
}

// POST requests automatically validated`
          }
        ]
      },
      {
        id: 'stores',
        title: 'Session Stores',
        content: `<p>Choose the right storage backend:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `// Redis (recommended for production)
let store = RedisStore::new("redis://localhost:6379")?;

// PostgreSQL (if you prefer SQL)
let store = PostgresStore::new(&database_pool)?;

// In-memory (development only!)
let store = MemoryStore::new();

// Custom store
#[async_trait]
impl SessionStore for MyStore {
    async fn get(&self, id: &str) -> Result<Option<Session>, Error>;
    async fn set(&self, session: &Session) -> Result<(), Error>;
    async fn delete(&self, id: &str) -> Result<(), Error>;
}`
          }
        ]
      },
      {
        id: 'best-practices',
        title: 'Best Practices',
        content: `<ul>
          <li><strong>Use HTTPS</strong> — Set <code>secure: true</code> for cookies</li>
          <li><strong>Regenerate after login</strong> — Prevent session fixation</li>
          <li><strong>Set reasonable TTL</strong> — Balance UX and security</li>
          <li><strong>Use HttpOnly cookies</strong> — Prevent XSS access</li>
          <li><strong>Implement CSRF protection</strong> — Required for stateful auth</li>
          <li><strong>Use Redis in production</strong> — Memory store doesn't scale</li>
        </ul>`
      }
    ],
    relatedDocs: [
      { id: 'auth-guide', title: 'Authentication', description: 'JWT and session auth' },
      { id: 'redis-guide', title: 'Redis', description: 'Session storage backend' }
    ],
    seeAlso: [
      { title: 'Security Guide', id: 'security-guide' },
      { title: 'Rate Limiting', id: 'rate-limiting' }
    ]
  };
}

