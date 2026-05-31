use std::sync::Arc;

use application::user::{
    AuthService, LoginCmd, LogoutRequest, RefreshTokenRequest, TokenResponse, UserDto, UserService,
};
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};

use crate::error::AppResult;
use crate::http::middleware::auth::{extract_bearer_token, map_auth_error};

pub async fn login(
    State(auth_service): State<Arc<AuthService>>,
    Json(cmd): Json<LoginCmd>,
) -> AppResult<Json<TokenResponse>> {
    let token_pair = auth_service.login(cmd).await.map_err(map_auth_error)?;
    let expires_in = auth_service.jwt_config().access_expiry_secs;
    Ok(Json(token_pair.into_response(expires_in)))
}

pub async fn logout(
    State(auth_service): State<Arc<AuthService>>,
    headers: HeaderMap,
    Json(body): Json<LogoutRequest>,
) -> AppResult<StatusCode> {
    let access_token = extract_bearer_token(headers.get(axum::http::header::AUTHORIZATION))?;

    auth_service
        .logout(&access_token, body.refresh_token.as_deref())
        .await
        .map_err(map_auth_error)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn refresh(
    State(auth_service): State<Arc<AuthService>>,
    Json(body): Json<RefreshTokenRequest>,
) -> AppResult<Json<TokenResponse>> {
    let token_pair = auth_service
        .refresh(&body.refresh_token)
        .await
        .map_err(map_auth_error)?;
    let expires_in = auth_service.jwt_config().access_expiry_secs;
    Ok(Json(token_pair.into_response(expires_in)))
}

pub async fn list_users(
    State(user_service): State<Arc<UserService>>,
) -> AppResult<Json<Vec<UserDto>>> {
    let users = user_service.list_users().await.map_err(map_auth_error)?;
    Ok(Json(users))
}
