import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { DocPageComponent, DocPage } from '../../shared/doc-page.component';

@Component({
  selector: 'app-lifecycle-hooks',
  standalone: true,
  imports: [CommonModule, DocPageComponent],
  template: `<app-doc-page [page]="page"></app-doc-page>`
})
export class LifecycleHooksComponent {
  page: DocPage = {
    title: 'Lifecycle Hooks',
    subtitle: 'Control service initialization, cleanup, and application lifecycle events.',
    icon: '🔄',
    badge: 'Core',
    features: [
      { icon: '🚀', title: 'OnInit', description: 'Run code when service is created' },
      { icon: '🗑️', title: 'OnDestroy', description: 'Cleanup when service is destroyed' },
      { icon: '📡', title: 'OnStart', description: 'Called when app starts listening' },
      { icon: '🛑', title: 'OnShutdown', description: 'Graceful shutdown handling' }
    ],
    sections: [
      {
        id: 'on-init',
        title: 'OnInit Hook',
        content: `<p>The <code>OnInit</code> trait is called after a service is created and all dependencies are injected:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `use armature::prelude::*;

#[injectable]
#[derive(Default, Clone)]
pub struct DatabaseService {
    pool: Option<Pool<Postgres>>,
}

#[async_trait]
impl OnInit for DatabaseService {
    async fn on_init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize database connection pool
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect("postgres://localhost/mydb")
            .await?;

        self.pool = Some(pool);
        println!("Database connection pool initialized");

        Ok(())
    }
}`
          }
        ]
      },
      {
        id: 'on-destroy',
        title: 'OnDestroy Hook',
        content: `<p>Clean up resources when the service is destroyed:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `#[injectable]
#[derive(Default, Clone)]
pub struct CacheService {
    client: Option<redis::Client>,
}

#[async_trait]
impl OnDestroy for CacheService {
    async fn on_destroy(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(client) = self.client.take() {
            // Close Redis connections
            drop(client);
            println!("Cache connections closed");
        }
        Ok(())
    }
}`
          }
        ]
      },
      {
        id: 'on-application-start',
        title: 'OnApplicationStart Hook',
        content: `<p>Run code when the HTTP server starts listening:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `#[injectable]
#[derive(Default, Clone)]
pub struct StartupService;

#[async_trait]
impl OnApplicationStart for StartupService {
    async fn on_application_start(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 Application started!");

        // Run migrations
        run_migrations().await?;

        // Warm up caches
        warm_cache().await?;

        // Register with service discovery
        register_service().await?;

        Ok(())
    }
}`
          }
        ]
      },
      {
        id: 'on-application-shutdown',
        title: 'OnApplicationShutdown Hook',
        content: `<p>Handle graceful shutdown when the application receives a termination signal:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `#[injectable]
#[derive(Default, Clone)]
pub struct ShutdownService {
    job_queue: JobQueue,
}

#[async_trait]
impl OnApplicationShutdown for ShutdownService {
    async fn on_application_shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("⏳ Shutting down gracefully...");

        // Stop accepting new jobs
        self.job_queue.stop_accepting().await;

        // Wait for in-flight jobs to complete (with timeout)
        self.job_queue.drain(Duration::from_secs(30)).await?;

        // Deregister from service discovery
        deregister_service().await?;

        println!("👋 Shutdown complete");
        Ok(())
    }
}`
          }
        ]
      },
      {
        id: 'module-lifecycle',
        title: 'Module Lifecycle',
        content: `<p>Modules also have lifecycle hooks for initialization and cleanup:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `#[module(
    providers: [DatabaseService, CacheService],
    controllers: [ApiController],
)]
#[derive(Default)]
pub struct AppModule;

#[async_trait]
impl OnModuleInit for AppModule {
    async fn on_module_init(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("AppModule initialized");
        Ok(())
    }
}

#[async_trait]
impl OnModuleDestroy for AppModule {
    async fn on_module_destroy(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("AppModule destroyed");
        Ok(())
    }
}`
          }
        ]
      },
      {
        id: 'execution-order',
        title: 'Execution Order',
        content: `<p>Lifecycle hooks are called in a specific order:</p>
        <ol>
          <li><strong>OnModuleInit</strong> — Called for each module (depth-first, imported modules first)</li>
          <li><strong>OnInit</strong> — Called for each service after instantiation</li>
          <li><strong>OnApplicationStart</strong> — Called once the HTTP server is ready</li>
          <li><em>Application runs...</em></li>
          <li><strong>OnApplicationShutdown</strong> — Called when shutdown signal received</li>
          <li><strong>OnDestroy</strong> — Called for each service during cleanup</li>
          <li><strong>OnModuleDestroy</strong> — Called for each module (reverse order)</li>
        </ol>`
      },
      {
        id: 'error-handling',
        title: 'Error Handling',
        content: `<p>If a lifecycle hook returns an error, the application will fail to start:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `#[async_trait]
impl OnInit for DatabaseService {
    async fn on_init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match self.connect().await {
            Ok(_) => Ok(()),
            Err(e) => {
                // Log the error
                error!("Failed to connect to database: {}", e);
                // Return error to prevent application from starting
                Err(e.into())
            }
        }
    }
}`
          }
        ]
      },
      {
        id: 'best-practices',
        title: 'Best Practices',
        content: `<ul>
          <li><strong>Keep hooks fast</strong> — Long-running init blocks application startup</li>
          <li><strong>Handle errors properly</strong> — Return errors to fail fast on bad config</li>
          <li><strong>Use timeouts</strong> — Don't let shutdown hang indefinitely</li>
          <li><strong>Log lifecycle events</strong> — Helps with debugging startup issues</li>
          <li><strong>Clean up resources</strong> — Always implement OnDestroy for resources that need cleanup</li>
        </ul>`
      }
    ],
    relatedDocs: [
      { id: 'di-guide', title: 'Dependency Injection', description: 'Service creation and injection' },
      { id: 'graceful-shutdown', title: 'Graceful Shutdown', description: 'Connection draining and cleanup' }
    ],
    seeAlso: [
      { title: 'Graceful Shutdown', id: 'graceful-shutdown' },
      { title: 'Health Checks', id: 'health-check' }
    ]
  };
}

