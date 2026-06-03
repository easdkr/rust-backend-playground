use std::sync::Arc;

use application::tag::dto::{AttachTagsCmd, CreateTagCmd, TagDto};
use application::tag::service::TagService;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use crate::error::{AppError, AppResult};
use crate::http::extractors::{AuthenticatedUser, ValidatedJson};

fn map_err(err: String) -> AppError {
    AppError::BadRequest(err)
}

/// List all tags
#[utoipa::path(
    get,
    path = "/tags",
    responses(
        (status = 200, description = "List of tags", body = Vec<TagDto>),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_tags(
    State(tag_service): State<Arc<TagService>>,
    _user: AuthenticatedUser,
) -> AppResult<Json<Vec<TagDto>>> {
    let tags = tag_service.list().await.map_err(map_err)?;
    Ok(Json(tags))
}

/// Create a tag
#[utoipa::path(
    post,
    path = "/tags",
    request_body = CreateTagCmd,
    responses(
        (status = 201, description = "Tag created", body = TagDto),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_tag(
    State(tag_service): State<Arc<TagService>>,
    _user: AuthenticatedUser,
    ValidatedJson(cmd): ValidatedJson<CreateTagCmd>,
) -> AppResult<(StatusCode, Json<TagDto>)> {
    let tag = tag_service
        .create(cmd.name, cmd.slug)
        .await
        .map_err(map_err)?;
    Ok((StatusCode::CREATED, Json(tag)))
}

/// List tags for a post
#[utoipa::path(
    get,
    path = "/posts/{id}/tags",
    params(("id" = i32, Path, description = "Post ID")),
    responses(
        (status = 200, description = "List of tags for the post", body = Vec<TagDto>),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_post_tags(
    State(tag_service): State<Arc<TagService>>,
    _user: AuthenticatedUser,
    Path(post_id): Path<i32>,
) -> AppResult<Json<Vec<TagDto>>> {
    let tags = tag_service.list_for_post(post_id).await.map_err(map_err)?;
    Ok(Json(tags))
}

/// Replace the set of tags for a post
#[utoipa::path(
    put,
    path = "/posts/{id}/tags",
    params(("id" = i32, Path, description = "Post ID")),
    request_body = AttachTagsCmd,
    responses(
        (status = 200, description = "Tags attached", body = Vec<TagDto>),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = []))
)]
pub async fn set_post_tags(
    State(tag_service): State<Arc<TagService>>,
    _user: AuthenticatedUser,
    Path(post_id): Path<i32>,
    ValidatedJson(cmd): ValidatedJson<AttachTagsCmd>,
) -> AppResult<Json<Vec<TagDto>>> {
    let tags = tag_service
        .set_for_post(post_id, cmd.tag_ids)
        .await
        .map_err(map_err)?;
    Ok(Json(tags))
}
