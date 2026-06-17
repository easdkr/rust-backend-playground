use std::sync::Arc;

use infrastructure::persistence::seaorm::like_repository::LikeRepository;

use super::dto::LikeResult;
use super::error::LikeError;

pub struct LikeService {
    repo: Arc<dyn LikeRepository>,
}

impl LikeService {
    pub fn new(repo: Arc<dyn LikeRepository>) -> Self {
        Self { repo }
    }

    pub async fn toggle(&self, post_id: i32, user_id: String) -> Result<LikeResult, LikeError> {
        let liked = self
            .repo
            .toggle(post_id, user_id)
            .await
            .map_err(LikeError::Internal)?;

        let count = self
            .repo
            .get_count(post_id)
            .await
            .map_err(LikeError::Internal)?;

        Ok(LikeResult {
            liked,
            like_count: count as i64,
        })
    }

    pub async fn is_liked(&self, post_id: i32, user_id: &str) -> Result<bool, LikeError> {
        self.repo
            .is_liked(post_id, user_id)
            .await
            .map_err(LikeError::Internal)
    }

    pub async fn get_count(&self, post_id: i32) -> Result<i64, LikeError> {
        let count = self
            .repo
            .get_count(post_id)
            .await
            .map_err(LikeError::Internal)?;
        Ok(count as i64)
    }
}
