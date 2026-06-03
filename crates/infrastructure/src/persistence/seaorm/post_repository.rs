use async_trait::async_trait;
use chrono::Utc;
use libs::error::IntoStringErr;
use sea_orm::sea_query::extension::postgres::PgExpr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, JoinType, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, RelationTrait, Set,
};

use super::post::{ActiveModel, Column, Entity, Post, Relation};
use super::user::{Entity as UserEntity, Model as UserModel};

#[derive(Debug, Clone, Default)]
pub struct CursorPaginationResult<T> {
    pub data: Vec<T>,
    pub next_cursor: Option<i32>,
    pub has_more: bool,
}

/// application 계층이 채워주는 단순 필터. 도메인 enum은 string으로 받습니다.
#[derive(Debug, Clone, Default)]
pub struct PostListFilter {
    pub cursor: Option<i32>,
    pub limit: usize,
    pub status: Option<String>,
    pub author_id: Option<String>,
    pub q: Option<String>,
    /// "newest" | "oldest" | "recently_updated" | "title_asc" | "most_viewed"
    pub sort: String,
    pub include_deleted: bool,
}

#[derive(Debug, Clone)]
pub struct PostWithAuthorRow {
    pub post: Post,
    pub author: Option<UserModel>,
}

#[async_trait]
pub trait PostRepository: Send + Sync {
    async fn find_all(&self) -> Result<Vec<Post>, String>;
    async fn find_many(
        &self,
        cursor: Option<i32>,
        limit: usize,
    ) -> Result<CursorPaginationResult<Post>, String>;
    async fn find_many_with_author(
        &self,
        filter: PostListFilter,
    ) -> Result<CursorPaginationResult<PostWithAuthorRow>, String>;
    async fn find_by_id(&self, id: i32) -> Result<Option<Post>, String>;
    async fn find_by_id_with_author(&self, id: i32) -> Result<Option<PostWithAuthorRow>, String>;
    async fn find_by_slug(&self, slug: &str) -> Result<Option<Post>, String>;
    async fn find_by_slug_with_author(
        &self,
        slug: &str,
    ) -> Result<Option<PostWithAuthorRow>, String>;
    async fn slug_exists(&self, slug: &str, exclude_id: Option<i32>) -> Result<bool, String>;
    async fn increment_view_count(&self, id: i32) -> Result<(), String>;
    async fn create(
        &self,
        title: String,
        content: String,
        status: String,
        user_id: String,
        slug: Option<String>,
        excerpt: Option<String>,
    ) -> Result<Post, String>;
    async fn update(
        &self,
        id: i32,
        title: String,
        content: String,
        excerpt: Option<String>,
        slug: Option<String>,
        status: String,
    ) -> Result<Post, String>;
    async fn delete(&self, id: i32) -> Result<bool, String>;
    async fn soft_delete(&self, id: i32) -> Result<bool, String>;
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
            .order_by_desc(Column::CreatedAt)
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
        let mut query = Entity::find()
            .filter(Column::DeletedAt.is_null())
            .order_by_desc(Column::Id);
        if let Some(cursor_id) = cursor {
            query = query.filter(Column::Id.lt(cursor_id));
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

    async fn find_many_with_author(
        &self,
        filter: PostListFilter,
    ) -> Result<CursorPaginationResult<PostWithAuthorRow>, String> {
        let limit = filter.limit.max(1);
        let limit_plus_one = limit + 1;

        let mut query = Entity::find()
            .join(JoinType::LeftJoin, Relation::User.def())
            .order_by_desc(Column::Id);

        if !filter.include_deleted {
            query = query.filter(Column::DeletedAt.is_null());
        }

        if let Some(cursor_id) = filter.cursor {
            query = query.filter(Column::Id.lt(cursor_id));
        }

        if let Some(status) = &filter.status {
            if !status.is_empty() {
                query = query.filter(Column::Status.eq(status.clone()));
            }
        }

        if let Some(author) = &filter.author_id {
            if !author.is_empty() {
                query = query.filter(Column::UserId.eq(author.clone()));
            }
        }

        if let Some(q) = &filter.q {
            let trimmed = q.trim();
            if !trimmed.is_empty() {
                let pattern = format!("%{}%", trimmed);
                let pat2 = pattern.clone();
                let pat3 = pattern.clone();
                query = query.filter(
                    sea_orm::sea_query::Expr::col(Column::Title)
                        .ilike(pattern)
                        .or(sea_orm::sea_query::Expr::col(Column::Excerpt).ilike(pat2))
                        .or(sea_orm::sea_query::Expr::col(Column::Content).ilike(pat3)),
                );
            }
        }

        query = match filter.sort.as_str() {
            "oldest" => query
                .order_by_asc(Column::CreatedAt)
                .order_by_desc(Column::Id),
            "recently_updated" => query
                .order_by_desc(Column::UpdatedAt)
                .order_by_desc(Column::Id),
            "title_asc" => query.order_by_asc(Column::Title).order_by_desc(Column::Id),
            "most_viewed" => query
                .order_by_desc(Column::ViewCount)
                .order_by_desc(Column::Id),
            _ => query
                .order_by_desc(Column::CreatedAt)
                .order_by_desc(Column::Id),
        };

        let rows: Vec<(Post, Option<UserModel>)> = query
            .find_also_related(UserEntity)
            .limit(limit_plus_one as u64)
            .all(&self.db)
            .await
            .map_err_string()?;

        let mut mapped: Vec<PostWithAuthorRow> = rows
            .into_iter()
            .map(|(post, author)| PostWithAuthorRow { post, author })
            .collect();

        let has_more = mapped.len() > limit;
        if has_more {
            mapped.pop();
        }
        let next_cursor = if has_more {
            mapped.last().map(|w| w.post.id)
        } else {
            None
        };

        Ok(CursorPaginationResult {
            data: mapped,
            next_cursor,
            has_more,
        })
    }

    async fn find_by_id(&self, id: i32) -> Result<Option<Post>, String> {
        Entity::find_by_id(id).one(&self.db).await.map_err_string()
    }

    async fn find_by_id_with_author(&self, id: i32) -> Result<Option<PostWithAuthorRow>, String> {
        let row: Option<(Post, Option<UserModel>)> = Entity::find_by_id(id)
            .join(JoinType::LeftJoin, Relation::User.def())
            .find_also_related(UserEntity)
            .one(&self.db)
            .await
            .map_err_string()?;
        Ok(row.map(|(post, author)| PostWithAuthorRow { post, author }))
    }

    async fn find_by_slug(&self, slug: &str) -> Result<Option<Post>, String> {
        Entity::find()
            .filter(Column::Slug.eq(slug))
            .filter(Column::DeletedAt.is_null())
            .one(&self.db)
            .await
            .map_err_string()
    }

    async fn find_by_slug_with_author(
        &self,
        slug: &str,
    ) -> Result<Option<PostWithAuthorRow>, String> {
        let row: Option<(Post, Option<UserModel>)> = Entity::find()
            .filter(Column::Slug.eq(slug))
            .filter(Column::DeletedAt.is_null())
            .join(JoinType::LeftJoin, Relation::User.def())
            .find_also_related(UserEntity)
            .one(&self.db)
            .await
            .map_err_string()?;
        Ok(row.map(|(post, author)| PostWithAuthorRow { post, author }))
    }

    async fn slug_exists(&self, slug: &str, exclude_id: Option<i32>) -> Result<bool, String> {
        let mut q = Entity::find().filter(Column::Slug.eq(slug));
        if let Some(id) = exclude_id {
            q = q.filter(Column::Id.ne(id));
        }
        let count = q.count(&self.db).await.map_err_string()?;
        Ok(count > 0)
    }

    async fn increment_view_count(&self, id: i32) -> Result<(), String> {
        use sea_orm::sea_query::Expr;
        Entity::update_many()
            .col_expr(Column::ViewCount, Expr::col(Column::ViewCount).add(1))
            .filter(Column::Id.eq(id))
            .exec(&self.db)
            .await
            .map_err_string()?;
        Ok(())
    }

    async fn create(
        &self,
        title: String,
        content: String,
        status: String,
        user_id: String,
        slug: Option<String>,
        excerpt: Option<String>,
    ) -> Result<Post, String> {
        let now = Utc::now();
        let active = ActiveModel {
            title: Set(title),
            content: Set(content),
            status: Set(status),
            user_id: Set(user_id),
            slug: Set(slug),
            excerpt: Set(excerpt),
            created_at: Set(now),
            updated_at: Set(now),
            view_count: Set(0),
            ..Default::default()
        };
        active.insert(&self.db).await.map_err_string()
    }

    async fn update(
        &self,
        id: i32,
        title: String,
        content: String,
        excerpt: Option<String>,
        slug: Option<String>,
        status: String,
    ) -> Result<Post, String> {
        let now = Utc::now();
        let active = ActiveModel {
            id: Set(id),
            title: Set(title),
            content: Set(content),
            excerpt: Set(excerpt),
            slug: Set(slug),
            status: Set(status),
            updated_at: Set(now),
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

    async fn soft_delete(&self, id: i32) -> Result<bool, String> {
        use sea_orm::sea_query::Expr;
        let now = Utc::now();
        let res = Entity::update_many()
            .col_expr(Column::DeletedAt, Expr::value(Some(now)))
            .col_expr(Column::UpdatedAt, Expr::value(now))
            .filter(Column::Id.eq(id))
            .exec(&self.db)
            .await
            .map_err_string()?;
        Ok(res.rows_affected > 0)
    }

    async fn count(&self) -> Result<u64, String> {
        Entity::find()
            .filter(Column::DeletedAt.is_null())
            .count(&self.db)
            .await
            .map_err_string()
    }
}
