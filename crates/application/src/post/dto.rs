use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::{Validate, ValidationError};

use super::entity::Post;
use crate::validation;

fn validate_update_post_cmd(cmd: &UpdatePostCmd) -> Result<(), ValidationError> {
    if let Some(ref title) = cmd.title {
        validation::non_empty_trimmed(title)?;
    }
    Ok(())
}

/// class-validator처럼 필드/구조체에 규칙을 선언합니다 (`#[validate(...)]`).
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreatePostCmd {
    #[validate(custom(function = "validation::non_empty_trimmed"))]
    pub title: String,
    pub content: String,
}

impl CreatePostCmd {
    pub fn into_domain(self, user_id: String) -> Post {
        Post::new(self.title, self.content, user_id)
    }
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[validate(schema(function = "validate_update_post_cmd"))]
pub struct UpdatePostCmd {
    pub title: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListPostsQuery {
    pub cursor: Option<i32>,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    20
}

impl ListPostsQuery {
    pub fn validate(&self) -> Result<(), String> {
        if self.limit == 0 || self.limit > 100 {
            return Err("limit must be between 1 and 100".to_string());
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PostDto {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub status: String,
    pub created_at: String,
}

impl From<Post> for PostDto {
    fn from(post: Post) -> Self {
        Self {
            id: post.id.expect("persisted post"),
            title: post.title,
            content: post.content,
            status: post.status.as_str().to_string(),
            created_at: post.created_at.expect("persisted post").to_rfc3339(),
        }
    }
}
