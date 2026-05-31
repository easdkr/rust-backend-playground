use std::sync::Arc;

use infrastructure::persistence::seaorm::post_repository::PostRepository;

use super::dto::{CreatePostCmd, UpdatePostCmd};
use super::entity::Post;
use super::error::PostError;
use super::mapper::domain_from_record;

pub struct PostService {
    repo: Arc<dyn PostRepository>,
}

impl PostService {
    pub fn new(repo: Arc<dyn PostRepository>) -> Self {
        Self { repo }
    }

    pub async fn create(&self, cmd: CreatePostCmd) -> Result<Post, PostError> {
        let post = cmd.into_domain();
        let saved = self
            .repo
            .create(post.title, post.content, post.status.as_str().to_string())
            .await
            .map_err(PostError::repo)?;
        domain_from_record(saved)
    }

    pub async fn list(&self) -> Result<Vec<Post>, PostError> {
        let records = self.repo.find_all().await.map_err(PostError::repo)?;
        records.into_iter().map(domain_from_record).collect()
    }

    pub async fn get(&self, id: i32) -> Result<Post, PostError> {
        let record = self
            .repo
            .find_by_id(id)
            .await
            .map_err(PostError::repo)?
            .ok_or(PostError::NotFound(id))?;
        domain_from_record(record)
    }

    pub async fn update(&self, id: i32, cmd: UpdatePostCmd) -> Result<Post, PostError> {
        let record = self
            .repo
            .find_by_id(id)
            .await
            .map_err(PostError::repo)?
            .ok_or(PostError::NotFound(id))?;

        let mut post = domain_from_record(record)?;
        post.apply_update(cmd.title, cmd.content);

        let saved = self
            .repo
            .update(
                id,
                post.title,
                post.content,
                post.status.as_str().to_string(),
            )
            .await
            .map_err(PostError::repo)?;
        domain_from_record(saved)
    }

    pub async fn publish(&self, id: i32) -> Result<Post, PostError> {
        let record = self
            .repo
            .find_by_id(id)
            .await
            .map_err(PostError::repo)?
            .ok_or(PostError::NotFound(id))?;

        let mut post = domain_from_record(record)?;
        post.publish()?;

        let saved = self
            .repo
            .update(
                id,
                post.title,
                post.content,
                post.status.as_str().to_string(),
            )
            .await
            .map_err(PostError::repo)?;
        domain_from_record(saved)
    }

    pub async fn delete(&self, id: i32) -> Result<(), PostError> {
        let deleted = self.repo.delete(id).await.map_err(PostError::repo)?;
        if !deleted {
            return Err(PostError::NotFound(id));
        }
        Ok(())
    }
}
