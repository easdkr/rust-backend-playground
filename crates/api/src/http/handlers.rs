use std::sync::Arc;

use application::comment::error::CommentError;
use application::comment::service::CommentService;
use application::pagination::CursorPage;
use application::post::dto::{
    BulkPostCmd, BulkPostResult, CreatePostCmd, ListPostsQuery, PostDto, PostRevisionDto,
    SearchPostsQuery, UpdatePostCmd,
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
        PostError::NotFoundBySlug(slug) => {
            AppError::NotFound(format!("Post with slug '{slug}' not found"))
        }
        PostError::OwnershipError => AppError::Forbidden("Not the owner of this post".to_string()),
        PostError::SlugConflict(slug) => {
            AppError::BadRequest(format!("Slug '{slug}' is already taken"))
        }
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

/// List posts with cursor pagination + filter/sort/search
#[utoipa::path(
    get,
    path = "/posts",
    params(ListPostsQuery),
    responses(
        (status = 200, description = "List of posts", body = CursorPage<PostDto>),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = []))
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

/// Full-text search posts
#[utoipa::path(
    get,
    path = "/posts/search",
    params(SearchPostsQuery),
    responses(
        (status = 200, description = "Search results", body = CursorPage<PostDto>),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = []))
)]
pub async fn search_posts(
    State(post_service): State<Arc<PostService>>,
    _user: AuthenticatedUser,
    axum::extract::Query(query): axum::extract::Query<SearchPostsQuery>,
) -> AppResult<Json<CursorPage<PostDto>>> {
    let result = post_service
        .search_fts(query)
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
    security(("bearer_auth" = []))
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

/// Get a single post by ID (also increments view count)
#[utoipa::path(
    get,
    path = "/posts/{id}",
    params(("id" = i32, Path, description = "Post ID")),
    responses(
        (status = 200, description = "Post found", body = PostDto),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Post not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_post(
    State(post_service): State<Arc<PostService>>,
    State(comment_service): State<Arc<CommentService>>,
    _user: AuthenticatedUser,
    Path(id): Path<i32>,
) -> AppResult<Json<PostDto>> {
    let post = post_service.get(id).await.map_err(map_post_error)?;
    // view count는 fire-and-forget (실패해도 응답에 영향 없음)
    tokio::spawn({
        let svc = Arc::clone(&post_service);
        async move {
            let _ = svc.increment_view(id).await;
        }
    });
    let (threads, count, has_more) = comment_service
        .recent_threads(id, 5)
        .await
        .map_err(map_comment_error)?;
    let mut dto = PostDto::from(post);
    dto.comment_count = count as i32;
    dto.has_more_comments = has_more;
    dto.comments = threads;
    Ok(Json(dto))
}

/// Get a single post by slug
#[utoipa::path(
    get,
    path = "/posts/by-slug/{slug}",
    params(("slug" = String, Path, description = "Post slug")),
    responses(
        (status = 200, description = "Post found", body = PostDto),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Post not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_post_by_slug(
    State(post_service): State<Arc<PostService>>,
    _user: AuthenticatedUser,
    axum::extract::Path(slug): axum::extract::Path<String>,
) -> AppResult<Json<PostDto>> {
    let post = post_service
        .get_by_slug(&slug)
        .await
        .map_err(map_post_error)?;
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
    security(("bearer_auth" = []))
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

/// Publish a post
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
    security(("bearer_auth" = []))
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

/// Unpublish a post (Published → Draft)
#[utoipa::path(
    post,
    path = "/posts/{id}/unpublish",
    params(("id" = i32, Path, description = "Post ID")),
    responses(
        (status = 200, description = "Post unpublished", body = PostDto),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - not owner"),
        (status = 404, description = "Post not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn unpublish_post(
    State(post_service): State<Arc<PostService>>,
    _user: AuthenticatedUser,
    _guard: OwnershipGuard<PostResource>,
    Path(id): Path<i32>,
) -> AppResult<Json<PostDto>> {
    let post = post_service.unpublish(id).await.map_err(map_post_error)?;
    Ok(Json(PostDto::from(post)))
}

/// Archive a post (Published → Archived)
#[utoipa::path(
    post,
    path = "/posts/{id}/archive",
    params(("id" = i32, Path, description = "Post ID")),
    responses(
        (status = 200, description = "Post archived", body = PostDto),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - not owner"),
        (status = 404, description = "Post not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn archive_post(
    State(post_service): State<Arc<PostService>>,
    _user: AuthenticatedUser,
    _guard: OwnershipGuard<PostResource>,
    Path(id): Path<i32>,
) -> AppResult<Json<PostDto>> {
    let post = post_service.archive(id).await.map_err(map_post_error)?;
    Ok(Json(PostDto::from(post)))
}

/// Restore an archived post (Archived → Published)
#[utoipa::path(
    post,
    path = "/posts/{id}/restore",
    params(("id" = i32, Path, description = "Post ID")),
    responses(
        (status = 200, description = "Post restored", body = PostDto),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - not owner"),
        (status = 404, description = "Post not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn restore_post(
    State(post_service): State<Arc<PostService>>,
    _user: AuthenticatedUser,
    _guard: OwnershipGuard<PostResource>,
    Path(id): Path<i32>,
) -> AppResult<Json<PostDto>> {
    let post = post_service.restore(id).await.map_err(map_post_error)?;
    Ok(Json(PostDto::from(post)))
}

/// Delete a post (soft delete)
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
    security(("bearer_auth" = []))
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

/// Bulk post operations
#[utoipa::path(
    post,
    path = "/posts/bulk",
    request_body = BulkPostCmd,
    responses(
        (status = 200, description = "Bulk operation result", body = BulkPostResult),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - insufficient permissions")
    ),
    security(("bearer_auth" = []))
)]
pub async fn bulk_posts(
    State(post_service): State<Arc<PostService>>,
    user: AuthenticatedUser,
    ValidatedJson(cmd): ValidatedJson<BulkPostCmd>,
) -> AppResult<Json<BulkPostResult>> {
    check_permission(&user.claims, Permission::PostUpdate)?;
    let result = post_service
        .bulk(&user.claims.sub, cmd.ids, cmd.action)
        .await
        .map_err(map_post_error)?;
    Ok(Json(result))
}

/// List revisions of a post
#[utoipa::path(
    get,
    path = "/posts/{id}/revisions",
    params(("id" = i32, Path, description = "Post ID")),
    responses(
        (status = 200, description = "List of revisions", body = Vec<PostRevisionDto>),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Post not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_post_revisions(
    State(post_service): State<Arc<PostService>>,
    _user: AuthenticatedUser,
    Path(id): Path<i32>,
) -> AppResult<Json<Vec<PostRevisionDto>>> {
    let revisions = post_service
        .list_revisions(id)
        .await
        .map_err(map_post_error)?;
    Ok(Json(revisions))
}

/// Get a specific revision of a post
#[utoipa::path(
    get,
    path = "/posts/{id}/revisions/{version}",
    params(
        ("id" = i32, Path, description = "Post ID"),
        ("version" = i32, Path, description = "Revision version")
    ),
    responses(
        (status = 200, description = "Revision found", body = PostRevisionDto),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Post or revision not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_post_revision(
    State(post_service): State<Arc<PostService>>,
    _user: AuthenticatedUser,
    Path((id, version)): Path<(i32, i32)>,
) -> AppResult<Json<PostRevisionDto>> {
    let revision = post_service
        .get_revision(id, version)
        .await
        .map_err(map_post_error)?;
    Ok(Json(revision))
}

/// Restore a post to a specific revision (Admin/Editor only)
#[utoipa::path(
    post,
    path = "/posts/{id}/revisions/{version}/restore",
    params(
        ("id" = i32, Path, description = "Post ID"),
        ("version" = i32, Path, description = "Revision version")
    ),
    responses(
        (status = 200, description = "Post restored", body = PostDto),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - insufficient permissions"),
        (status = 404, description = "Post or revision not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn restore_post_revision(
    State(post_service): State<Arc<PostService>>,
    user: AuthenticatedUser,
    Path((id, version)): Path<(i32, i32)>,
) -> AppResult<Json<PostDto>> {
    check_permission(&user.claims, Permission::PostUpdate)?;
    let post = post_service
        .restore_revision(id, version, &user.claims.sub)
        .await
        .map_err(map_post_error)?;
    Ok(Json(PostDto::from(post)))
}
