import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { DocPageComponent, DocPage } from '../../shared/doc-page.component';

@Component({
  selector: 'app-api-versioning',
  standalone: true,
  imports: [CommonModule, DocPageComponent],
  template: `<app-doc-page [page]="page"></app-doc-page>`
})
export class ApiVersioningComponent {
  page: DocPage = {
    title: 'API Versioning',
    subtitle: 'Version your API with URL paths, headers, or query parameters for backward compatibility.',
    icon: '🏷️',
    badge: 'API',
    features: [
      { icon: '🔗', title: 'URL Versioning', description: '/api/v1/, /api/v2/' },
      { icon: '📋', title: 'Header Versioning', description: 'Accept-Version header' },
      { icon: '❓', title: 'Query Versioning', description: '?version=1' },
      { icon: '🎭', title: 'Media Type', description: 'application/vnd.api+json;v=1' }
    ],
    sections: [
      {
        id: 'url-versioning',
        title: 'URL Path Versioning',
        content: `<p>The most common and visible approach:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `// Version 1 API
#[controller("/api/v1/users")]
struct UserControllerV1;

impl UserControllerV1 {
    #[get("/:id")]
    async fn get_user(&self, id: Path<u32>) -> Json<UserV1> {
        // V1 response format
        Json(UserV1 { id: *id, name: "John".into() })
    }
}

// Version 2 API (extended response)
#[controller("/api/v2/users")]
struct UserControllerV2;

impl UserControllerV2 {
    #[get("/:id")]
    async fn get_user(&self, id: Path<u32>) -> Json<UserV2> {
        // V2 response with additional fields
        Json(UserV2 {
            id: *id,
            name: "John".into(),
            email: "john@example.com".into(),
            created_at: Utc::now(),
        })
    }
}`
          }
        ]
      },
      {
        id: 'header-versioning',
        title: 'Header Versioning',
        content: `<p>Version via custom headers:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `#[controller("/api/users")]
struct UserController;

impl UserController {
    #[get("/:id")]
    async fn get_user(&self, req: HttpRequest, id: Path<u32>) -> HttpResponse {
        let version = req.header("Accept-Version")
            .unwrap_or("1");

        match version {
            "1" => HttpResponse::ok().json(&self.get_v1(*id)),
            "2" => HttpResponse::ok().json(&self.get_v2(*id)),
            _ => HttpResponse::bad_request().body("Unknown version"),
        }
    }
}`
          },
          {
            language: 'bash',
            code: `# Request V1
$ curl -H "Accept-Version: 1" http://api.example.com/users/123

# Request V2
$ curl -H "Accept-Version: 2" http://api.example.com/users/123`
          }
        ]
      },
      {
        id: 'version-middleware',
        title: 'Version Middleware',
        content: `<p>Extract version globally with middleware:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `pub struct ApiVersionMiddleware;

#[async_trait]
impl Middleware for ApiVersionMiddleware {
    async fn handle(&self, mut req: HttpRequest, next: Next) -> Result<HttpResponse, Error> {
        // Extract version from header, query, or URL
        let version = req.header("Api-Version")
            .or_else(|| req.query("version"))
            .unwrap_or("1")
            .to_string();

        // Add to request extensions
        req.extensions_mut().insert(ApiVersion(version));

        next.run(req).await
    }
}

// Use in handlers
#[get("/data")]
async fn get_data(req: HttpRequest) -> Json<Value> {
    let version = req.extensions().get::<ApiVersion>();
    // Return appropriate response for version
}`
          }
        ]
      },
      {
        id: 'deprecation',
        title: 'Deprecation Notices',
        content: `<p>Warn clients about deprecated versions:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `#[controller("/api/v1/users")]
#[deprecated_version(sunset = "2025-01-01", message = "Use /api/v2/users")]
struct UserControllerV1;

// Automatically adds headers:
// Deprecation: true
// Sunset: Sat, 01 Jan 2025 00:00:00 GMT
// Link: </api/v2/users>; rel="successor-version"`
          }
        ]
      },
      {
        id: 'best-practices',
        title: 'Best Practices',
        content: `<ul>
          <li><strong>Start with versioning</strong> — Easier to add v2 than retrofit</li>
          <li><strong>URL versioning is clearest</strong> — Visible in logs, easy to route</li>
          <li><strong>Don't break existing versions</strong> — Additive changes only</li>
          <li><strong>Document breaking changes</strong> — Changelog for each version</li>
          <li><strong>Set sunset dates</strong> — Give clients time to migrate</li>
          <li><strong>Monitor version usage</strong> — Know when to deprecate</li>
        </ul>`
      }
    ],
    relatedDocs: [
      { id: 'route-groups', title: 'Route Groups', description: 'Organize versioned routes' },
      { id: 'content-negotiation', title: 'Content Negotiation', description: 'Accept header handling' }
    ],
    seeAlso: [
      { title: 'Middleware', id: 'use-middleware' },
      { title: 'Response Caching', id: 'response-caching' }
    ]
  };
}

