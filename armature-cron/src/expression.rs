//! Cron expression parsing and evaluation.

use crate::error::{CronError, CronResult};
use chrono::{DateTime, Utc};
use cron::Schedule;
use std::str::FromStr;

/// Parsed cron expression.
#[derive(Debug, Clone)]
pub struct CronExpression {
    schedule: Schedule,
    expression: String,
}

impl CronExpression {
    /// Parse a cron expression.
    ///
    /// Supports standard cron format with 6 fields:
    /// - Second (0-59)
    /// - Minute (0-59)
    /// - Hour (0-23)
    /// - Day of month (1-31)
    /// - Month (1-12)
    /// - Day of week (0-6, Sunday = 0)
    ///
    /// # Examples
    ///
    /// ```
    /// use armature_cron::CronExpression;
    ///
    /// // Every minute
    /// let expr = CronExpression::parse("0 * * * * *").unwrap();
    ///
    /// // Every day at midnight
    /// let expr = CronExpression::parse("0 0 0 * * *").unwrap();
    ///
    /// // Every Monday at 9 AM
    /// let expr = CronExpression::parse("0 0 9 * * MON").unwrap();
    /// ```
    pub fn parse(expression: &str) -> CronResult<Self> {
        let schedule = Schedule::from_str(expression)
            .map_err(|e| CronError::InvalidExpression(format!("{}: {}", expression, e)))?;

        Ok(Self {
            schedule,
            expression: expression.to_string(),
        })
    }

    /// Get the next execution time after the given time.
    pub fn next_after(&self, after: DateTime<Utc>) -> Option<DateTime<Utc>> {
        self.schedule.after(&after).next()
    }

    /// Get the next execution time from now.
    pub fn next(&self) -> Option<DateTime<Utc>> {
        self.next_after(Utc::now())
    }

    /// Get the expression string.
    pub fn expression(&self) -> &str {
        &self.expression
    }

    /// Check if the expression would execute at the given time.
    pub fn matches(&self, time: DateTime<Utc>) -> bool {
        self.schedule
            .upcoming(Utc)
            .take(1)
            .any(|t| t.timestamp() == time.timestamp())
    }
}

/// Common cron expression presets.
pub struct CronPresets;

impl CronPresets {
    /// Every second
    pub const EVERY_SECOND: &'static str = "* * * * * *";

    /// Every minute
    pub const EVERY_MINUTE: &'static str = "0 * * * * *";

    /// Every 5 minutes
    pub const EVERY_5_MINUTES: &'static str = "0 */5 * * * *";

    /// Every 15 minutes
    pub const EVERY_15_MINUTES: &'static str = "0 */15 * * * *";

    /// Every 30 minutes
    pub const EVERY_30_MINUTES: &'static str = "0 */30 * * * *";

    /// Every hour
    pub const EVERY_HOUR: &'static str = "0 0 * * * *";

    /// Every day at midnight
    pub const DAILY: &'static str = "0 0 0 * * *";

    /// Every week on Sunday at midnight
    pub const WEEKLY: &'static str = "0 0 0 * * SUN";

    /// Every month on the 1st at midnight
    pub const MONTHLY: &'static str = "0 0 0 1 * *";

    /// Every year on January 1st at midnight
    pub const YEARLY: &'static str = "0 0 0 1 1 *";

    /// Every weekday (Monday-Friday) at 9 AM
    pub const WEEKDAYS_9AM: &'static str = "0 0 9 * * MON-FRI";

    /// Every weekend (Saturday-Sunday) at 10 AM
    pub const WEEKENDS_10AM: &'static str = "0 0 10 * * SAT,SUN";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_expression() {
        let expr = CronExpression::parse("0 * * * * *");
        assert!(expr.is_ok());
    }

    #[test]
    fn test_parse_invalid_expression() {
        let expr = CronExpression::parse("invalid");
        assert!(expr.is_err());
    }

    #[test]
    fn test_presets() {
        assert!(CronExpression::parse(CronPresets::EVERY_MINUTE).is_ok());
        assert!(CronExpression::parse(CronPresets::DAILY).is_ok());
        assert!(CronExpression::parse(CronPresets::WEEKLY).is_ok());
    }

    #[test]
    fn test_next_execution() {
        let expr = CronExpression::parse("0 * * * * *").unwrap();
        let next = expr.next();
        assert!(next.is_some());
    }
}
