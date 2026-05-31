use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

use super::entity::Post;

fn non_empty_trimmed(value: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        let mut err = ValidationError::new("non_empty_trimmed");
        err.message = Some("must not be empty".into());
        return Err(err);
    }
    Ok(())
}

fn validate_update_post_cmd(cmd: &UpdatePostCmd) -> Result<(), ValidationError> {
    if let Some(ref title) = cmd.title {
        non_empty_trimmed(title)?;
    }
    Ok(())
}

/// class-validator처럼 필드/구조체에 규칙을 선언합니다 (`#[validate(...)]`).
#[derive(Debug, Deserialize, Validate)]
pub struct CreatePostCmd {
    #[validate(custom(function = "non_empty_trimmed"))]
    pub title: String,
    pub content: String,
}

impl CreatePostCmd {
    pub fn into_domain(self) -> Post {
        Post::new(self.title, self.content)
    }
}

#[derive(Debug, Deserialize, Validate)]
#[validate(schema(function = "validate_update_post_cmd"))]
pub struct UpdatePostCmd {
    pub title: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Serialize)]
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
