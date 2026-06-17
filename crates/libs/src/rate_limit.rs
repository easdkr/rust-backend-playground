use std::time::{SystemTime, UNIX_EPOCH};

use redis::{AsyncCommands, RedisResult};

/// Sliding window rate limiter using Redis sorted sets.
pub struct RateLimiter {
    redis: redis::aio::ConnectionManager,
    pub window_secs: u64,
    pub max_requests: u64,
    pub key_prefix: String,
}

impl RateLimiter {
    pub fn new(
        redis: redis::aio::ConnectionManager,
        window_secs: u64,
        max_requests: u64,
        key_prefix: String,
    ) -> Self {
        Self {
            redis,
            window_secs,
            max_requests,
            key_prefix,
        }
    }

    /// Check if the request is allowed and record it.
    /// Returns (allowed, remaining_requests, reset_after_secs).
    pub async fn is_allowed(&self, identifier: &str) -> RedisResult<(bool, u64, u64)> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        let window_start = now.saturating_sub(self.window_secs * 1000);
        let key = format!("{}:{}", self.key_prefix, identifier);

        let mut conn = self.redis.clone();

        // Remove old entries outside the window
        let _: () = redis::cmd("ZREMRANGEBYSCORE")
            .arg(&key)
            .arg(0)
            .arg(window_start)
            .query_async(&mut conn)
            .await?;

        // Count current entries in the window
        let current_count: u64 = conn.zcount(&key, window_start as i64, now as i64).await?;

        if current_count >= self.max_requests {
            // Get the oldest timestamp to calculate reset time
            let oldest: Vec<(u64, String)> = conn.zrange_withscores(&key, 0, 0).await?;
            let reset_after = if let Some((ts, _)) = oldest.first() {
                ((ts + self.window_secs * 1000).saturating_sub(now)) / 1000 + 1
            } else {
                self.window_secs
            };
            return Ok((false, 0, reset_after));
        }

        // Add current request timestamp
        let _: () = conn.zadd(&key, now.to_string(), now as f64).await?;

        // Set expiry on the key
        let _: () = conn.expire(&key, self.window_secs as i64).await?;

        let remaining = self.max_requests - current_count - 1;
        Ok((true, remaining, self.window_secs))
    }
}
