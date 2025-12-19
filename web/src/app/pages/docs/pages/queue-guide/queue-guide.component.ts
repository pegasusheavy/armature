import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { DocPageComponent, DocPage } from '../../shared/doc-page.component';

@Component({
  selector: 'app-queue-guide',
  standalone: true,
  imports: [CommonModule, DocPageComponent],
  template: `<app-doc-page [page]="page"></app-doc-page>`
})
export class QueueGuideComponent {
  page: DocPage = {
    title: 'Job Queues',
    subtitle: 'Process background jobs asynchronously with Redis-backed queues, retries, priorities, and scheduled execution.',
    icon: '📬',
    badge: 'Background',
    features: [
      { icon: '🔄', title: 'Automatic Retries', description: 'Configurable retry with backoff' },
      { icon: '⏰', title: 'Scheduled Jobs', description: 'Run jobs at specific times' },
      { icon: '📊', title: 'Priorities', description: 'High, normal, low priority queues' },
      { icon: '💀', title: 'Dead Letter', description: 'Handle failed jobs gracefully' }
    ],
    sections: [
      {
        id: 'basic-usage',
        title: 'Basic Usage',
        content: `<p>Define a job and enqueue it for background processing:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `use armature_queue::*;

// Define a job
#[derive(Serialize, Deserialize)]
pub struct SendEmailJob {
    pub to: String,
    pub subject: String,
    pub body: String,
}

#[async_trait]
impl Job for SendEmailJob {
    const NAME: &'static str = "send_email";

    async fn execute(&self, ctx: &JobContext) -> Result<(), JobError> {
        // Send the email
        send_email(&self.to, &self.subject, &self.body).await?;
        Ok(())
    }
}

// Enqueue the job
let queue = Queue::new("redis://localhost:6379").await?;

queue.enqueue(SendEmailJob {
    to: "user@example.com".into(),
    subject: "Welcome!".into(),
    body: "Thanks for signing up!".into(),
}).await?;`
          }
        ]
      },
      {
        id: 'worker',
        title: 'Running Workers',
        content: `<p>Start workers to process queued jobs:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `use armature_queue::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let queue = Queue::new("redis://localhost:6379").await?;

    // Register job handlers
    let worker = Worker::new(queue)
        .register::<SendEmailJob>()
        .register::<ProcessImageJob>()
        .register::<GenerateReportJob>()
        .concurrency(10)  // Process 10 jobs concurrently
        .build();

    // Start processing (blocks until shutdown)
    worker.run().await?;

    Ok(())
}`
          }
        ]
      },
      {
        id: 'retries',
        title: 'Retry Configuration',
        content: `<p>Configure automatic retries with exponential backoff:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `#[derive(Serialize, Deserialize)]
pub struct UnreliableJob {
    pub data: String,
}

#[async_trait]
impl Job for UnreliableJob {
    const NAME: &'static str = "unreliable_job";

    fn config() -> JobConfig {
        JobConfig::default()
            .retries(5)                          // Max 5 attempts
            .backoff(BackoffStrategy::Exponential {
                initial: Duration::from_secs(1),
                max: Duration::from_secs(300),   // Max 5 minute delay
                multiplier: 2.0,
            })
    }

    async fn execute(&self, ctx: &JobContext) -> Result<(), JobError> {
        println!("Attempt {} of {}", ctx.attempt, ctx.max_attempts);
        // Do work...
        Ok(())
    }
}`
          }
        ]
      },
      {
        id: 'scheduled-jobs',
        title: 'Scheduled Jobs',
        content: `<p>Schedule jobs to run at a specific time:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `// Schedule for later
queue.enqueue_at(
    SendReminderJob { user_id: 123 },
    Utc::now() + Duration::hours(24)  // Run in 24 hours
).await?;

// Schedule with delay
queue.enqueue_in(
    SendFollowUpJob { order_id: 456 },
    Duration::from_secs(3600)  // Run in 1 hour
).await?;`
          }
        ]
      },
      {
        id: 'priorities',
        title: 'Job Priorities',
        content: `<p>Process important jobs first with priority queues:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `// High priority - processed first
queue.enqueue_with_priority(
    CriticalAlertJob { message: "Server down!" },
    Priority::High
).await?;

// Normal priority (default)
queue.enqueue(
    SendNotificationJob { user_id: 123 }
).await?;

// Low priority - processed when queue is empty
queue.enqueue_with_priority(
    CleanupJob { older_than: 30 },
    Priority::Low
).await?;`
          }
        ]
      },
      {
        id: 'dead-letter',
        title: 'Dead Letter Queue',
        content: `<p>Handle jobs that fail after all retries:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `// Jobs that fail all retries go to dead letter queue
let failed_jobs = queue.dead_letter().list(0, 100).await?;

for job in failed_jobs {
    println!("Failed job: {} - Error: {}", job.id, job.error);

    // Optionally retry
    queue.dead_letter().retry(&job.id).await?;

    // Or delete
    queue.dead_letter().delete(&job.id).await?;
}

// Retry all dead letter jobs
queue.dead_letter().retry_all().await?;`
          }
        ]
      },
      {
        id: 'monitoring',
        title: 'Queue Monitoring',
        content: `<p>Monitor queue health and job status:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `// Get queue stats
let stats = queue.stats().await?;

println!("Pending: {}", stats.pending);
println!("Processing: {}", stats.processing);
println!("Completed: {}", stats.completed);
println!("Failed: {}", stats.failed);
println!("Dead letter: {}", stats.dead_letter);

// List pending jobs
let pending = queue.list_pending(0, 50).await?;

// Get job status
let status = queue.job_status(&job_id).await?;
match status {
    JobStatus::Pending => println!("Waiting to be processed"),
    JobStatus::Processing => println!("Currently running"),
    JobStatus::Completed => println!("Done!"),
    JobStatus::Failed(err) => println!("Failed: {}", err),
}`
          }
        ]
      },
      {
        id: 'best-practices',
        title: 'Best Practices',
        content: `<ul>
          <li><strong>Keep jobs small</strong> — Serialize minimal data, fetch rest in execute()</li>
          <li><strong>Make jobs idempotent</strong> — Safe to retry without side effects</li>
          <li><strong>Set reasonable timeouts</strong> — Prevent jobs from running forever</li>
          <li><strong>Monitor dead letter queue</strong> — Alert on failed jobs</li>
          <li><strong>Use priorities sparingly</strong> — Too many high priority defeats the purpose</li>
          <li><strong>Graceful shutdown</strong> — Wait for in-flight jobs to complete</li>
        </ul>`
      }
    ],
    relatedDocs: [
      { id: 'redis-guide', title: 'Redis', description: 'Queue backend configuration' },
      { id: 'cron-guide', title: 'Scheduled Tasks', description: 'Recurring job scheduling' }
    ],
    seeAlso: [
      { title: 'Graceful Shutdown', id: 'graceful-shutdown' },
      { title: 'Observability', id: 'metrics-guide' }
    ]
  };
}

