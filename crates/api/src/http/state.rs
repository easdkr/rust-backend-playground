use std::sync::Arc;

use application::post::service::PostService;
use application::user::{AuthService, JwtConfig, UserService};
use axum::extract::FromRef;

#[derive(Clone)]
pub struct AppState {
    pub post_service: Arc<PostService>,
    pub auth_service: Arc<AuthService>,
    pub user_service: Arc<UserService>,
    pub jwt_config: Arc<JwtConfig>,
}

impl FromRef<AppState> for Arc<PostService> {
    fn from_ref(state: &AppState) -> Self {
        state.post_service.clone()
    }
}

impl FromRef<AppState> for Arc<AuthService> {
    fn from_ref(state: &AppState) -> Self {
        state.auth_service.clone()
    }
}

impl FromRef<AppState> for Arc<UserService> {
    fn from_ref(state: &AppState) -> Self {
        state.user_service.clone()
    }
}

impl FromRef<AppState> for Arc<JwtConfig> {
    fn from_ref(state: &AppState) -> Self {
        state.jwt_config.clone()
    }
}
