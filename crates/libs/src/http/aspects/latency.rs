use std::time::Instant;

use axum::{extract::Request, middleware::Next, response::Response};
use tracing::info;

/// Logs structured request latency after the handler completes.
pub async fn log_latency(request: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();

    let response = next.run(request).await;

    info!(
        method = %method,
        uri = %uri,
        status = response.status().as_u16(),
        latency_ms = start.elapsed().as_millis() as u64,
        "request completed"
    );

    response
}
