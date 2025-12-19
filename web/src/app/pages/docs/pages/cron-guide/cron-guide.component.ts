import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { DocPageComponent, DocPage } from '../../shared/doc-page.component';

@Component({
  selector: 'app-cron-guide',
  standalone: true,
  imports: [CommonModule, DocPageComponent],
  template: `<app-doc-page [page]="page"></app-doc-page>`
})
export class CronGuideComponent {
  page: DocPage = {
    title: 'Scheduled Tasks (Cron)',
    subtitle: 'Schedule recurring tasks with cron expressions, timezone support, and distributed locking.',
    icon: '⏰',
    badge: 'Background',
    features: [
      { icon: '📅', title: 'Cron Expressions', description: 'Standard cron syntax support' },
      { icon: '🌍', title: 'Timezones', description: 'Run in any timezone' },
      { icon: '🔒', title: 'Distributed Lock', description: 'Run once across cluster' },
      { icon: '📊', title: 'Monitoring', description: 'Track execution history' }
    ],
    sections: [
      {
        id: 'basic-usage',
        title: 'Basic Scheduled Tasks',
        content: `<p>Define scheduled tasks with the <code>#[cron]</code> decorator:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `use armature_cron::*;

#[injectable]
#[derive(Default, Clone)]
pub struct ScheduledTasks;

impl ScheduledTasks {
    // Run every minute
    #[cron("* * * * *")]
    async fn every_minute(&self) {
        println!("Running every minute!");
    }

    // Run at midnight every day
    #[cron("0 0 * * *")]
    async fn daily_cleanup(&self) {
        cleanup_old_sessions().await;
    }

    // Run every Monday at 9 AM
    #[cron("0 9 * * 1")]
    async fn weekly_report(&self) {
        generate_weekly_report().await;
    }
}`
          }
        ]
      },
      {
        id: 'cron-expressions',
        title: 'Cron Expression Syntax',
        content: `<p>Standard 5-field cron expressions:</p>
        <ul>
          <li><code>* * * * *</code> — Every minute</li>
          <li><code>0 * * * *</code> — Every hour (at minute 0)</li>
          <li><code>0 0 * * *</code> — Every day at midnight</li>
          <li><code>0 0 * * 0</code> — Every Sunday at midnight</li>
          <li><code>0 0 1 * *</code> — First day of every month</li>
          <li><code>*/5 * * * *</code> — Every 5 minutes</li>
          <li><code>0 9-17 * * 1-5</code> — Every hour 9AM-5PM, Mon-Fri</li>
        </ul>`,
        codeBlocks: [
          {
            language: 'bash',
            code: `# Field positions:
# ┌───────────── minute (0-59)
# │ ┌───────────── hour (0-23)
# │ │ ┌───────────── day of month (1-31)
# │ │ │ ┌───────────── month (1-12)
# │ │ │ │ ┌───────────── day of week (0-6, Sunday=0)
# │ │ │ │ │
# * * * * *`
          }
        ]
      },
      {
        id: 'timezone',
        title: 'Timezone Support',
        content: `<p>Run tasks in specific timezones:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `// Run at 9 AM New York time
#[cron("0 9 * * *", timezone = "America/New_York")]
async fn new_york_morning(&self) {
    send_morning_digest().await;
}

// Run at 5 PM Tokyo time
#[cron("0 17 * * *", timezone = "Asia/Tokyo")]
async fn tokyo_evening(&self) {
    send_daily_summary().await;
}

// Default is UTC
#[cron("0 0 * * *")]  // Midnight UTC
async fn utc_midnight(&self) {
    run_daily_job().await;
}`
          }
        ]
      },
      {
        id: 'distributed',
        title: 'Distributed Locking',
        content: `<p>Ensure tasks run only once across multiple instances:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `// With distributed lock (default for clustered deployments)
#[cron("0 0 * * *", distributed = true)]
async fn daily_aggregation(&self) {
    // Only runs on ONE instance, even with 10 replicas
    aggregate_daily_stats().await;
}

// Without lock (runs on all instances)
#[cron("* * * * *", distributed = false)]
async fn local_cache_refresh(&self) {
    // Runs on EVERY instance
    refresh_local_cache().await;
}`
          }
        ]
      },
      {
        id: 'with-services',
        title: 'Using Injected Services',
        content: `<p>Scheduled tasks can use dependency injection:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `#[injectable]
#[derive(Clone)]
pub struct ReportScheduler {
    report_service: ReportService,
    email_service: EmailService,
    user_service: UserService,
}

impl ReportScheduler {
    #[cron("0 8 * * 1")]  // Monday 8 AM
    async fn send_weekly_reports(&self) {
        let users = self.user_service.get_report_subscribers().await;

        for user in users {
            let report = self.report_service.generate_weekly(&user).await;
            self.email_service.send(&user.email, report).await;
        }
    }
}`
          }
        ]
      },
      {
        id: 'error-handling',
        title: 'Error Handling',
        content: `<p>Handle errors in scheduled tasks:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `#[cron("0 * * * *", retry = 3, retry_delay = "5m")]
async fn hourly_sync(&self) -> Result<(), CronError> {
    // If this fails, it will retry up to 3 times
    // with 5 minute delays between attempts
    sync_external_data().await?;
    Ok(())
}

// Or handle errors manually
#[cron("*/10 * * * *")]
async fn frequent_task(&self) {
    if let Err(e) = do_work().await {
        error!("Scheduled task failed: {}", e);
        // Send alert, update metrics, etc.
        self.alerting.send_alert(&format!("Cron failed: {}", e)).await;
    }
}`
          }
        ]
      },
      {
        id: 'monitoring',
        title: 'Monitoring & History',
        content: `<p>Track scheduled task execution:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `// Get scheduler stats
let scheduler = app.get::<CronScheduler>();

let stats = scheduler.stats().await;
println!("Registered tasks: {}", stats.task_count);
println!("Executions today: {}", stats.executions_today);
println!("Failed today: {}", stats.failures_today);

// Get execution history
let history = scheduler.history("daily_cleanup", 10).await;
for execution in history {
    println!("{}: {} - {:?}",
        execution.started_at,
        execution.task_name,
        execution.status
    );
}`
          }
        ]
      },
      {
        id: 'best-practices',
        title: 'Best Practices',
        content: `<ul>
          <li><strong>Use distributed locks</strong> — Prevent duplicate execution in clusters</li>
          <li><strong>Add timeouts</strong> — Don't let tasks run forever</li>
          <li><strong>Log execution</strong> — Track when tasks run and their results</li>
          <li><strong>Stagger schedules</strong> — Don't run everything at midnight</li>
          <li><strong>Handle failures</strong> — Implement retry logic or alerting</li>
          <li><strong>Test in staging</strong> — Verify cron expressions before production</li>
        </ul>`
      }
    ],
    relatedDocs: [
      { id: 'queue-guide', title: 'Job Queues', description: 'One-off background jobs' },
      { id: 'graceful-shutdown', title: 'Graceful Shutdown', description: 'Clean task termination' }
    ],
    seeAlso: [
      { title: 'Distributed Locks', id: 'redis-guide' },
      { title: 'Logging', id: 'logging-guide' }
    ]
  };
}

