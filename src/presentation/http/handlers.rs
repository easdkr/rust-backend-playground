use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;
use crate::application::post::service::PostService;
use crate::application::post::dto::{CreatePostCmd, UpdatePostCmd, PostDto};
use crate::error::{AppResult, AppError};

#[derive(Clone)]
pub struct AppState {
    pub post_service: Arc<PostService>,
}

pub async fn list_posts(
    State(state): State<AppState>,
) -> AppResult<Json<Vec<PostDto>>> {
    let posts = state.post_service.list().await
        .map_err(AppError::BadRequest)?;
    Ok(Json(posts))
}

pub async fn create_post(
    State(state): State<AppState>,
    Json(payload): Json<CreatePostCmd>,
) -> AppResult<Json<PostDto>> {
    let post = state.post_service.create(payload).await
        .map_err(AppError::BadRequest)?;
    Ok(Json(post))
}

pub async fn get_post(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> AppResult<Json<PostDto>> {
    let post = state.post_service.get(id).await
        .map_err(AppError::NotFound)?;
    Ok(Json(post))
}

pub async fn update_post(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdatePostCmd>,
) -> AppResult<Json<PostDto>> {
    let post = state.post_service.update(id, payload).await
        .map_err(AppError::BadRequest)?;
    Ok(Json(post))
}

pub async fn delete_post(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> AppResult<Json<serde_json::Value>> {
    state.post_service.delete(id).await
        .map_err(AppError::NotFound)?;
        
    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Post with id {} deleted successfully", id)
    })))
}
