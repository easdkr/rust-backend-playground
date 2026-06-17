use std::sync::Arc;

use application::comment::dto::{
    CommentDto, CommentThreadDto, CreateCommentCmd, ListCommentsQuery, UpdateCommentCmd,
};
use application::comment::error::CommentError;
use application::comment::service::CommentService;
use application::pagination::CursorPage;
use axum::{
    Json,
    extract::{Path, State},
};

use crate::error::{AppError, AppResult};
use crate::http::extractors::{AuthenticatedUser, ValidatedJson};

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

/// Create a root comment on a post
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
    security(("bearer_auth" = []))
)]
pub async fn create_comment(
    State(comment_service): State<Arc<CommentService>>,
    user: AuthenticatedUser,
    Path(post_id): Path<i32>,
    ValidatedJson(cmd): ValidatedJson<CreateCommentCmd>,
) -> AppResult<(axum::http::StatusCode, Json<CommentDto>)> {
    let comment = comment_service
        .create_root(user.claims.sub, post_id, cmd)
        .await
        .map_err(map_comment_error)?;
    Ok((
        axum::http::StatusCode::CREATED,
        Json(CommentDto::from(comment)),
    ))
}

/// Create a reply to a comment
#[utoipa::path(
    post,
    path = "/posts/{post_id}/comments/{parent_comment_id}/replies",
    params(
        ("post_id" = i32, Path, description = "Post ID"),
        ("parent_comment_id" = i32, Path, description = "Parent comment ID")
    ),
    request_body = CreateCommentCmd,
    responses(
        (status = 201, description = "Reply created", body = CommentDto),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Post or parent comment not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_reply(
    State(comment_service): State<Arc<CommentService>>,
    user: AuthenticatedUser,
    Path((post_id, parent_comment_id)): Path<(i32, i32)>,
    ValidatedJson(cmd): ValidatedJson<CreateCommentCmd>,
) -> AppResult<(axum::http::StatusCode, Json<CommentDto>)> {
    let comment = comment_service
        .create_reply(user.claims.sub, post_id, parent_comment_id, cmd)
        .await
        .map_err(map_comment_error)?;
    Ok((
        axum::http::StatusCode::CREATED,
        Json(CommentDto::from(comment)),
    ))
}

/// List comment threads of a post with cursor pagination
#[utoipa::path(
    get,
    path = "/posts/{post_id}/comments",
    params(
        ("post_id" = i32, Path, description = "Post ID"),
        ListCommentsQuery
    ),
    responses(
        (status = 200, description = "List of comment threads", body = CursorPage<CommentThreadDto>),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Post not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_comments(
    State(comment_service): State<Arc<CommentService>>,
    _user: AuthenticatedUser,
    Path(post_id): Path<i32>,
    axum::extract::Query(query): axum::extract::Query<ListCommentsQuery>,
) -> AppResult<Json<CursorPage<CommentThreadDto>>> {
    let page = comment_service
        .list(post_id, query)
        .await
        .map_err(map_comment_error)?;
    Ok(Json(page))
}

/// Get a single comment
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
    security(("bearer_auth" = []))
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

/// Update a comment
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
    security(("bearer_auth" = []))
)]
pub async fn update_comment(
    State(comment_service): State<Arc<CommentService>>,
    user: AuthenticatedUser,
    Path((post_id, comment_id)): Path<(i32, i32)>,
    ValidatedJson(cmd): ValidatedJson<UpdateCommentCmd>,
) -> AppResult<Json<CommentDto>> {
    let comment = comment_service
        .update(post_id, comment_id, &user.claims.sub, cmd)
        .await
        .map_err(map_comment_error)?;
    Ok(Json(CommentDto::from(comment)))
}

/// Delete a comment
#[utoipa::path(
    delete,
    path = "/posts/{post_id}/comments/{comment_id}",
    params(
        ("post_id" = i32, Path, description = "Post ID"),
        ("comment_id" = i32, Path, description = "Comment ID")
    ),
    responses(
        (status = 200, description = "Comment deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - not owner"),
        (status = 404, description = "Post or comment not found")
    ),
    security(("bearer_auth" = []))
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
