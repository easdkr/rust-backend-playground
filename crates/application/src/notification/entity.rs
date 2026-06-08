use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    PostPublished,
    PostCommented,
    PostLiked,
    System,
    Mention,
}

impl NotificationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            NotificationType::PostPublished => "post_published",
            NotificationType::PostCommented => "post_commented",
            NotificationType::PostLiked => "post_liked",
            NotificationType::System => "system",
            NotificationType::Mention => "mention",
        }
    }
}

impl std::fmt::Display for NotificationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for NotificationType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "post_published" => Ok(NotificationType::PostPublished),
            "post_commented" => Ok(NotificationType::PostCommented),
            "post_liked" => Ok(NotificationType::PostLiked),
            "system" => Ok(NotificationType::System),
            "mention" => Ok(NotificationType::Mention),
            _ => Err(format!("Unknown notification type: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: i32,
    pub user_id: String,
    pub notification_type: NotificationType,
    pub title: String,
    pub body: String,
    pub data: Option<Value>,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
    pub read_at: Option<DateTime<Utc>>,
}
