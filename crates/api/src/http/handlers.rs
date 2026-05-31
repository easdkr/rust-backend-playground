use std::sync::Arc;

use application::pagination::CursorPage;
use application::post::dto::{CreatePostCmd, ListPostsQuery, PostDto, UpdatePostCmd};
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
        PostError::OwnershipError => AppError::Forbidden("Not the owner of this post".to_string()),
        PostError::Domain(msg) => AppError::BadRequest(msg),
        PostError::Internal(msg) => {
            tracing::error!("Post operation failed: {msg}");
            AppError::DatabaseError(msg)
        }
    }
}

/// List all posts with cursor pagination
#[utoipa::path(
    get,
    path = "/posts",
    params(ListPostsQuery),
    responses(
        (status = 200, description = "List of posts", body = CursorPage<PostDto>),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_posts(
    State(post_service): State<Arc<PostService>>,
    _user: AuthenticatedUser,
    axum::extract::Query(query): axum::extract::Query<ListPostsQuery>,
) -> AppResult<Json<CursorPage<PostDto>>> {
    let result = post_service
        .find_many(query)
        .await
        .map_err(map_post_error)?;
    Ok(Json(result))
}

/// Create a new post
#[utoipa::path(
    post,
    path = "/posts",
    request_body = CreatePostCmd,
    responses(
        (status = 201, description = "Post created", body = PostDto),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - insufficient permissions")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_post(
    State(post_service): State<Arc<PostService>>,
    user: AuthenticatedUser,
    ValidatedJson(payload): ValidatedJson<CreatePostCmd>,
) -> AppResult<(axum::http::StatusCode, Json<PostDto>)> {
    check_permission(&user.claims, Permission::PostCreate)?;
    let post = post_service
        .create(user.claims.sub.clone(), payload)
        .await
        .map_err(map_post_error)?;
    Ok((axum::http::StatusCode::CREATED, Json(PostDto::from(post))))
}

/// Get a single post by ID
#[utoipa::path(
    get,
    path = "/posts/{id}",
    params(("id" = i32, Path, description = "Post ID")),
    responses(
        (status = 200, description = "Post found", body = PostDto),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Post not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_post(
    State(post_service): State<Arc<PostService>>,
    _user: AuthenticatedUser,
    Path(id): Path<i32>,
) -> AppResult<Json<PostDto>> {
    let post = post_service.get(id).await.map_err(map_post_error)?;
    Ok(Json(PostDto::from(post)))
}

/// Update an existing post
#[utoipa::path(
    put,
    path = "/posts/{id}",
    params(("id" = i32, Path, description = "Post ID")),
    request_body = UpdatePostCmd,
    responses(
        (status = 200, description = "Post updated", body = PostDto),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - not owner or insufficient permissions"),
        (status = 404, description = "Post not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
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

/// Publish a post (change status to published)
#[utoipa::path(
    post,
    path = "/posts/{id}/publish",
    params(("id" = i32, Path, description = "Post ID")),
    responses(
        (status = 200, description = "Post published", body = PostDto),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - insufficient permissions"),
        (status = 404, description = "Post not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn publish_post(
    State(post_service): State<Arc<PostService>>,
    user: AuthenticatedUser,
    Path(id): Path<i32>,
) -> AppResult<Json<PostDto>> {
    check_permission(&user.claims, Permission::PostPublish)?;
    let post = post_service.publish(id).await.map_err(map_post_error)?;
    Ok(Json(PostDto::from(post)))
}

/// Delete a post
#[utoipa::path(
    delete,
    path = "/posts/{id}",
    params(("id" = i32, Path, description = "Post ID")),
    responses(
        (status = 200, description = "Post deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - not owner or insufficient permissions"),
        (status = 404, description = "Post not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
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
