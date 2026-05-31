use axum::http::HeaderName;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};

pub fn layers(
    header: &'static str,
) -> (SetRequestIdLayer<MakeRequestUuid>, PropagateRequestIdLayer) {
    let request_id = HeaderName::from_static(header);
    (
        SetRequestIdLayer::new(request_id.clone(), MakeRequestUuid),
        PropagateRequestIdLayer::new(request_id),
    )
}
