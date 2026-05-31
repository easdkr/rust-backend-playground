mod latency;
mod panic;
mod request_id;
mod trace;

use axum::Router;

pub use latency::log_latency;

const DEFAULT_REQUEST_ID_HEADER: &str = "x-request-id";

/// Composes standard HTTP cross-cutting layers (request ID, panic recovery, tracing, latency).
pub struct HttpAspects {
    request_id_header: &'static str,
}

impl HttpAspects {
    pub fn new() -> Self {
        Self {
            request_id_header: DEFAULT_REQUEST_ID_HEADER,
        }
    }

    pub fn with_request_id_header(mut self, header: &'static str) -> Self {
        self.request_id_header = header;
        self
    }

    pub fn apply<S>(self, router: Router<S>) -> Router<S>
    where
        S: Clone + Send + Sync + 'static,
    {
        let (set_layer, propagate_layer) = request_id::layers(self.request_id_header);

        router
            .layer(axum::middleware::from_fn(log_latency))
            .layer(set_layer)
            .layer(propagate_layer)
            .layer(panic::layer())
            .layer(trace::layer())
    }
}

impl Default for HttpAspects {
    fn default() -> Self {
        Self::new()
    }
}
