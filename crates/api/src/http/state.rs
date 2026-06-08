use std::sync::Arc;

use application::notification::service::NotificationService;
use application::post::service::PostService;
use application::tag::service::TagService;
use application::user::{AuthService, JwtConfig, UserService};
use axum::extract::FromRef;

#[derive(Clone)]
pub struct AppState {
    pub post_service: Arc<PostService>,
    pub tag_service: Arc<TagService>,
    pub auth_service: Arc<AuthService>,
    pub user_service: Arc<UserService>,
    pub jwt_config: Arc<JwtConfig>,
    pub notification_service: Arc<NotificationService>,
}

impl FromRef<AppState> for Arc<PostService> {
    fn from_ref(state: &AppState) -> Self {
        state.post_service.clone()
    }
}

impl FromRef<AppState> for Arc<TagService> {
    fn from_ref(state: &AppState) -> Self {
        state.tag_service.clone()
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

impl FromRef<AppState> for Arc<NotificationService> {
    fn from_ref(state: &AppState) -> Self {
        state.notification_service.clone()
    }
}
