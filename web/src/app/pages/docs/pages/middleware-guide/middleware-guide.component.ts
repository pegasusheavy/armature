import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { DocPageComponent, DocPage } from '../../shared/doc-page.component';

@Component({
  selector: 'app-middleware-guide',
  standalone: true,
  imports: [CommonModule, DocPageComponent],
  template: `<app-doc-page [page]="page"></app-doc-page>`
})
export class MiddlewareGuideComponent {
  page: DocPage = {
    title: 'Middleware',
    subtitle: 'Process requests and responses with reusable middleware for logging, auth, compression, and more.',
    icon: '🔗',
    badge: 'Routing',
    features: [
      { icon: '📝', title: 'Request Processing', description: 'Modify requests before handlers' },
      { icon: '📤', title: 'Response Processing', description: 'Transform responses after handlers' },
      { icon: '⛓️', title: 'Chainable', description: 'Stack multiple middleware' },
      { icon: '🎯', title: 'Selective', description: 'Apply to specific routes' }
    ],
    sections: [
      {
        id: 'creating-middleware',
        title: 'Creating Middleware',
        content: `<p>Implement the <code>Middleware</code> trait to create custom middleware:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `use armature::prelude::*;

pub struct LoggingMiddleware;

#[async_trait]
impl Middleware for LoggingMiddleware {
    async fn handle(
        &self,
        req: HttpRequest,
        next: Next,
    ) -> Result<HttpResponse, Error> {
        let start = Instant::now();
        let method = req.method().clone();
        let path = req.path().to_string();

        // Call the next middleware/handler
        let response = next.run(req).await?;

        let duration = start.elapsed();
        info!("{} {} - {} ({:?})",
            method, path, response.status(), duration);

        Ok(response)
    }
}`
          }
        ]
      },
      {
        id: 'applying-middleware',
        title: 'Applying Middleware',
        content: `<p>Apply middleware at different levels:</p>`,
        subsections: [
          {
            id: 'global',
            title: 'Global Middleware',
            content: `<p>Apply to all routes:</p>`,
            codeBlocks: [
              {
                language: 'rust',
                code: `Application::new()
    .middleware(LoggingMiddleware)
    .middleware(CompressionMiddleware)
    .middleware(CorsMiddleware::permissive())
    .run()
    .await`
              }
            ]
          },
          {
            id: 'controller',
            title: 'Controller Middleware',
            content: `<p>Apply to all routes in a controller:</p>`,
            codeBlocks: [
              {
                language: 'rust',
                code: `#[controller("/api/admin")]
#[use_middleware(AuthMiddleware)]
#[use_middleware(AuditMiddleware)]
struct AdminController;`
              }
            ]
          },
          {
            id: 'route',
            title: 'Route Middleware',
            content: `<p>Apply to specific routes:</p>`,
            codeBlocks: [
              {
                language: 'rust',
                code: `impl ApiController {
    #[get("/public")]
    async fn public_endpoint(&self) -> Json<Data> {
        // No auth required
        ...
    }

    #[get("/private")]
    #[use_middleware(AuthMiddleware)]
    async fn private_endpoint(&self) -> Json<Data> {
        // Auth required for this route only
        ...
    }
}`
              }
            ]
          }
        ]
      },
      {
        id: 'modifying-request',
        title: 'Modifying Requests',
        content: `<p>Add data to requests for downstream handlers:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `pub struct UserContextMiddleware;

#[async_trait]
impl Middleware for UserContextMiddleware {
    async fn handle(&self, mut req: HttpRequest, next: Next) -> Result<HttpResponse, Error> {
        // Extract user from JWT token
        if let Some(user) = extract_user_from_token(&req) {
            // Add user to request extensions
            req.extensions_mut().insert(user);
        }

        next.run(req).await
    }
}

// Access in handlers
#[get("/profile")]
async fn get_profile(req: HttpRequest) -> Result<Json<User>, Error> {
    let user = req.extensions()
        .get::<User>()
        .ok_or(Error::Unauthorized)?;
    Ok(Json(user.clone()))
}`
          }
        ]
      },
      {
        id: 'modifying-response',
        title: 'Modifying Responses',
        content: `<p>Transform responses after the handler runs:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `pub struct SecurityHeadersMiddleware;

#[async_trait]
impl Middleware for SecurityHeadersMiddleware {
    async fn handle(&self, req: HttpRequest, next: Next) -> Result<HttpResponse, Error> {
        let mut response = next.run(req).await?;

        // Add security headers to all responses
        response.headers_mut().insert(
            "X-Content-Type-Options",
            "nosniff".parse().unwrap()
        );
        response.headers_mut().insert(
            "X-Frame-Options",
            "DENY".parse().unwrap()
        );
        response.headers_mut().insert(
            "X-XSS-Protection",
            "1; mode=block".parse().unwrap()
        );

        Ok(response)
    }
}`
          }
        ]
      },
      {
        id: 'short-circuiting',
        title: 'Short-Circuiting',
        content: `<p>Return early without calling the next handler:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `pub struct MaintenanceMiddleware {
    enabled: bool,
}

#[async_trait]
impl Middleware for MaintenanceMiddleware {
    async fn handle(&self, req: HttpRequest, next: Next) -> Result<HttpResponse, Error> {
        if self.enabled && !req.path().starts_with("/health") {
            // Return immediately without calling handler
            return Ok(HttpResponse::service_unavailable()
                .with_json(&json!({
                    "error": "Service under maintenance",
                    "retry_after": 3600
                }))?);
        }

        next.run(req).await
    }
}`
          }
        ]
      },
      {
        id: 'middleware-order',
        title: 'Middleware Order',
        content: `<p>Middleware executes in the order defined (first to last for requests, last to first for responses):</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `Application::new()
    .middleware(A)  // 1st request, 3rd response
    .middleware(B)  // 2nd request, 2nd response
    .middleware(C)  // 3rd request, 1st response
    .run()
    .await

// Request flow:  A -> B -> C -> Handler
// Response flow: Handler -> C -> B -> A`
          }
        ]
      },
      {
        id: 'best-practices',
        title: 'Best Practices',
        content: `<ul>
          <li><strong>Keep middleware focused</strong> — One responsibility per middleware</li>
          <li><strong>Consider order</strong> — Auth before logging, compression last</li>
          <li><strong>Don't block</strong> — Use async for I/O operations</li>
          <li><strong>Handle errors</strong> — Return appropriate error responses</li>
          <li><strong>Be careful with cloning</strong> — Request bodies can only be read once</li>
        </ul>`
      }
    ],
    relatedDocs: [
      { id: 'guards-interceptors', title: 'Guards & Interceptors', description: 'Access control patterns' },
      { id: 'route-groups', title: 'Route Groups', description: 'Organize routes with middleware' }
    ],
    seeAlso: [
      { title: 'Authentication', id: 'auth-guide' },
      { title: 'Logging', id: 'logging-guide' }
    ]
  };
}

