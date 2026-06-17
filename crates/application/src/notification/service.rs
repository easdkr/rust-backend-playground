use std::sync::Arc;

use super::dto::{CreateNotificationCmd, NotificationDto, NotificationListQuery};
use super::entity::NotificationType;
use super::error::NotificationError;
use super::repository::{NotificationFilter, NotificationListResult, NotificationRepository};

pub struct NotificationService {
    repo: Arc<dyn NotificationRepository>,
    broadcaster: Option<Arc<dyn NotificationBroadcaster>>,
}

#[async_trait::async_trait]
pub trait NotificationBroadcaster: Send + Sync {
    async fn broadcast(&self, user_id: &str, notification: &NotificationDto);
}

impl NotificationService {
    pub fn new(
        repo: Arc<dyn NotificationRepository>,
        broadcaster: Option<Arc<dyn NotificationBroadcaster>>,
    ) -> Self {
        Self { repo, broadcaster }
    }

    pub async fn create(
        &self,
        cmd: CreateNotificationCmd,
    ) -> Result<NotificationDto, NotificationError> {
        let notification_type = cmd
            .notification_type
            .parse::<NotificationType>()
            .map_err(NotificationError::Domain)?;

        let notification = self
            .repo
            .create(
                cmd.user_id.clone(),
                notification_type.as_str().to_string(),
                cmd.title.clone(),
                cmd.body.clone(),
                cmd.data.clone(),
            )
            .await
            .map_err(NotificationError::repo)?;

        let dto = NotificationDto::from(notification);

        if let Some(ref broadcaster) = self.broadcaster {
            broadcaster.broadcast(&cmd.user_id, &dto).await;
        }

        Ok(dto)
    }

    pub async fn list(
        &self,
        user_id: String,
        query: NotificationListQuery,
    ) -> Result<NotificationListResult, NotificationError> {
        let filter = NotificationFilter {
            user_id,
            is_read: query.is_read,
            limit: query.limit.unwrap_or(20).clamp(1, 100),
            offset: query.offset.unwrap_or(0).max(0),
        };

        self.repo
            .find_many(filter)
            .await
            .map_err(NotificationError::repo)
    }

    pub async fn mark_as_read(&self, id: i32, user_id: &str) -> Result<(), NotificationError> {
        let updated = self
            .repo
            .mark_as_read(id, user_id)
            .await
            .map_err(NotificationError::repo)?;

        if !updated {
            return Err(NotificationError::NotFound(id));
        }

        Ok(())
    }

    pub async fn mark_all_as_read(&self, user_id: &str) -> Result<u64, NotificationError> {
        self.repo
            .mark_all_as_read(user_id)
            .await
            .map_err(NotificationError::repo)
    }

    pub async fn count_unread(&self, user_id: &str) -> Result<i64, NotificationError> {
        self.repo
            .count_unread(user_id)
            .await
            .map_err(NotificationError::repo)
    }

    pub async fn delete(&self, id: i32, user_id: &str) -> Result<(), NotificationError> {
        let deleted = self
            .repo
            .delete(id, user_id)
            .await
            .map_err(NotificationError::repo)?;

        if !deleted {
            return Err(NotificationError::NotFound(id));
        }

        Ok(())
    }
}
