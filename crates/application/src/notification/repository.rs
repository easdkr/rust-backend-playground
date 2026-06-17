use async_trait::async_trait;

use infrastructure::persistence::seaorm::notification::Notification;

#[derive(Debug, Clone)]
pub struct NotificationFilter {
    pub user_id: String,
    pub is_read: Option<bool>,
    pub limit: i32,
    pub offset: i32,
}

#[derive(Debug, Clone)]
pub struct NotificationListResult {
    pub items: Vec<Notification>,
    pub total: i64,
}

#[async_trait]
pub trait NotificationRepository: Send + Sync {
    async fn create(
        &self,
        user_id: String,
        notification_type: String,
        title: String,
        body: String,
        data: Option<serde_json::Value>,
    ) -> Result<Notification, String>;

    async fn find_by_id(&self, id: i32) -> Result<Option<Notification>, String>;

    async fn find_many(&self, filter: NotificationFilter)
    -> Result<NotificationListResult, String>;

    async fn mark_as_read(&self, id: i32, user_id: &str) -> Result<bool, String>;

    async fn mark_all_as_read(&self, user_id: &str) -> Result<u64, String>;

    async fn count_unread(&self, user_id: &str) -> Result<i64, String>;

    async fn delete(&self, id: i32, user_id: &str) -> Result<bool, String>;
}
