use async_trait::async_trait;
use chrono::Utc;
use libs::error::IntoStringErr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, Set,
};

use super::comment::{ActiveModel, Comment, Entity};

#[derive(Debug, Clone)]
pub struct CursorPaginationResult<T> {
    pub data: Vec<T>,
    pub next_cursor: Option<i32>,
    pub has_more: bool,
}

#[async_trait]
pub trait CommentRepository: Send + Sync {
    async fn find_root_page(
        &self,
        post_id: i32,
        cursor: Option<i32>,
        limit: usize,
    ) -> Result<CursorPaginationResult<Comment>, String>;
    async fn find_recent_roots(
        &self,
        post_id: i32,
        limit: usize,
    ) -> Result<CursorPaginationResult<Comment>, String>;
    async fn find_descendants_for_roots(
        &self,
        post_id: i32,
        root_ids: &[i32],
    ) -> Result<Vec<Comment>, String>;
    async fn find_by_id(&self, id: i32) -> Result<Option<Comment>, String>;
    async fn count_by_post(&self, post_id: i32) -> Result<u64, String>;
    async fn create(
        &self,
        post_id: i32,
        parent_comment_id: Option<i32>,
        user_id: String,
        content: String,
        depth: i32,
    ) -> Result<Comment, String>;
    async fn update(&self, id: i32, content: String) -> Result<Comment, String>;
    async fn delete(&self, id: i32) -> Result<bool, String>;
}

pub struct SeaOrmCommentRepository {
    db: DatabaseConnection,
}

impl SeaOrmCommentRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl CommentRepository for SeaOrmCommentRepository {
    async fn find_root_page(
        &self,
        post_id: i32,
        cursor: Option<i32>,
        limit: usize,
    ) -> Result<CursorPaginationResult<Comment>, String> {
        let limit_plus_one = limit + 1;

        let mut query = Entity::find()
            .filter(super::comment::Column::PostId.eq(post_id))
            .filter(super::comment::Column::ParentCommentId.is_null())
            .order_by_asc(super::comment::Column::Id);

        if let Some(cursor_id) = cursor {
            query = query.filter(super::comment::Column::Id.gt(cursor_id));
        }

        let mut comments = query
            .limit(limit_plus_one as u64)
            .all(&self.db)
            .await
            .map_err_string()?;

        let has_more = comments.len() > limit;
        if has_more {
            comments.pop();
        }

        let next_cursor = if has_more {
            comments.last().map(|comment| comment.id)
        } else {
            None
        };

        Ok(CursorPaginationResult {
            data: comments,
            next_cursor,
            has_more,
        })
    }

    async fn find_recent_roots(
        &self,
        post_id: i32,
        limit: usize,
    ) -> Result<CursorPaginationResult<Comment>, String> {
        let mut comments = Entity::find()
            .filter(super::comment::Column::PostId.eq(post_id))
            .filter(super::comment::Column::ParentCommentId.is_null())
            .order_by_desc(super::comment::Column::Id)
            .limit((limit + 1) as u64)
            .all(&self.db)
            .await
            .map_err_string()?;

        let has_more = comments.len() > limit;
        if has_more {
            comments.pop();
        }

        let next_cursor = if has_more {
            comments.last().map(|comment| comment.id)
        } else {
            None
        };

        Ok(CursorPaginationResult {
            data: comments,
            next_cursor,
            has_more,
        })
    }

    async fn find_descendants_for_roots(
        &self,
        post_id: i32,
        root_ids: &[i32],
    ) -> Result<Vec<Comment>, String> {
        if root_ids.is_empty() {
            return Ok(Vec::new());
        }

        let direct_replies = Entity::find()
            .filter(super::comment::Column::PostId.eq(post_id))
            .filter(super::comment::Column::ParentCommentId.is_in(root_ids.to_vec()))
            .order_by_asc(super::comment::Column::Id)
            .all(&self.db)
            .await
            .map_err_string()?;

        let direct_reply_ids = direct_replies
            .iter()
            .map(|comment| comment.id)
            .collect::<Vec<_>>();

        if direct_reply_ids.is_empty() {
            return Ok(direct_replies);
        }

        let nested_replies = Entity::find()
            .filter(super::comment::Column::PostId.eq(post_id))
            .filter(super::comment::Column::ParentCommentId.is_in(direct_reply_ids))
            .order_by_asc(super::comment::Column::Id)
            .all(&self.db)
            .await
            .map_err_string()?;

        let mut comments = direct_replies;
        comments.extend(nested_replies);
        comments.sort_by_key(|comment| comment.id);
        Ok(comments)
    }

    async fn find_by_id(&self, id: i32) -> Result<Option<Comment>, String> {
        Entity::find_by_id(id).one(&self.db).await.map_err_string()
    }

    async fn count_by_post(&self, post_id: i32) -> Result<u64, String> {
        Entity::find()
            .filter(super::comment::Column::PostId.eq(post_id))
            .count(&self.db)
            .await
            .map_err_string()
    }

    async fn create(
        &self,
        post_id: i32,
        parent_comment_id: Option<i32>,
        user_id: String,
        content: String,
        depth: i32,
    ) -> Result<Comment, String> {
        let now = Utc::now();
        let active = ActiveModel {
            post_id: Set(post_id),
            parent_comment_id: Set(parent_comment_id),
            user_id: Set(user_id),
            content: Set(content),
            depth: Set(depth),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };
        active.insert(&self.db).await.map_err_string()
    }

    async fn update(&self, id: i32, content: String) -> Result<Comment, String> {
        let active = ActiveModel {
            id: Set(id),
            content: Set(content),
            updated_at: Set(Utc::now()),
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
}
