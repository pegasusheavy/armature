import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { DocPageComponent, DocPage } from '../../shared/doc-page.component';

@Component({
  selector: 'app-di-guide',
  standalone: true,
  imports: [CommonModule, DocPageComponent],
  template: `<app-doc-page [page]="page"></app-doc-page>`
})
export class DiGuideComponent {
  page: DocPage = {
    title: 'Dependency Injection',
    subtitle: 'Automatic, type-safe dependency injection inspired by Angular and NestJS. Services are injected based on field types, enabling loose coupling and testability.',
    icon: '💉',
    badge: 'Core',
    features: [
      {
        icon: '🎯',
        title: 'Automatic Injection',
        description: 'Dependencies resolved by field types, no manual wiring'
      },
      {
        icon: '🔒',
        title: 'Type-Safe',
        description: 'Compile-time verification of all dependencies'
      },
      {
        icon: '📦',
        title: 'Module System',
        description: 'Organize services with imports and exports'
      },
      {
        icon: '🧪',
        title: 'Testable',
        description: 'Easy mocking and dependency replacement'
      }
    ],
    sections: [
      {
        id: 'core-concepts',
        title: 'Core Concepts',
        content: `<p>Armature's DI system is built on three key concepts:</p>`,
        subsections: [
          {
            id: 'injectable-services',
            title: '1. Injectable Services',
            content: `<p>Mark a struct with <code>#[injectable]</code> to make it available for injection:</p>`,
            codeBlocks: [
              {
                language: 'rust',
                code: `#[injectable]
#[derive(Default, Clone)]
struct DatabaseService {
    connection_string: String,
}

// Requirements:
// - Must implement Default (for automatic instantiation)
// - Must implement Clone (for sharing across the app)
// - Must be Send + Sync + 'static (thread safety)`
              }
            ]
          },
          {
            id: 'service-dependencies',
            title: '2. Service Dependencies',
            content: `<p>Services can depend on other services by declaring them as fields:</p>`,
            codeBlocks: [
              {
                language: 'rust',
                code: `#[injectable]
#[derive(Default, Clone)]
struct UserService {
    database: DatabaseService,  // Auto-injected!
    logger: LoggerService,      // Also auto-injected!
}`
              }
            ]
          },
          {
            id: 'controllers-with-di',
            title: '3. Controllers with DI',
            content: `<p>Controllers automatically receive injected services through their fields:</p>`,
            codeBlocks: [
              {
                language: 'rust',
                code: `#[controller("/users")]
#[derive(Default, Clone)]
struct UserController {
    user_service: UserService,  // Automatically injected!
}

impl UserController {
    #[get("")]
    async fn get_users(&self) -> Result<Json<Vec<User>>, Error> {
        let users = self.user_service.find_all();
        Ok(Json(users))
    }
}`
              }
            ]
          }
        ]
      },
      {
        id: 'how-it-works',
        title: 'How It Works',
        content: `<p>The framework handles dependency registration and resolution automatically:</p>
        <ol>
          <li><strong>Imported modules</strong> are registered first (depth-first)</li>
          <li><strong>Providers</strong> (services) are registered in declaration order</li>
          <li><strong>Controllers</strong> are instantiated with resolved dependencies</li>
          <li><strong>Routes</strong> are registered for each controller</li>
        </ol>
        <p>Services are <strong>singletons</strong> by default — once created, the same instance is shared across the entire application.</p>`
      },
      {
        id: 'usage-examples',
        title: 'Usage Examples',
        subsections: [
          {
            id: 'simple-injection',
            title: 'Simple Service Injection',
            codeBlocks: [
              {
                language: 'rust',
                filename: 'main.rs',
                code: `use armature::prelude::*;

// Service with no dependencies
#[injectable]
#[derive(Default, Clone)]
struct ConfigService {
    api_url: String,
}

// Controller using the service
#[controller("/api")]
#[derive(Default, Clone)]
struct ApiController {
    config: ConfigService,
}

impl ApiController {
    #[get("/info")]
    async fn info(&self) -> Result<Json<String>, Error> {
        Ok(Json(self.config.api_url.clone()))
    }
}

#[module(
    providers: [ConfigService],
    controllers: [ApiController]
)]
#[derive(Default)]
struct AppModule;`
              }
            ]
          },
          {
            id: 'service-chain',
            title: 'Service Dependency Chain',
            content: `<p>The framework ensures all dependencies are resolved in the correct order:</p>`,
            codeBlocks: [
              {
                language: 'rust',
                code: `// Level 1: Base service
#[injectable]
#[derive(Default, Clone)]
struct LoggerService;

// Level 2: Service depending on Logger
#[injectable]
#[derive(Default, Clone)]
struct DatabaseService {
    logger: LoggerService,
}

// Level 3: Service depending on Database
#[injectable]
#[derive(Default, Clone)]
struct UserService {
    database: DatabaseService,
}

// Level 4: Controller depending on UserService
#[controller("/users")]
#[derive(Default, Clone)]
struct UserController {
    user_service: UserService,
}

#[module(
    providers: [LoggerService, DatabaseService, UserService],
    controllers: [UserController]
)]
#[derive(Default)]
struct AppModule;`
              }
            ]
          }
        ]
      },
      {
        id: 'module-system',
        title: 'Module System',
        content: `<p>Modules organize your application into reusable, composable units.</p>`,
        subsections: [
          {
            id: 'provider-declaration',
            title: 'Provider Declaration',
            content: `<p><strong>Order matters</strong> — list services with no dependencies first, then services that depend on earlier ones:</p>`,
            codeBlocks: [
              {
                language: 'rust',
                code: `#[module(
    providers: [ServiceA, ServiceB, ServiceC],
    controllers: [ControllerX, ControllerY]
)]
#[derive(Default)]
struct AppModule;`
              }
            ]
          },
          {
            id: 'module-imports',
            title: 'Module Imports & Exports',
            content: `<p>Modules can import other modules to access their exported services:</p>`,
            codeBlocks: [
              {
                language: 'rust',
                code: `#[module(
    providers: [SharedService],
    exports: [SharedService]  // Make available to importers
)]
#[derive(Default)]
struct SharedModule;

#[module(
    providers: [UserService],
    controllers: [UserController],
    imports: [SharedModule]  // Import shared services
)]
#[derive(Default)]
struct UserModule;`
              }
            ]
          }
        ]
      },
      {
        id: 'registering-services',
        title: 'Registering Built-in Services',
        content: `<p>Armature provides many built-in services you can register in the DI container:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            filename: 'health_service_setup.rs',
            code: `use armature_core::{
    Container, HealthService, HealthServiceBuilder,
    MemoryHealthIndicator, DiskHealthIndicator,
};

// Register a pre-built HealthService
fn register_health_service(container: &Container) {
    let health_service = HealthServiceBuilder::new()
        .with_defaults()
        .with_info(|info| {
            info.name("my-api")
                .version("1.0.0")
                .description("My REST API")
        })
        .build();

    container.register(health_service);
}

// Using in a controller
#[controller("/health")]
#[derive(Default, Clone)]
struct HealthController {
    health_service: HealthService,
}

impl HealthController {
    #[get("/")]
    async fn check(&self) -> Result<HttpResponse, Error> {
        let response = self.health_service.check().await;
        Ok(HttpResponse::ok().with_json(&response)?)
    }
}`
          }
        ]
      },
      {
        id: 'testing',
        title: 'Testing with DI',
        content: `<p>DI makes testing easier by allowing mock injection:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            filename: 'tests.rs',
            code: `#[cfg(test)]
mod tests {
    use super::*;

    #[injectable]
    #[derive(Default, Clone)]
    struct MockDatabaseService {
        // Mock implementation
    }

    #[test]
    fn test_controller() {
        let container = Container::new();
        container.register(MockDatabaseService::default());

        let controller = UserController::new_with_di(&container).unwrap();
        // Test controller with mock dependencies
    }
}`
          }
        ]
      },
      {
        id: 'best-practices',
        title: 'Best Practices',
        content: `<ul>
          <li><strong>Keep services stateless</strong> — Services should be stateless or have immutable state</li>
          <li><strong>Use descriptive names</strong> — <code>UserAuthenticationService</code> instead of <code>Service1</code></li>
          <li><strong>Minimize dependencies</strong> — Keep 2-3 dependencies max per service</li>
          <li><strong>Single responsibility</strong> — Create focused services: UserRepository, UserValidator, UserNotifier</li>
          <li><strong>Use Arc for expensive resources</strong> — Wrap connection pools in Arc for cheap cloning</li>
        </ul>`
      },
      {
        id: 'troubleshooting',
        title: 'Troubleshooting',
        content: `<p>Common issues and their solutions:</p>`,
        subsections: [
          {
            id: 'provider-not-found',
            title: '"Provider not found" Error',
            content: `<p>Ensure the service is listed in the module's <code>providers</code> array:</p>`,
            codeBlocks: [
              {
                language: 'rust',
                code: `#[module(
    providers: [MyService],  // Must be listed here!
    controllers: [MyController]
)]`
              }
            ]
          },
          {
            id: 'circular-deps',
            title: 'Circular Dependencies',
            content: `<p>Refactor to break the cycle by extracting shared logic:</p>`,
            codeBlocks: [
              {
                language: 'rust',
                code: `// Bad: Circular dependency
struct ServiceA { b: ServiceB }
struct ServiceB { a: ServiceA }  // Circular!

// Good: Extract shared dependency
struct ServiceA { shared: SharedService }
struct ServiceB { shared: SharedService }
struct SharedService { /* shared logic */ }`
              }
            ]
          }
        ]
      }
    ],
    relatedDocs: [
      {
        id: 'lifecycle-hooks',
        title: 'Lifecycle Hooks',
        description: 'Control service initialization and cleanup'
      },
      {
        id: 'macro-overview',
        title: 'Macros Overview',
        description: 'All available decorators and macros'
      },
      {
        id: 'config-guide',
        title: 'Configuration Guide',
        description: 'Environment and config management'
      }
    ],
    seeAlso: [
      { title: 'Project Templates', id: 'project-templates' },
      { title: 'Testing Guide', id: 'testing-guide' },
      { title: 'Health Checks', id: 'health-check' }
    ]
  };
}

