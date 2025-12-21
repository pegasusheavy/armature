# armature-cron

Cron job scheduling for the Armature framework.

## Features

- **Cron Expressions** - Standard cron syntax
- **Named Jobs** - Identify and manage jobs
- **Async Tasks** - Non-blocking job execution
- **Error Handling** - Job failure callbacks
- **Timezone Support** - Schedule in any timezone

## Installation

```toml
[dependencies]
armature-cron = "0.1"
```

## Quick Start

```rust
use armature_cron::CronScheduler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let scheduler = CronScheduler::new();

    // Run every minute
    scheduler.add("cleanup", "* * * * *", || async {
        println!("Running cleanup...");
        Ok(())
    })?;

    // Run daily at midnight
    scheduler.add("daily_report", "0 0 * * *", || async {
        generate_report().await
    })?;

    // Run every Monday at 9am
    scheduler.add("weekly_email", "0 9 * * MON", || async {
        send_weekly_digest().await
    })?;

    scheduler.start().await?;
    Ok(())
}
```

## Cron Syntax

```
┌───────────── minute (0-59)
│ ┌───────────── hour (0-23)
│ │ ┌───────────── day of month (1-31)
│ │ │ ┌───────────── month (1-12)
│ │ │ │ ┌───────────── day of week (0-6, Sun=0)
│ │ │ │ │
* * * * *
```

## License

MIT OR Apache-2.0

