use std::sync::Arc;

use axum::{
    extract::{ConnectInfo, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use libs::rate_limit::RateLimiter;
use serde_json::json;

#[derive(Clone)]
pub struct RateLimitConfig {
    pub window_secs: u64,
    pub max_requests: u64,
    pub key_prefix: String,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            window_secs: 60,
            max_requests: 100,
            key_prefix: "rl".to_string(),
        }
    }
}

pub async fn rate_limit_middleware(
    State(limiter): State<Arc<RateLimiter>>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    request: Request,
    next: Next,
) -> Response {
    let identifier = addr.ip().to_string();

    match limiter.is_allowed(&identifier).await {
        Ok((allowed, remaining, reset_after)) => {
            if allowed {
                let mut response = next.run(request).await;
                let headers = response.headers_mut();
                headers.insert(
                    "X-RateLimit-Limit",
                    limiter.max_requests.to_string().parse().unwrap(),
                );
                headers.insert(
                    "X-RateLimit-Remaining",
                    remaining.to_string().parse().unwrap(),
                );
                headers.insert(
                    "X-RateLimit-Reset",
                    reset_after.to_string().parse().unwrap(),
                );
                response
            } else {
                (
                    StatusCode::TOO_MANY_REQUESTS,
                    axum::Json(json!({
                        "error": "Rate limit exceeded",
                        "retry_after": reset_after
                    })),
                )
                    .into_response()
            }
        }
        Err(e) => {
            tracing::warn!("Rate limiter error: {}", e);
            // Fail open - allow request if rate limiter is down
            next.run(request).await
        }
    }
}
