//! Rate limiting algorithms
//!
//! This module provides different rate limiting algorithms:
//!
//! - **Token Bucket**: Smooth rate limiting with burst capacity
//! - **Sliding Window Log**: Precise rate limiting with individual request tracking
//! - **Fixed Window**: Simple rate limiting with fixed time windows

mod fixed_window;
mod sliding_window;
mod token_bucket;

pub use fixed_window::FixedWindow;
pub use sliding_window::SlidingWindowLog;
pub use token_bucket::TokenBucket;

use std::time::Duration;

/// Rate limiting algorithm configuration
#[derive(Debug, Clone)]
pub enum Algorithm {
    /// Token bucket algorithm
    ///
    /// Tokens are added at a fixed rate and consumed on each request.
    /// Allows bursts up to the bucket capacity.
    TokenBucket {
        /// Maximum number of tokens (burst capacity)
        capacity: u64,
        /// Tokens added per second
        refill_rate: f64,
    },

    /// Sliding window log algorithm
    ///
    /// Tracks individual request timestamps within a sliding window.
    /// Most accurate but requires more storage.
    SlidingWindowLog {
        /// Maximum requests allowed in the window
        max_requests: u64,
        /// Window duration
        window: Duration,
    },

    /// Fixed window algorithm
    ///
    /// Divides time into fixed windows and counts requests per window.
    /// Simple but can allow bursts at window boundaries.
    FixedWindow {
        /// Maximum requests allowed per window
        max_requests: u64,
        /// Window duration
        window: Duration,
    },
}

impl Algorithm {
    /// Create a token bucket algorithm with default values (100 capacity, 10/sec refill)
    pub fn token_bucket_default() -> Self {
        Self::TokenBucket {
            capacity: 100,
            refill_rate: 10.0,
        }
    }

    /// Create a sliding window algorithm with default values (100 requests per minute)
    pub fn sliding_window_default() -> Self {
        Self::SlidingWindowLog {
            max_requests: 100,
            window: Duration::from_secs(60),
        }
    }

    /// Create a fixed window algorithm with default values (100 requests per minute)
    pub fn fixed_window_default() -> Self {
        Self::FixedWindow {
            max_requests: 100,
            window: Duration::from_secs(60),
        }
    }

    /// Get the effective limit for this algorithm
    pub fn limit(&self) -> u64 {
        match self {
            Algorithm::TokenBucket { capacity, .. } => *capacity,
            Algorithm::SlidingWindowLog { max_requests, .. } => *max_requests,
            Algorithm::FixedWindow { max_requests, .. } => *max_requests,
        }
    }

    /// Get a human-readable description of the algorithm
    pub fn description(&self) -> String {
        match self {
            Algorithm::TokenBucket {
                capacity,
                refill_rate,
            } => format!(
                "Token bucket: {} capacity, {:.2} tokens/sec refill",
                capacity, refill_rate
            ),
            Algorithm::SlidingWindowLog {
                max_requests,
                window,
            } => format!("Sliding window: {} requests per {:?}", max_requests, window),
            Algorithm::FixedWindow {
                max_requests,
                window,
            } => format!("Fixed window: {} requests per {:?}", max_requests, window),
        }
    }
}

/// Trait for rate limiting algorithm implementations
pub trait RateLimitAlgorithm: Send + Sync {
    /// Check if a request is allowed
    /// Returns (allowed, remaining_count)
    fn check(&self, key: &str) -> (bool, u64);

    /// Reset the state for a key
    fn reset(&self, key: &str);

    /// Get the current remaining count for a key
    fn remaining(&self, key: &str) -> u64;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_algorithm_limit() {
        assert_eq!(
            Algorithm::TokenBucket {
                capacity: 50,
                refill_rate: 5.0
            }
            .limit(),
            50
        );
        assert_eq!(
            Algorithm::SlidingWindowLog {
                max_requests: 100,
                window: Duration::from_secs(60)
            }
            .limit(),
            100
        );
        assert_eq!(
            Algorithm::FixedWindow {
                max_requests: 200,
                window: Duration::from_secs(30)
            }
            .limit(),
            200
        );
    }

    #[test]
    fn test_algorithm_description() {
        let algo = Algorithm::TokenBucket {
            capacity: 100,
            refill_rate: 10.0,
        };
        assert!(algo.description().contains("Token bucket"));
        assert!(algo.description().contains("100"));
    }

    #[test]
    fn test_default_algorithms() {
        let tb = Algorithm::token_bucket_default();
        assert!(matches!(
            tb,
            Algorithm::TokenBucket {
                capacity: 100,
                refill_rate: _
            }
        ));

        let sw = Algorithm::sliding_window_default();
        assert!(matches!(
            sw,
            Algorithm::SlidingWindowLog {
                max_requests: 100,
                ..
            }
        ));

        let fw = Algorithm::fixed_window_default();
        assert!(matches!(
            fw,
            Algorithm::FixedWindow {
                max_requests: 100,
                ..
            }
        ));
    }
}
