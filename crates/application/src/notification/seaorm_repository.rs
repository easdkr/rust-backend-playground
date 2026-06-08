use async_trait::async_trait;
use chrono::Utc;
use libs::error::IntoStringErr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, Set,
};

use infrastructure::persistence::seaorm::notification::{ActiveModel, Entity, Notification};

use super::repository::{
    NotificationFilter, NotificationListResult, NotificationRepository,
};

pub struct SeaOrmNotificationRepository {
    db: sea_orm::DatabaseConnection,
}

impl SeaOrmNotificationRepository {
    pub fn new(db: sea_orm::DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl NotificationRepository for SeaOrmNotificationRepository {
    async fn create(
        &self,
        user_id: String,
        notification_type: String,
        title: String,
        body: String,
        data: Option<serde_json::Value>,
    ) -> Result<Notification, String> {
        let model = ActiveModel {
            user_id: Set(user_id),
            notification_type: Set(notification_type),
            title: Set(title),
            body: Set(body),
            data: Set(data),
            is_read: Set(false),
            created_at: Set(Utc::now()),
            read_at: Set(None),
            ..Default::default()
        };

        model.insert(&self.db).await.map_err_string()
    }

    async fn find_by_id(&self, id: i32) -> Result<Option<Notification>, String> {
        Entity::find_by_id(id).one(&self.db).await.map_err_string()
    }

    async fn find_many(
        &self,
        filter: NotificationFilter,
    ) -> Result<NotificationListResult, String> {
        let mut condition = Condition::all().add(
            infrastructure::persistence::seaorm::notification::Column::UserId.eq(filter.user_id),
        );

        if let Some(is_read) = filter.is_read {
            condition = condition.add(
                infrastructure::persistence::seaorm::notification::Column::IsRead.eq(is_read),
            );
        }

        let total = Entity::find()
            .filter(condition.clone())
            .count(&self.db)
            .await
            .map_err_string()? as i64;

        let items = Entity::find()
            .filter(condition)
            .order_by_desc(infrastructure::persistence::seaorm::notification::Column::CreatedAt)
            .limit(filter.limit as u64)
            .offset(filter.offset as u64)
            .all(&self.db)
            .await
            .map_err_string()?;

        Ok(NotificationListResult { items, total })
    }

    async fn mark_as_read(&self, id: i32, user_id: &str) -> Result<bool, String> {
        let notification = Entity::find_by_id(id)
            .filter(infrastructure::persistence::seaorm::notification::Column::UserId.eq(user_id))
            .one(&self.db)
            .await
            .map_err_string()?;

        match notification {
            Some(n) => {
                let mut active: ActiveModel = n.into();
                active.is_read = Set(true);
                active.read_at = Set(Some(Utc::now()));
                active.update(&self.db).await.map_err_string()?;
                Ok(true)
            }
            None => Ok(false),
        }
    }

    async fn mark_all_as_read(&self, user_id: &str) -> Result<u64, String> {
        let notifications = Entity::find()
            .filter(
                infrastructure::persistence::seaorm::notification::Column::UserId.eq(user_id),
            )
            .filter(
                infrastructure::persistence::seaorm::notification::Column::IsRead.eq(false),
            )
            .all(&self.db)
            .await
            .map_err_string()?;

        let mut count = 0u64;
        for n in notifications {
            let mut active: ActiveModel = n.into();
            active.is_read = Set(true);
            active.read_at = Set(Some(Utc::now()));
            active.update(&self.db).await.map_err_string()?;
            count += 1;
        }

        Ok(count)
    }

    async fn count_unread(&self, user_id: &str) -> Result<i64, String> {
        Entity::find()
            .filter(
                infrastructure::persistence::seaorm::notification::Column::UserId.eq(user_id),
            )
            .filter(
                infrastructure::persistence::seaorm::notification::Column::IsRead.eq(false),
            )
            .count(&self.db)
            .await
            .map_err_string()
            .map(|c| c as i64)
    }

    async fn delete(&self, id: i32, user_id: &str) -> Result<bool, String> {
        let result = Entity::delete_many()
            .filter(infrastructure::persistence::seaorm::notification::Column::Id.eq(id))
            .filter(
                infrastructure::persistence::seaorm::notification::Column::UserId.eq(user_id),
            )
            .exec(&self.db)
            .await
            .map_err_string()?;

        Ok(result.rows_affected > 0)
    }
}
