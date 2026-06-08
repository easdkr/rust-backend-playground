use std::sync::Arc;

use application::notification::dto::{CreateNotificationCmd, NotificationDto, NotificationListQuery};
use application::notification::service::NotificationService;
use axum::{
    Json,
    extract::{Query, State},
};

use crate::error::{AppError, AppResult};
use crate::http::extractors::AuthenticatedUser;

fn map_notification_error(err: application::notification::error::NotificationError) -> AppError {
    match err {
        application::notification::error::NotificationError::NotFound(id) => {
            AppError::NotFound(format!("Notification with id {id} not found"))
        }
        application::notification::error::NotificationError::Domain(msg) => AppError::BadRequest(msg),
        application::notification::error::NotificationError::Internal(msg) => {
            tracing::error!("Notification operation failed: {msg}");
            AppError::DatabaseError(msg)
        }
    }
}

/// List notifications for the current user
#[utoipa::path(
    get,
    path = "/notifications",
    params(NotificationListQuery),
    responses(
        (status = 200, description = "List of notifications", body = Vec<NotificationDto>),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_notifications(
    State(notification_service): State<Arc<NotificationService>>,
    user: AuthenticatedUser,
    Query(query): Query<NotificationListQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let result = notification_service
        .list(user.claims.sub.clone(), query)
        .await
        .map_err(map_notification_error)?;

    Ok(Json(serde_json::json!({
        "items": result.items.into_iter().map(NotificationDto::from).collect::<Vec<_>>(),
        "total": result.total,
    })))
}

/// Create a notification (admin/system use)
#[utoipa::path(
    post,
    path = "/notifications",
    request_body = CreateNotificationCmd,
    responses(
        (status = 201, description = "Notification created", body = NotificationDto),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_notification(
    State(notification_service): State<Arc<NotificationService>>,
    Json(cmd): Json<CreateNotificationCmd>,
) -> AppResult<(axum::http::StatusCode, Json<NotificationDto>)> {
    let notification = notification_service
        .create(cmd)
        .await
        .map_err(map_notification_error)?;
    Ok((axum::http::StatusCode::CREATED, Json(notification)))
}

/// Mark a notification as read
#[utoipa::path(
    post,
    path = "/notifications/{id}/read",
    params(("id" = i32, Path, description = "Notification ID")),
    responses(
        (status = 200, description = "Notification marked as read"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Notification not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn mark_as_read(
    State(notification_service): State<Arc<NotificationService>>,
    user: AuthenticatedUser,
    axum::extract::Path(id): axum::extract::Path<i32>,
) -> AppResult<Json<serde_json::Value>> {
    notification_service
        .mark_as_read(id, &user.claims.sub)
        .await
        .map_err(map_notification_error)?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Notification {id} marked as read")
    })))
}

/// Mark all notifications as read
#[utoipa::path(
    post,
    path = "/notifications/read-all",
    responses(
        (status = 200, description = "All notifications marked as read"),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = []))
)]
pub async fn mark_all_as_read(
    State(notification_service): State<Arc<NotificationService>>,
    user: AuthenticatedUser,
) -> AppResult<Json<serde_json::Value>> {
    let count = notification_service
        .mark_all_as_read(&user.claims.sub)
        .await
        .map_err(map_notification_error)?;

    Ok(Json(serde_json::json!({
        "success": true,
        "count": count
    })))
}

/// Get unread notification count
#[utoipa::path(
    get,
    path = "/notifications/unread-count",
    responses(
        (status = 200, description = "Unread count"),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = []))
)]
pub async fn unread_count(
    State(notification_service): State<Arc<NotificationService>>,
    user: AuthenticatedUser,
) -> AppResult<Json<serde_json::Value>> {
    let count = notification_service
        .count_unread(&user.claims.sub)
        .await
        .map_err(map_notification_error)?;

    Ok(Json(serde_json::json!({ "count": count })))
}
