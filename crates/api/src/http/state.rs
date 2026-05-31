use std::sync::Arc;

use application::post::service::PostService;
use axum::extract::FromRef;

#[derive(Clone)]
pub struct AppState {
    pub post_service: Arc<PostService>,
}

impl FromRef<AppState> for Arc<PostService> {
    fn from_ref(state: &AppState) -> Self {
        state.post_service.clone()
    }
}
