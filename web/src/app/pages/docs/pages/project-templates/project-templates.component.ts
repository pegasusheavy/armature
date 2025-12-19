import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { DocPageComponent, DocPage } from '../../shared/doc-page.component';

@Component({
  selector: 'app-project-templates',
  standalone: true,
  imports: [CommonModule, DocPageComponent],
  template: `<app-doc-page [page]="page"></app-doc-page>`
})
export class ProjectTemplatesComponent {
  page: DocPage = {
    title: 'Project Templates',
    subtitle: 'Get started quickly with pre-configured project templates for common use cases.',
    icon: '📦',
    badge: 'CLI',
    features: [
      { icon: '🚀', title: 'Quick Start', description: 'Create a new project in seconds' },
      { icon: '⚙️', title: 'Pre-configured', description: 'Best practices built-in' },
      { icon: '🔧', title: 'Customizable', description: 'Modify templates to fit your needs' },
      { icon: '📚', title: 'Multiple Options', description: 'REST API, GraphQL, Full-stack' }
    ],
    sections: [
      {
        id: 'installation',
        title: 'Installing the CLI',
        content: `<p>First, install the Armature CLI globally:</p>`,
        codeBlocks: [
          {
            language: 'bash',
            code: `$ cargo install armature-cli

# Verify installation
$ armature --version
armature-cli 0.1.0`
          }
        ]
      },
      {
        id: 'available-templates',
        title: 'Available Templates',
        content: `<p>Armature provides several project templates for different use cases:</p>`,
        subsections: [
          {
            id: 'rest-api',
            title: 'REST API Template',
            content: `<p>A minimal REST API with authentication and database integration:</p>`,
            codeBlocks: [
              {
                language: 'bash',
                code: `$ armature new my-api --template rest-api

# Project structure:
# my-api/
# ├── Cargo.toml
# ├── src/
# │   ├── main.rs
# │   ├── controllers/
# │   ├── services/
# │   ├── models/
# │   └── config.rs
# ├── .env.example
# └── README.md`
              }
            ]
          },
          {
            id: 'graphql',
            title: 'GraphQL Template',
            content: `<p>A GraphQL API with schema-first design:</p>`,
            codeBlocks: [
              {
                language: 'bash',
                code: `$ armature new my-graphql --template graphql

# Includes:
# - async-graphql integration
# - Schema file structure
# - Playground endpoint
# - Example resolvers`
              }
            ]
          },
          {
            id: 'microservice',
            title: 'Microservice Template',
            content: `<p>A production-ready microservice with observability:</p>`,
            codeBlocks: [
              {
                language: 'bash',
                code: `$ armature new my-service --template microservice

# Includes:
# - Health checks (liveness/readiness)
# - Prometheus metrics
# - OpenTelemetry tracing
# - Docker + Kubernetes manifests
# - CI/CD workflows`
              }
            ]
          }
        ]
      },
      {
        id: 'creating-project',
        title: 'Creating a New Project',
        content: `<p>Use the <code>armature new</code> command to scaffold a project:</p>`,
        codeBlocks: [
          {
            language: 'bash',
            code: `# Basic usage
$ armature new my-project

# With specific template
$ armature new my-project --template rest-api

# With options
$ armature new my-project \\
  --template microservice \\
  --database postgres \\
  --auth jwt`
          }
        ]
      },
      {
        id: 'template-options',
        title: 'Template Options',
        content: `<p>Customize your project with these flags:</p>
        <ul>
          <li><code>--database</code> - Database type: <code>postgres</code>, <code>mysql</code>, <code>sqlite</code>, <code>none</code></li>
          <li><code>--auth</code> - Authentication: <code>jwt</code>, <code>oauth2</code>, <code>session</code>, <code>none</code></li>
          <li><code>--cache</code> - Caching: <code>redis</code>, <code>memory</code>, <code>none</code></li>
          <li><code>--queue</code> - Job queue: <code>redis</code>, <code>rabbitmq</code>, <code>none</code></li>
          <li><code>--docker</code> - Include Docker files</li>
          <li><code>--k8s</code> - Include Kubernetes manifests</li>
        </ul>`
      },
      {
        id: 'generated-structure',
        title: 'Generated Project Structure',
        content: `<p>A typical generated project includes:</p>`,
        codeBlocks: [
          {
            language: 'bash',
            code: `my-project/
├── Cargo.toml              # Dependencies and metadata
├── src/
│   ├── main.rs             # Application entry point
│   ├── app.rs              # Module registration
│   ├── config.rs           # Configuration management
│   ├── controllers/        # HTTP controllers
│   │   ├── mod.rs
│   │   └── health.rs
│   ├── services/           # Business logic
│   │   └── mod.rs
│   ├── models/             # Data models
│   │   └── mod.rs
│   └── middleware/         # Custom middleware
│       └── mod.rs
├── tests/                  # Integration tests
├── .env.example            # Environment template
├── .gitignore
├── Dockerfile              # (if --docker)
├── docker-compose.yml      # (if --docker)
└── README.md`
          }
        ]
      },
      {
        id: 'running-project',
        title: 'Running Your Project',
        content: `<p>After creating your project:</p>`,
        codeBlocks: [
          {
            language: 'bash',
            code: `# Navigate to project
$ cd my-project

# Copy environment file
$ cp .env.example .env

# Run in development mode
$ cargo run

# Or use the CLI for hot-reloading
$ armature dev`
          }
        ]
      }
    ],
    relatedDocs: [
      { id: 'config-guide', title: 'Configuration', description: 'Environment and config management' },
      { id: 'di-guide', title: 'Dependency Injection', description: 'Service injection patterns' }
    ],
    seeAlso: [
      { title: 'Configuration Guide', id: 'config-guide' },
      { title: 'Docker Guide', id: 'docker-guide' }
    ]
  };
}

