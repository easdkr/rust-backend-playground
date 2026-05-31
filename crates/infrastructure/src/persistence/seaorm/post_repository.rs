use async_trait::async_trait;
use chrono::Utc;
use libs::error::IntoStringErr;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryOrder, Set};

use super::post::{ActiveModel, Entity, Post};

#[async_trait]
pub trait PostRepository: Send + Sync {
    async fn find_all(&self) -> Result<Vec<Post>, String>;
    async fn find_by_id(&self, id: i32) -> Result<Option<Post>, String>;
    async fn create(&self, title: String, content: String, status: String) -> Result<Post, String>;
    async fn update(
        &self,
        id: i32,
        title: String,
        content: String,
        status: String,
    ) -> Result<Post, String>;
    async fn delete(&self, id: i32) -> Result<bool, String>;
    async fn count(&self) -> Result<u64, String>;
}

pub struct SeaOrmPostRepository {
    db: DatabaseConnection,
}

impl SeaOrmPostRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl PostRepository for SeaOrmPostRepository {
    async fn find_all(&self) -> Result<Vec<Post>, String> {
        Entity::find()
            .order_by_desc(super::post::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err_string()
    }

    async fn find_by_id(&self, id: i32) -> Result<Option<Post>, String> {
        Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err_string()
    }

    async fn create(&self, title: String, content: String, status: String) -> Result<Post, String> {
        let active = ActiveModel {
            title: Set(title),
            content: Set(content),
            status: Set(status),
            created_at: Set(Utc::now()),
            ..Default::default()
        };
        active.insert(&self.db).await.map_err_string()
    }

    async fn update(
        &self,
        id: i32,
        title: String,
        content: String,
        status: String,
    ) -> Result<Post, String> {
        let active = ActiveModel {
            id: Set(id),
            title: Set(title),
            content: Set(content),
            status: Set(status),
            ..Default::default()
        };
        active.update(&self.db).await.map_err_string()
    }

    async fn delete(&self, id: i32) -> Result<bool, String> {
        let res = Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err_string()?;

        Ok(res.rows_affected > 0)
    }

    async fn count(&self) -> Result<u64, String> {
        Entity::find()
            .count(&self.db)
            .await
            .map_err_string()
    }
}
