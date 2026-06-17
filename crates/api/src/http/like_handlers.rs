use std::sync::Arc;

use application::like::dto::LikeResult;
use application::like::service::LikeService;
use axum::{
    Json,
    extract::{Path, State},
};

use crate::error::{AppError, AppResult};
use crate::http::extractors::AuthenticatedUser;

fn map_like_error(err: application::like::error::LikeError) -> AppError {
    match err {
        application::like::error::LikeError::NotFound(id) => {
            AppError::NotFound(format!("Like for post {id} not found"))
        }
        application::like::error::LikeError::AlreadyExists => {
            AppError::BadRequest("Already liked".to_string())
        }
        application::like::error::LikeError::Internal(msg) => {
            tracing::error!("Like operation failed: {msg}");
            AppError::DatabaseError(msg)
        }
    }
}

/// Toggle like on a post
#[utoipa::path(
    post,
    path = "/posts/{id}/like",
    params(("id" = i32, Path, description = "Post ID")),
    responses(
        (status = 200, description = "Like toggled", body = LikeResult),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Post not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn toggle_like(
    State(like_service): State<Arc<LikeService>>,
    user: AuthenticatedUser,
    Path(id): Path<i32>,
) -> AppResult<Json<LikeResult>> {
    let result = like_service
        .toggle(id, user.claims.sub.clone())
        .await
        .map_err(map_like_error)?;
    Ok(Json(result))
}

/// Get like status for a post
#[utoipa::path(
    get,
    path = "/posts/{id}/like",
    params(("id" = i32, Path, description = "Post ID")),
    responses(
        (status = 200, description = "Like status", body = LikeResult),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_like_status(
    State(like_service): State<Arc<LikeService>>,
    user: AuthenticatedUser,
    Path(id): Path<i32>,
) -> AppResult<Json<LikeResult>> {
    let liked = like_service
        .is_liked(id, &user.claims.sub)
        .await
        .map_err(map_like_error)?;
    let count = like_service.get_count(id).await.map_err(map_like_error)?;
    Ok(Json(LikeResult {
        liked,
        like_count: count,
    }))
}
