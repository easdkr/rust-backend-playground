use async_trait::async_trait;
use chrono::Utc;
use libs::error::IntoStringErr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, Set,
};

use super::post::{ActiveModel, Entity, Post};

#[derive(Debug, Clone)]
pub struct CursorPaginationResult<T> {
    pub data: Vec<T>,
    pub next_cursor: Option<i32>,
    pub has_more: bool,
}

#[async_trait]
pub trait PostRepository: Send + Sync {
    async fn find_all(&self) -> Result<Vec<Post>, String>;
    async fn find_many(
        &self,
        cursor: Option<i32>,
        limit: usize,
    ) -> Result<CursorPaginationResult<Post>, String>;
    async fn find_by_id(&self, id: i32) -> Result<Option<Post>, String>;
    async fn create(
        &self,
        title: String,
        content: String,
        status: String,
        user_id: String,
    ) -> Result<Post, String>;
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

    async fn find_many(
        &self,
        cursor: Option<i32>,
        limit: usize,
    ) -> Result<CursorPaginationResult<Post>, String> {
        let limit_plus_one = limit + 1;

        let mut query = Entity::find().order_by_desc(super::post::Column::Id);

        if let Some(cursor_id) = cursor {
            query = query.filter(super::post::Column::Id.lt(cursor_id));
        }

        let mut posts = query
            .limit(limit_plus_one as u64)
            .all(&self.db)
            .await
            .map_err_string()?;

        let has_more = posts.len() > limit;
        if has_more {
            posts.pop();
        }

        let next_cursor = if has_more {
            posts.last().map(|p| p.id)
        } else {
            None
        };

        Ok(CursorPaginationResult {
            data: posts,
            next_cursor,
            has_more,
        })
    }

    async fn find_by_id(&self, id: i32) -> Result<Option<Post>, String> {
        Entity::find_by_id(id).one(&self.db).await.map_err_string()
    }

    async fn create(
        &self,
        title: String,
        content: String,
        status: String,
        user_id: String,
    ) -> Result<Post, String> {
        let active = ActiveModel {
            title: Set(title),
            content: Set(content),
            status: Set(status),
            user_id: Set(user_id),
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
        Entity::find().count(&self.db).await.map_err_string()
    }
}
