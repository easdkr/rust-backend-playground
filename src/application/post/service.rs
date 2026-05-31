use std::sync::Arc;
use crate::domain::post::repository::PostRepository;
use crate::domain::post::entity::Post;
use super::dto::{CreatePostCmd, UpdatePostCmd, PostDto};

pub struct PostService {
    repo: Arc<dyn PostRepository>,
}

impl PostService {
    pub fn new(repo: Arc<dyn PostRepository>) -> Self {
        Self { repo }
    }

    pub async fn create(&self, cmd: CreatePostCmd) -> Result<PostDto, String> {
        let post = Post::new(cmd.title, cmd.content)?;
        let saved = self.repo.save(post).await?;
        Ok(PostDto::from(saved))
    }

    pub async fn list(&self) -> Result<Vec<PostDto>, String> {
        let posts = self.repo.find_all().await?;
        Ok(posts.into_iter().map(PostDto::from).collect())
    }

    pub async fn get(&self, id: i32) -> Result<PostDto, String> {
        let post = self.repo.find_by_id(id).await?
            .ok_or_else(|| format!("Post with id {} not found", id))?;
        Ok(PostDto::from(post))
    }

    pub async fn update(&self, id: i32, cmd: UpdatePostCmd) -> Result<PostDto, String> {
        let mut post = self.repo.find_by_id(id).await?
            .ok_or_else(|| format!("Post with id {} not found", id))?;
            
        post.update(cmd.title, cmd.content)?;
        let saved = self.repo.save(post).await?;
        Ok(PostDto::from(saved))
    }

    pub async fn delete(&self, id: i32) -> Result<(), String> {
        let deleted = self.repo.delete(id).await?;
        if !deleted {
            return Err(format!("Post with id {} not found", id));
        }
        Ok(())
    }
}
