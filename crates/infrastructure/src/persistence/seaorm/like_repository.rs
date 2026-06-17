use async_trait::async_trait;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, Set,
    TransactionTrait,
};

use super::like::{ActiveModel as LikeActiveModel, Entity as LikeEntity};
use super::post::{ActiveModel as PostActiveModel, Entity as PostEntity};

#[async_trait]
pub trait LikeRepository: Send + Sync {
    async fn toggle(&self, post_id: i32, user_id: String) -> Result<bool, String>;
    async fn get_count(&self, post_id: i32) -> Result<u64, String>;
    async fn is_liked(&self, post_id: i32, user_id: &str) -> Result<bool, String>;
}

pub struct SeaOrmLikeRepository {
    db: DatabaseConnection,
}

impl SeaOrmLikeRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl LikeRepository for SeaOrmLikeRepository {
    async fn toggle(&self, post_id: i32, user_id: String) -> Result<bool, String> {
        let txn = self.db.begin().await.map_err(|e| e.to_string())?;

        let existing = LikeEntity::find()
            .filter(super::like::Column::PostId.eq(post_id))
            .filter(super::like::Column::UserId.eq(&user_id))
            .one(&txn)
            .await
            .map_err(|e| e.to_string())?;

        let liked = if let Some(like) = existing {
            LikeEntity::delete_by_id(like.id)
                .exec(&txn)
                .await
                .map_err(|e| e.to_string())?;
            false
        } else {
            let new_like = LikeActiveModel {
                post_id: Set(post_id),
                user_id: Set(user_id),
                ..Default::default()
            };
            LikeEntity::insert(new_like)
                .exec(&txn)
                .await
                .map_err(|e| e.to_string())?;
            true
        };

        // like_count 동기화
        let count = LikeEntity::find()
            .filter(super::like::Column::PostId.eq(post_id))
            .count(&txn)
            .await
            .map_err(|e| e.to_string())?;

        let post = PostEntity::find_by_id(post_id)
            .one(&txn)
            .await
            .map_err(|e| e.to_string())?
            .ok_or("Post not found")?;

        let mut post_am: PostActiveModel = post.into();
        post_am.like_count = Set(count as i32);
        PostEntity::update(post_am)
            .exec(&txn)
            .await
            .map_err(|e| e.to_string())?;

        txn.commit().await.map_err(|e| e.to_string())?;

        Ok(liked)
    }

    async fn get_count(&self, post_id: i32) -> Result<u64, String> {
        LikeEntity::find()
            .filter(super::like::Column::PostId.eq(post_id))
            .count(&self.db)
            .await
            .map_err(|e| e.to_string())
    }

    async fn is_liked(&self, post_id: i32, user_id: &str) -> Result<bool, String> {
        let count = LikeEntity::find()
            .filter(super::like::Column::PostId.eq(post_id))
            .filter(super::like::Column::UserId.eq(user_id))
            .count(&self.db)
            .await
            .map_err(|e| e.to_string())?;
        Ok(count > 0)
    }
}
