use std::sync::Arc;

use application::user::{AuthService, Role};
use axum::{
    extract::{Request, State},
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};

use crate::error::AppError;

pub async fn jwt_auth_middleware(
    State(auth_service): State<Arc<AuthService>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = extract_bearer_token(request.headers().get(AUTHORIZATION))?;

    let claims = auth_service
        .validate_access_token(&token)
        .await
        .map_err(map_auth_error)?;

    request.extensions_mut().insert(claims);
    Ok(next.run(request).await)
}

pub async fn require_admin_middleware(request: Request, next: Next) -> Result<Response, AppError> {
    let claims = request
        .extensions()
        .get::<application::user::AccessTokenClaims>()
        .ok_or_else(|| AppError::Unauthorized("Not authenticated".to_string()))?;

    if claims.role != Role::Admin {
        return Err(AppError::Forbidden("Admin role required".to_string()));
    }

    Ok(next.run(request).await)
}

pub fn extract_bearer_token(header: Option<&axum::http::HeaderValue>) -> Result<String, AppError> {
    let header = header
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".to_string()))?;

    header
        .strip_prefix("Bearer ")
        .map(str::to_string)
        .ok_or_else(|| AppError::Unauthorized("Invalid Authorization header".to_string()))
}

pub fn map_auth_error(err: application::user::AuthError) -> AppError {
    match err {
        application::user::AuthError::InvalidCredentials => {
            AppError::Unauthorized("Invalid username or password".to_string())
        }
        application::user::AuthError::UserNotFound => {
            AppError::Unauthorized("User not found".to_string())
        }
        application::user::AuthError::TokenExpired => {
            AppError::TokenExpired("Access token has expired".to_string())
        }
        application::user::AuthError::TokenBlacklisted => {
            AppError::TokenBlacklisted("Access token has been revoked".to_string())
        }
        application::user::AuthError::TokenInvalid(msg) => AppError::Unauthorized(msg),
        application::user::AuthError::InsufficientPermissions => {
            AppError::Forbidden("Insufficient permissions".to_string())
        }
        application::user::AuthError::Internal(msg) => {
            tracing::error!("Auth error: {msg}");
            AppError::DatabaseError(msg)
        }
    }
}
