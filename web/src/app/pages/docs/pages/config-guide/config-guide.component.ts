import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { DocPageComponent, DocPage } from '../../shared/doc-page.component';

@Component({
  selector: 'app-config-guide',
  standalone: true,
  imports: [CommonModule, DocPageComponent],
  template: `<app-doc-page [page]="page"></app-doc-page>`
})
export class ConfigGuideComponent {
  page: DocPage = {
    title: 'Configuration',
    subtitle: 'Type-safe configuration management with environment variables, validation, and hot-reloading.',
    icon: '⚙️',
    badge: 'Core',
    features: [
      { icon: '🔒', title: 'Type-Safe', description: 'Compile-time config validation' },
      { icon: '🌍', title: 'Environment', description: 'Load from .env files' },
      { icon: '✅', title: 'Validation', description: 'Validate config at startup' },
      { icon: '🔄', title: 'Hot-Reload', description: 'Reload without restart' }
    ],
    sections: [
      {
        id: 'basic-config',
        title: 'Basic Configuration',
        content: `<p>Define your configuration struct with the <code>#[config]</code> attribute:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            filename: 'config.rs',
            code: `use armature::prelude::*;

#[config]
#[derive(Debug, Clone)]
pub struct AppConfig {
    #[env("DATABASE_URL")]
    pub database_url: String,

    #[env("PORT")]
    #[default(3000)]
    pub port: u16,

    #[env("LOG_LEVEL")]
    #[default("info")]
    pub log_level: String,

    #[env("JWT_SECRET")]
    pub jwt_secret: String,
}`
          }
        ]
      },
      {
        id: 'loading-config',
        title: 'Loading Configuration',
        content: `<p>Load configuration from environment variables and .env files:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            filename: 'main.rs',
            code: `use armature::prelude::*;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file (optional)
    dotenv().ok();

    // Parse configuration (validates at startup)
    let config = AppConfig::from_env()?;

    println!("Starting server on port {}", config.port);

    // Use config in your application
    Application::new()
        .config(config)
        .run()
        .await
}`
          }
        ]
      },
      {
        id: 'env-file',
        title: 'Environment Files',
        content: `<p>Create a <code>.env</code> file in your project root:</p>`,
        codeBlocks: [
          {
            language: 'bash',
            filename: '.env',
            code: `# Database
DATABASE_URL=postgres://user:pass@localhost:5432/mydb

# Server
PORT=3000
HOST=0.0.0.0

# Security
JWT_SECRET=your-super-secret-key-here
JWT_EXPIRY=3600

# Logging
LOG_LEVEL=debug
LOG_FORMAT=json

# Redis
REDIS_URL=redis://localhost:6379`
          }
        ]
      },
      {
        id: 'validation',
        title: 'Configuration Validation',
        content: `<p>Add validation rules to ensure config is correct at startup:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `#[config]
#[derive(Debug, Clone)]
pub struct AppConfig {
    #[env("PORT")]
    #[validate(range(min = 1, max = 65535))]
    pub port: u16,

    #[env("DATABASE_URL")]
    #[validate(url)]
    pub database_url: String,

    #[env("ADMIN_EMAIL")]
    #[validate(email)]
    pub admin_email: String,

    #[env("MAX_CONNECTIONS")]
    #[validate(range(min = 1, max = 1000))]
    #[default(100)]
    pub max_connections: u32,
}`
          }
        ]
      },
      {
        id: 'nested-config',
        title: 'Nested Configuration',
        content: `<p>Organize complex configuration with nested structs:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `#[config]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
}

#[config(prefix = "SERVER")]
pub struct ServerConfig {
    #[env("HOST")]
    #[default("0.0.0.0")]
    pub host: String,

    #[env("PORT")]
    #[default(3000)]
    pub port: u16,
}

#[config(prefix = "DB")]
pub struct DatabaseConfig {
    #[env("URL")]
    pub url: String,

    #[env("POOL_SIZE")]
    #[default(10)]
    pub pool_size: u32,
}`
          }
        ]
      },
      {
        id: 'accessing-config',
        title: 'Accessing Configuration',
        content: `<p>Inject configuration into services and controllers:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `#[injectable]
#[derive(Clone)]
pub struct MyService {
    config: AppConfig,
}

impl MyService {
    pub fn get_database_url(&self) -> &str {
        &self.config.database_url
    }
}

#[controller("/api")]
#[derive(Clone)]
pub struct ApiController {
    config: AppConfig,
}

impl ApiController {
    #[get("/config")]
    async fn get_config(&self) -> Json<ConfigInfo> {
        Json(ConfigInfo {
            port: self.config.port,
            log_level: self.config.log_level.clone(),
        })
    }
}`
          }
        ]
      },
      {
        id: 'profiles',
        title: 'Configuration Profiles',
        content: `<p>Use different configurations for different environments:</p>`,
        codeBlocks: [
          {
            language: 'bash',
            code: `# Development
$ APP_ENV=development cargo run  # Loads .env.development

# Staging
$ APP_ENV=staging cargo run      # Loads .env.staging

# Production
$ APP_ENV=production cargo run   # Loads .env.production`
          }
        ]
      },
      {
        id: 'best-practices',
        title: 'Best Practices',
        content: `<ul>
          <li><strong>Never commit secrets</strong> — Use <code>.env.example</code> as a template without real values</li>
          <li><strong>Use defaults wisely</strong> — Provide sensible defaults for development</li>
          <li><strong>Validate early</strong> — Fail fast if configuration is invalid</li>
          <li><strong>Group related config</strong> — Use nested structs for organization</li>
          <li><strong>Document all variables</strong> — Keep <code>.env.example</code> up to date</li>
        </ul>`
      }
    ],
    relatedDocs: [
      { id: 'di-guide', title: 'Dependency Injection', description: 'Inject config into services' },
      { id: 'deployment-guide', title: 'Deployment', description: 'Production configuration' }
    ],
    seeAlso: [
      { title: 'Project Templates', id: 'project-templates' },
      { title: 'Environment Files', id: 'docker-guide' }
    ]
  };
}

