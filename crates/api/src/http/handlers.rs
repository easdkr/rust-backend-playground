use std::sync::Arc;

use application::comment::dto::{
    CommentDto, CommentThreadDto, CreateCommentCmd, ListCommentsQuery, UpdateCommentCmd,
};
use application::comment::error::CommentError;
use application::comment::service::CommentService;
use application::pagination::CursorPage;
use application::post::dto::{
    CreatePostCmd, ListPostsQuery, PostDetailDto, PostDto, UpdatePostCmd,
};
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

fn map_comment_error(err: CommentError) -> AppError {
    match err {
        CommentError::PostNotFound(id) => {
            AppError::NotFound(format!("Post with id {id} not found"))
        }
        CommentError::NotFound(id) => AppError::NotFound(format!("Comment with id {id} not found")),
        CommentError::OwnershipError => {
            AppError::Forbidden("Not the owner of this comment".to_string())
        }
        CommentError::Domain(msg) => AppError::BadRequest(msg),
        CommentError::Internal(msg) => {
            tracing::error!("Comment operation failed: {msg}");
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
        (status = 200, description = "Post found", body = PostDetailDto),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Post not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_post(
    State(post_service): State<Arc<PostService>>,
    State(comment_service): State<Arc<CommentService>>,
    _user: AuthenticatedUser,
    Path(id): Path<i32>,
) -> AppResult<Json<PostDetailDto>> {
    let post = post_service.get(id).await.map_err(map_post_error)?;
    let (comments, comment_count, has_more_comments) = comment_service
        .recent_threads(id, 5)
        .await
        .map_err(map_comment_error)?;
    Ok(Json(PostDetailDto::new(
        post,
        comments,
        comment_count,
        has_more_comments,
    )))
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

#[utoipa::path(
    get,
    path = "/posts/{post_id}/comments",
    params(
        ("post_id" = i32, Path, description = "Post ID"),
        ListCommentsQuery
    ),
    responses(
        (status = 200, description = "Post comments", body = CursorPage<CommentThreadDto>),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Post not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_comments(
    State(comment_service): State<Arc<CommentService>>,
    _user: AuthenticatedUser,
    Path(post_id): Path<i32>,
    axum::extract::Query(query): axum::extract::Query<ListCommentsQuery>,
) -> AppResult<Json<CursorPage<CommentThreadDto>>> {
    let result = comment_service
        .list(post_id, query)
        .await
        .map_err(map_comment_error)?;
    Ok(Json(result))
}

#[utoipa::path(
    post,
    path = "/posts/{post_id}/comments",
    params(("post_id" = i32, Path, description = "Post ID")),
    request_body = CreateCommentCmd,
    responses(
        (status = 201, description = "Comment created", body = CommentDto),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Post not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_comment(
    State(comment_service): State<Arc<CommentService>>,
    user: AuthenticatedUser,
    Path(post_id): Path<i32>,
    ValidatedJson(payload): ValidatedJson<CreateCommentCmd>,
) -> AppResult<(axum::http::StatusCode, Json<CommentDto>)> {
    let comment = comment_service
        .create_root(user.claims.sub.clone(), post_id, payload)
        .await
        .map_err(map_comment_error)?;
    Ok((
        axum::http::StatusCode::CREATED,
        Json(CommentDto::from(comment)),
    ))
}

#[utoipa::path(
    post,
    path = "/posts/{post_id}/comments/{comment_id}/replies",
    params(
        ("post_id" = i32, Path, description = "Post ID"),
        ("comment_id" = i32, Path, description = "Parent comment ID")
    ),
    request_body = CreateCommentCmd,
    responses(
        (status = 201, description = "Reply created", body = CommentDto),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Post or comment not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_comment_reply(
    State(comment_service): State<Arc<CommentService>>,
    user: AuthenticatedUser,
    Path((post_id, comment_id)): Path<(i32, i32)>,
    ValidatedJson(payload): ValidatedJson<CreateCommentCmd>,
) -> AppResult<(axum::http::StatusCode, Json<CommentDto>)> {
    let comment = comment_service
        .create_reply(user.claims.sub.clone(), post_id, comment_id, payload)
        .await
        .map_err(map_comment_error)?;
    Ok((
        axum::http::StatusCode::CREATED,
        Json(CommentDto::from(comment)),
    ))
}

#[utoipa::path(
    get,
    path = "/posts/{post_id}/comments/{comment_id}",
    params(
        ("post_id" = i32, Path, description = "Post ID"),
        ("comment_id" = i32, Path, description = "Comment ID")
    ),
    responses(
        (status = 200, description = "Comment found", body = CommentDto),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Post or comment not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_comment(
    State(comment_service): State<Arc<CommentService>>,
    _user: AuthenticatedUser,
    Path((post_id, comment_id)): Path<(i32, i32)>,
) -> AppResult<Json<CommentDto>> {
    let comment = comment_service
        .get(post_id, comment_id)
        .await
        .map_err(map_comment_error)?;
    Ok(Json(CommentDto::from(comment)))
}

#[utoipa::path(
    put,
    path = "/posts/{post_id}/comments/{comment_id}",
    params(
        ("post_id" = i32, Path, description = "Post ID"),
        ("comment_id" = i32, Path, description = "Comment ID")
    ),
    request_body = UpdateCommentCmd,
    responses(
        (status = 200, description = "Comment updated", body = CommentDto),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - not owner"),
        (status = 404, description = "Post or comment not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_comment(
    State(comment_service): State<Arc<CommentService>>,
    user: AuthenticatedUser,
    Path((post_id, comment_id)): Path<(i32, i32)>,
    ValidatedJson(payload): ValidatedJson<UpdateCommentCmd>,
) -> AppResult<Json<CommentDto>> {
    let comment = comment_service
        .update(post_id, comment_id, &user.claims.sub, payload)
        .await
        .map_err(map_comment_error)?;
    Ok(Json(CommentDto::from(comment)))
}

#[utoipa::path(
    delete,
    path = "/posts/{post_id}/comments/{comment_id}",
    params(
        ("post_id" = i32, Path, description = "Post ID"),
        ("comment_id" = i32, Path, description = "Comment ID")
    ),
    responses(
        (status = 200, description = "Comment deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - not owner"),
        (status = 404, description = "Post or comment not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_comment(
    State(comment_service): State<Arc<CommentService>>,
    user: AuthenticatedUser,
    Path((post_id, comment_id)): Path<(i32, i32)>,
) -> AppResult<Json<serde_json::Value>> {
    comment_service
        .delete(post_id, comment_id, &user.claims.sub)
        .await
        .map_err(map_comment_error)?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Comment with id {comment_id} deleted successfully")
    })))
}
