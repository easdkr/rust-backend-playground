use async_trait::async_trait;
use chrono::Utc;
use libs::error::IntoStringErr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, Set,
};

use application::notification::repository::{
    NotificationFilter, NotificationListResult, NotificationRepository,
};

use super::notification::{ActiveModel, Entity, Notification};

pub struct SeaOrmNotificationRepository {
    db: DatabaseConnection,
}

impl SeaOrmNotificationRepository {
    pub fn new(db: DatabaseConnection) -> Self {
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
        let mut condition = Condition::all().add(super::notification::Column::UserId.eq(filter.user_id));

        if let Some(is_read) = filter.is_read {
            condition = condition.add(super::notification::Column::IsRead.eq(is_read));
        }

        let total = Entity::find()
            .filter(condition.clone())
            .count(&self.db)
            .await
            .map_err_string()?;

        let items = Entity::find()
            .filter(condition)
            .order_by_desc(super::notification::Column::CreatedAt)
            .limit(filter.limit as u64)
            .offset(filter.offset as u64)
            .all(&self.db)
            .await
            .map_err_string()?;

        Ok(NotificationListResult { items, total })
    }

    async fn mark_as_read(&self, id: i32, user_id: &str) -> Result<bool, String> {
        let notification = Entity::find_by_id(id)
            .filter(super::notification::Column::UserId.eq(user_id))
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
        let result = Entity::update_many()
            .filter(super::notification::Column::UserId.eq(user_id))
            .filter(super::notification::Column::IsRead.eq(false))
            .col_expr(super::notification::Column::IsRead, sea_orm::Value::Bool(Some(true)))
            .col_expr(super::notification::Column::ReadAt, sea_orm::Value::ChronoDateTimeUtc(Some(Box::new(Utc::now()))))
            .exec(&self.db)
            .await
            .map_err_string()?;

        Ok(result.rows_affected)
    }

    async fn count_unread(&self, user_id: &str) -> Result<i64, String> {
        Entity::find()
            .filter(super::notification::Column::UserId.eq(user_id))
            .filter(super::notification::Column::IsRead.eq(false))
            .count(&self.db)
            .await
            .map_err_string()
    }

    async fn delete(&self, id: i32, user_id: &str) -> Result<bool, String> {
        let result = Entity::delete_many()
            .filter(super::notification::Column::Id.eq(id))
            .filter(super::notification::Column::UserId.eq(user_id))
            .exec(&self.db)
            .await
            .map_err_string()?;

        Ok(result.rows_affected > 0)
    }
}
