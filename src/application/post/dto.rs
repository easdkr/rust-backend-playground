use serde::{Deserialize, Serialize};
use crate::domain::post::entity::Post;

#[derive(Debug, Deserialize)]
pub struct CreatePostCmd {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePostCmd {
    pub title: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PostDto {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub created_at: String,
}

impl From<Post> for PostDto {
    fn from(post: Post) -> Self {
        Self {
            id: post.id.unwrap_or(0),
            title: post.title,
            content: post.content,
            created_at: post.created_at.to_rfc3339(),
        }
    }
}
