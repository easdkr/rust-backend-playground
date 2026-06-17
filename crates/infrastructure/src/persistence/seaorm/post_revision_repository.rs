use async_trait::async_trait;
use chrono::Utc;
use libs::error::IntoStringErr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};

use super::post_revision::Column;
use super::post_revision::{ActiveModel, Entity, PostRevision};

#[async_trait]
pub trait PostRevisionRepository: Send + Sync {
    async fn save_revision(
        &self,
        post_id: i32,
        editor_id: String,
        title: String,
        content: String,
        excerpt: Option<String>,
        status: String,
    ) -> Result<PostRevision, String>;

    async fn find_by_post_id(&self, post_id: i32) -> Result<Vec<PostRevision>, String>;

    async fn find_by_id(&self, id: i32) -> Result<Option<PostRevision>, String>;

    async fn find_by_post_id_and_version(
        &self,
        post_id: i32,
        version: i32,
    ) -> Result<Option<PostRevision>, String>;
}

pub struct SeaOrmPostRevisionRepository {
    db: DatabaseConnection,
}

impl SeaOrmPostRevisionRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl PostRevisionRepository for SeaOrmPostRevisionRepository {
    async fn save_revision(
        &self,
        post_id: i32,
        editor_id: String,
        title: String,
        content: String,
        excerpt: Option<String>,
        status: String,
    ) -> Result<PostRevision, String> {
        let next_version = Entity::find()
            .filter(Column::PostId.eq(post_id))
            .order_by_desc(Column::Version)
            .one(&self.db)
            .await
            .map_err_string()?
            .map(|r| r.version + 1)
            .unwrap_or(1);

        let active = ActiveModel {
            post_id: Set(post_id),
            editor_id: Set(editor_id),
            title: Set(title),
            content: Set(content),
            excerpt: Set(excerpt),
            status: Set(status),
            version: Set(next_version),
            created_at: Set(Utc::now()),
            ..Default::default()
        };

        active.insert(&self.db).await.map_err_string()
    }

    async fn find_by_post_id(&self, post_id: i32) -> Result<Vec<PostRevision>, String> {
        Entity::find()
            .filter(Column::PostId.eq(post_id))
            .order_by_desc(Column::Version)
            .all(&self.db)
            .await
            .map_err_string()
    }

    async fn find_by_id(&self, id: i32) -> Result<Option<PostRevision>, String> {
        Entity::find_by_id(id).one(&self.db).await.map_err_string()
    }

    async fn find_by_post_id_and_version(
        &self,
        post_id: i32,
        version: i32,
    ) -> Result<Option<PostRevision>, String> {
        Entity::find()
            .filter(Column::PostId.eq(post_id))
            .filter(Column::Version.eq(version))
            .one(&self.db)
            .await
            .map_err_string()
    }
}
