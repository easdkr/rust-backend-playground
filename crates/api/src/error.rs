use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    #[allow(dead_code)]
    DatabaseError(String),
    NotFound(String),
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    TokenExpired(String),
    TokenBlacklisted(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            AppError::DatabaseError(err) => {
                tracing::error!("Database error occurred: {}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal database error".to_string(),
                )
            }
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
            AppError::TokenExpired(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::TokenBlacklisted(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
        };

        let body = Json(json!({
            "success": false,
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;

pub trait ServiceResultExt<T> {
    fn map_bad_request(self) -> AppResult<T>;
    fn map_not_found(self) -> AppResult<T>;
}

impl<T> ServiceResultExt<T> for Result<T, String> {
    fn map_bad_request(self) -> AppResult<T> {
        self.map_err(AppError::BadRequest)
    }

    fn map_not_found(self) -> AppResult<T> {
        self.map_err(AppError::NotFound)
    }
}
