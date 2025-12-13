//! Retry policy for webhook delivery

use std::time::Duration;

/// Retry policy configuration
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts
    pub max_attempts: u32,

    /// Initial delay before first retry
    pub initial_delay: Duration,

    /// Maximum delay between retries
    pub max_delay: Duration,

    /// Multiplier for exponential backoff
    pub backoff_multiplier: f64,

    /// Whether to add jitter to delays
    pub jitter: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

impl RetryPolicy {
    /// Create a policy with no retries
    pub fn none() -> Self {
        Self {
            max_attempts: 0,
            ..Default::default()
        }
    }

    /// Create a policy with a fixed number of retries
    pub fn fixed(attempts: u32, delay: Duration) -> Self {
        Self {
            max_attempts: attempts,
            initial_delay: delay,
            max_delay: delay,
            backoff_multiplier: 1.0,
            jitter: false,
        }
    }

    /// Create a policy with exponential backoff
    pub fn exponential(max_attempts: u32) -> Self {
        Self {
            max_attempts,
            ..Default::default()
        }
    }

    /// Calculate the delay for a given attempt number
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return Duration::ZERO;
        }

        let base_delay =
            self.initial_delay.as_secs_f64() * self.backoff_multiplier.powi((attempt - 1) as i32);

        let delay_secs = base_delay.min(self.max_delay.as_secs_f64());

        let final_delay = if self.jitter {
            // Add up to 25% jitter
            let jitter_factor = 1.0 + (rand_jitter() * 0.25);
            delay_secs * jitter_factor
        } else {
            delay_secs
        };

        Duration::from_secs_f64(final_delay)
    }

    /// Check if another retry should be attempted
    pub fn should_retry(&self, attempt: u32) -> bool {
        attempt < self.max_attempts
    }
}

/// Simple deterministic jitter based on current time
fn rand_jitter() -> f64 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    (nanos % 1000) as f64 / 1000.0
}

/// Result of a retry operation
#[derive(Debug, Clone)]
pub enum RetryResult<T> {
    /// Operation succeeded
    Success(T),

    /// Operation failed but can be retried
    Retry { attempt: u32, error: String },

    /// Operation failed and should not be retried
    Failed { attempts: u32, error: String },
}

impl<T> RetryResult<T> {
    /// Check if the result is successful
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success(_))
    }

    /// Check if the result indicates a retry should be attempted
    pub fn should_retry(&self) -> bool {
        matches!(self, Self::Retry { .. })
    }

    /// Get the inner value if successful
    pub fn ok(self) -> Option<T> {
        match self {
            Self::Success(v) => Some(v),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_policy() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_attempts, 3);
        assert!(policy.jitter);
    }

    #[test]
    fn test_no_retries() {
        let policy = RetryPolicy::none();
        assert_eq!(policy.max_attempts, 0);
        assert!(!policy.should_retry(0));
    }

    #[test]
    fn test_fixed_policy() {
        let policy = RetryPolicy::fixed(5, Duration::from_secs(10));
        assert_eq!(policy.max_attempts, 5);
        assert_eq!(policy.backoff_multiplier, 1.0);
        assert!(!policy.jitter);

        // All delays should be the same
        let delay1 = policy.delay_for_attempt(1);
        let delay2 = policy.delay_for_attempt(2);
        assert_eq!(delay1, delay2);
    }

    #[test]
    fn test_exponential_backoff() {
        let policy = RetryPolicy {
            max_attempts: 5,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            jitter: false,
        };

        // Delays should increase exponentially
        let delay1 = policy.delay_for_attempt(1);
        let delay2 = policy.delay_for_attempt(2);
        let delay3 = policy.delay_for_attempt(3);

        assert_eq!(delay1, Duration::from_secs(1));
        assert_eq!(delay2, Duration::from_secs(2));
        assert_eq!(delay3, Duration::from_secs(4));
    }

    #[test]
    fn test_max_delay_cap() {
        let policy = RetryPolicy {
            max_attempts: 10,
            initial_delay: Duration::from_secs(10),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter: false,
        };

        // Should not exceed max_delay
        let delay = policy.delay_for_attempt(5);
        assert!(delay <= Duration::from_secs(30));
    }

    #[test]
    fn test_should_retry() {
        let policy = RetryPolicy::exponential(3);

        assert!(policy.should_retry(0));
        assert!(policy.should_retry(1));
        assert!(policy.should_retry(2));
        assert!(!policy.should_retry(3));
    }

    #[test]
    fn test_retry_result() {
        let success: RetryResult<i32> = RetryResult::Success(42);
        assert!(success.is_success());
        assert_eq!(success.ok(), Some(42));

        let retry: RetryResult<i32> = RetryResult::Retry {
            attempt: 1,
            error: "timeout".to_string(),
        };
        assert!(retry.should_retry());
    }
}
