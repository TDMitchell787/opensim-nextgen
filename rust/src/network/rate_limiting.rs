//! Rate limiting module for preventing abuse and DDoS attacks
//!
//! Implements token bucket rate limiting algorithm.

use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// Rate limiter using token bucket algorithm
pub struct RateLimiter {
    /// Maximum requests per minute
    max_requests_per_minute: u32,
    /// Token bucket for each client
    buckets: Arc<RwLock<HashMap<String, TokenBucket>>>,
    /// Cleanup interval for expired buckets
    cleanup_interval: Duration,
}

/// Token bucket for rate limiting
struct TokenBucket {
    /// Current number of tokens
    tokens: u32,
    /// Maximum number of tokens
    max_tokens: u32,
    /// Last refill time
    last_refill: Instant,
    /// Refill rate (tokens per second)
    refill_rate: f64,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(max_requests_per_minute: u32) -> Self {
        Self {
            max_requests_per_minute,
            buckets: Arc::new(RwLock::new(HashMap::new())),
            cleanup_interval: Duration::from_secs(300), // 5 minutes
        }
    }

    /// Check if a client is within rate limits
    pub async fn check_rate_limit(&self, client_id: &str) -> bool {
        let mut buckets = self.buckets.write().await;

        // Get or create bucket for this client
        let bucket = buckets.entry(client_id.to_string()).or_insert_with(|| {
            TokenBucket {
                tokens: self.max_requests_per_minute,
                max_tokens: self.max_requests_per_minute,
                last_refill: Instant::now(),
                refill_rate: self.max_requests_per_minute as f64 / 60.0, // tokens per second
            }
        });

        // Refill tokens based on time elapsed
        self.refill_tokens(bucket);

        // Check if we have tokens available
        if bucket.tokens > 0 {
            bucket.tokens -= 1;
            debug!("Rate limit check passed for client {}", client_id);
            true
        } else {
            warn!("Rate limit exceeded for client {}", client_id);
            false
        }
    }

    /// Refill tokens in a bucket based on time elapsed
    fn refill_tokens(&self, bucket: &mut TokenBucket) {
        let now = Instant::now();
        let elapsed = now.duration_since(bucket.last_refill);
        let tokens_to_add = (elapsed.as_secs_f64() * bucket.refill_rate) as u32;

        if tokens_to_add > 0 {
            bucket.tokens = bucket
                .tokens
                .saturating_add(tokens_to_add)
                .min(bucket.max_tokens);
            bucket.last_refill = now;
        }
    }

    /// Clean up old buckets to prevent memory leaks
    pub async fn cleanup_old_buckets(&self) {
        let mut buckets = self.buckets.write().await;
        let now = Instant::now();

        buckets.retain(|_, bucket| now.duration_since(bucket.last_refill) < self.cleanup_interval);

        debug!(
            "Cleaned up old rate limit buckets, {} buckets remaining",
            buckets.len()
        );
    }

    /// Get rate limit statistics for a client
    pub async fn get_client_stats(&self, client_id: &str) -> Option<RateLimitStats> {
        let buckets = self.buckets.read().await;

        if let Some(bucket) = buckets.get(client_id) {
            Some(RateLimitStats {
                tokens_remaining: bucket.tokens,
                max_tokens: bucket.max_tokens,
                refill_rate: bucket.refill_rate,
                last_refill: bucket.last_refill,
            })
        } else {
            None
        }
    }

    /// Reset rate limit for a specific client
    pub async fn reset_client(&self, client_id: &str) {
        let mut buckets = self.buckets.write().await;
        buckets.remove(client_id);
        debug!("Reset rate limit for client {}", client_id);
    }

    /// Update rate limit configuration
    pub fn update_config(&mut self, max_requests_per_minute: u32) {
        self.max_requests_per_minute = max_requests_per_minute;
        debug!(
            "Updated rate limit configuration: {} requests per minute",
            max_requests_per_minute
        );
    }
}

/// Rate limit statistics for a client
#[derive(Debug, Clone)]
pub struct RateLimitStats {
    /// Number of tokens remaining
    pub tokens_remaining: u32,
    /// Maximum number of tokens
    pub max_tokens: u32,
    /// Token refill rate (tokens per second)
    pub refill_rate: f64,
    /// Last time tokens were refilled
    pub last_refill: Instant,
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(1000) // Default to 1000 requests per minute
    }
}
