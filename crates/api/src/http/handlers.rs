use std::sync::Arc;

use application::post::dto::{CreatePostCmd, PostDto, UpdatePostCmd};
use application::post::error::PostError;
use application::post::service::PostService;
use application::user::Permission;
use axum::{
    Json,
    extract::{Path, State},
};

use crate::error::{AppError, AppResult};
use crate::http::extractors::{AuthenticatedUser, OwnershipGuard, PostResource, ValidatedJson};
use crate::http::guards::check_permission;

fn map_post_error(err: PostError) -> AppError {
    match err {
        PostError::NotFound(id) => AppError::NotFound(format!("Post with id {id} not found")),
        PostError::OwnershipError => {
            AppError::Forbidden("Not the owner of this post".to_string())
        }
        PostError::Domain(msg) => AppError::BadRequest(msg),
        PostError::Internal(msg) => {
            tracing::error!("Post operation failed: {msg}");
            AppError::DatabaseError(msg)
        }
    }
}

pub async fn list_posts(
    State(post_service): State<Arc<PostService>>,
    _user: AuthenticatedUser,
) -> AppResult<Json<Vec<PostDto>>> {
    let posts = post_service.list().await.map_err(map_post_error)?;
    Ok(Json(posts.into_iter().map(PostDto::from).collect()))
}

pub async fn create_post(
    State(post_service): State<Arc<PostService>>,
    user: AuthenticatedUser,
    ValidatedJson(payload): ValidatedJson<CreatePostCmd>,
) -> AppResult<Json<PostDto>> {
    check_permission(&user.claims, Permission::PostCreate)?;
    let post = post_service
        .create(user.claims.sub.clone(), payload)
        .await
        .map_err(map_post_error)?;
    Ok(Json(PostDto::from(post)))
}

pub async fn get_post(
    State(post_service): State<Arc<PostService>>,
    _user: AuthenticatedUser,
    Path(id): Path<i32>,
) -> AppResult<Json<PostDto>> {
    let post = post_service.get(id).await.map_err(map_post_error)?;
    Ok(Json(PostDto::from(post)))
}

pub async fn update_post(
    State(post_service): State<Arc<PostService>>,
    user: AuthenticatedUser,
    _guard: OwnershipGuard<PostResource>,
    Path(id): Path<i32>,
    ValidatedJson(payload): ValidatedJson<UpdatePostCmd>,
) -> AppResult<Json<PostDto>> {
    check_permission(&user.claims, Permission::PostUpdate)?;
    let post = post_service
        .update(id, &user.claims.sub, payload)
        .await
        .map_err(map_post_error)?;
    Ok(Json(PostDto::from(post)))
}

pub async fn publish_post(
    State(post_service): State<Arc<PostService>>,
    user: AuthenticatedUser,
    Path(id): Path<i32>,
) -> AppResult<Json<PostDto>> {
    check_permission(&user.claims, Permission::PostPublish)?;
    let post = post_service.publish(id).await.map_err(map_post_error)?;
    Ok(Json(PostDto::from(post)))
}

pub async fn delete_post(
    State(post_service): State<Arc<PostService>>,
    user: AuthenticatedUser,
    _guard: OwnershipGuard<PostResource>,
    Path(id): Path<i32>,
) -> AppResult<Json<serde_json::Value>> {
    check_permission(&user.claims, Permission::PostDelete)?;
    post_service
        .delete(id, &user.claims.sub)
        .await
        .map_err(map_post_error)?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Post with id {id} deleted successfully")
    })))
}
