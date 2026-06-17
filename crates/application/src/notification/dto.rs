use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use validator::Validate;

use super::entity::Notification;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NotificationDto {
    pub id: i32,
    pub user_id: String,
    pub notification_type: String,
    pub title: String,
    pub body: String,
    pub data: Option<Value>,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
    pub read_at: Option<DateTime<Utc>>,
}

impl From<Notification> for NotificationDto {
    fn from(n: Notification) -> Self {
        Self {
            id: n.id,
            user_id: n.user_id,
            notification_type: n.notification_type.as_str().to_string(),
            title: n.title,
            body: n.body,
            data: n.data,
            is_read: n.is_read,
            created_at: n.created_at,
            read_at: n.read_at,
        }
    }
}

impl From<infrastructure::persistence::seaorm::notification::Notification> for NotificationDto {
    fn from(n: infrastructure::persistence::seaorm::notification::Notification) -> Self {
        Self {
            id: n.id,
            user_id: n.user_id,
            notification_type: n.notification_type,
            title: n.title,
            body: n.body,
            data: n.data,
            is_read: n.is_read,
            created_at: n.created_at,
            read_at: n.read_at,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct CreateNotificationCmd {
    pub user_id: String,
    pub notification_type: String,
    pub title: String,
    pub body: String,
    pub data: Option<Value>,
}

#[derive(Debug, Clone, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct NotificationListQuery {
    pub is_read: Option<bool>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

impl Default for NotificationListQuery {
    fn default() -> Self {
        Self {
            is_read: None,
            limit: Some(20),
            offset: Some(0),
        }
    }
}
