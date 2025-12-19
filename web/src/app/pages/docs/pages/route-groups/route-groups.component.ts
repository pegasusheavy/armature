import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { DocPageComponent, DocPage } from '../../shared/doc-page.component';

@Component({
  selector: 'app-route-groups',
  standalone: true,
  imports: [CommonModule, DocPageComponent],
  template: `<app-doc-page [page]="page"></app-doc-page>`
})
export class RouteGroupsComponent {
  page: DocPage = {
    title: 'Route Groups',
    subtitle: 'Organize routes with shared prefixes, middleware, and guards for cleaner code.',
    icon: '📂',
    badge: 'Routing',
    features: [
      { icon: '🏷️', title: 'Prefixes', description: 'Shared URL prefixes for related routes' },
      { icon: '🔗', title: 'Middleware', description: 'Apply middleware to entire groups' },
      { icon: '🛡️', title: 'Guards', description: 'Protect groups with auth guards' },
      { icon: '📊', title: 'Versioning', description: 'Version your API by groups' }
    ],
    sections: [
      {
        id: 'basic-groups',
        title: 'Basic Route Groups',
        content: `<p>Group related routes under a common prefix using the <code>#[controller]</code> attribute:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `#[controller("/api/users")]
#[derive(Default, Clone)]
struct UserController;

impl UserController {
    #[get("")]           // GET /api/users
    async fn list(&self) -> Json<Vec<User>> { ... }

    #[get("/:id")]       // GET /api/users/:id
    async fn get(&self, id: Path<u32>) -> Json<User> { ... }

    #[post("")]          // POST /api/users
    async fn create(&self, body: Json<CreateUser>) -> Json<User> { ... }

    #[put("/:id")]       // PUT /api/users/:id
    async fn update(&self, id: Path<u32>) -> Json<User> { ... }

    #[delete("/:id")]    // DELETE /api/users/:id
    async fn delete(&self, id: Path<u32>) -> StatusCode { ... }
}`
          }
        ]
      },
      {
        id: 'nested-groups',
        title: 'Nested Route Groups',
        content: `<p>Create nested route structures for complex APIs:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `// /api/v1/admin/users
#[controller("/api/v1/admin/users")]
#[derive(Default, Clone)]
struct AdminUserController;

// /api/v1/admin/settings
#[controller("/api/v1/admin/settings")]
#[derive(Default, Clone)]
struct AdminSettingsController;

// Group both under admin module
#[module(
    controllers: [AdminUserController, AdminSettingsController]
)]
struct AdminModule;`
          }
        ]
      },
      {
        id: 'group-middleware',
        title: 'Group Middleware',
        content: `<p>Apply middleware to all routes in a controller:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `#[controller("/api/admin")]
#[use_middleware(AuthMiddleware)]      // Applied to all routes
#[use_middleware(LoggingMiddleware)]   // Can stack multiple
#[derive(Default, Clone)]
struct AdminController;

impl AdminController {
    #[get("/dashboard")]
    async fn dashboard(&self) -> Json<Dashboard> {
        // Both AuthMiddleware and LoggingMiddleware run first
        ...
    }

    #[get("/users")]
    async fn users(&self) -> Json<Vec<User>> {
        // Same middleware chain
        ...
    }
}`
          }
        ]
      },
      {
        id: 'group-guards',
        title: 'Group Guards',
        content: `<p>Protect entire route groups with guards:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `#[controller("/api/admin")]
#[use_guard(AdminGuard)]  // All routes require admin role
#[derive(Default, Clone)]
struct AdminController;

#[controller("/api/user")]
#[use_guard(AuthGuard)]   // All routes require authentication
#[derive(Default, Clone)]
struct UserController;

// Define the guard
pub struct AdminGuard;

#[async_trait]
impl Guard for AdminGuard {
    async fn can_activate(&self, ctx: &RequestContext) -> bool {
        ctx.user()
            .map(|u| u.role == "admin")
            .unwrap_or(false)
    }
}`
          }
        ]
      },
      {
        id: 'api-versioning',
        title: 'API Versioning with Groups',
        content: `<p>Version your API using route groups:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `// Version 1 API
#[controller("/api/v1/users")]
#[derive(Default, Clone)]
struct UserControllerV1;

impl UserControllerV1 {
    #[get("/:id")]
    async fn get(&self, id: Path<u32>) -> Json<UserV1> {
        // V1 response format
        ...
    }
}

// Version 2 API (new format)
#[controller("/api/v2/users")]
#[derive(Default, Clone)]
struct UserControllerV2;

impl UserControllerV2 {
    #[get("/:id")]
    async fn get(&self, id: Path<u32>) -> Json<UserV2> {
        // V2 response format with additional fields
        ...
    }
}

#[module(
    controllers: [UserControllerV1, UserControllerV2]
)]
struct ApiModule;`
          }
        ]
      },
      {
        id: 'route-ordering',
        title: 'Route Ordering',
        content: `<p>Routes are matched in order of specificity. More specific routes match first:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `#[controller("/api/users")]
struct UserController;

impl UserController {
    #[get("/me")]        // More specific - matches first
    async fn current_user(&self) -> Json<User> { ... }

    #[get("/:id")]       // Less specific - matches after /me
    async fn get_user(&self, id: Path<u32>) -> Json<User> { ... }
}

// GET /api/users/me   -> current_user()
// GET /api/users/123  -> get_user()`
          }
        ]
      },
      {
        id: 'best-practices',
        title: 'Best Practices',
        content: `<ul>
          <li><strong>Use RESTful prefixes</strong> — <code>/api/v1/resource</code> is clearer than <code>/getResource</code></li>
          <li><strong>Group by domain</strong> — Keep related routes together</li>
          <li><strong>Apply auth at group level</strong> — Don't repeat guards on every route</li>
          <li><strong>Keep controllers focused</strong> — One resource per controller</li>
          <li><strong>Use versioning from the start</strong> — Easier to add v2 later</li>
        </ul>`
      }
    ],
    relatedDocs: [
      { id: 'use-middleware', title: 'Middleware', description: 'Request/response middleware' },
      { id: 'use-guard', title: 'Using Guards', description: 'Route protection' }
    ],
    seeAlso: [
      { title: 'Route Constraints', id: 'route-constraints' },
      { title: 'API Versioning', id: 'api-versioning' }
    ]
  };
}

