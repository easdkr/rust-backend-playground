use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::{Validate, ValidationError};

use super::entity::Post;
use crate::comment::dto::CommentThreadDto;
use crate::pagination;
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

/// Post 목록 조회용 페이징 설정
pub const POST_DEFAULT_LIMIT: usize = 20;
pub const POST_MIN_LIMIT: usize = 1;
pub const POST_MAX_LIMIT: usize = 100;

fn default_post_limit() -> usize {
    POST_DEFAULT_LIMIT
}

#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListPostsQuery {
    pub cursor: Option<i32>,
    #[serde(default = "default_post_limit")]
    pub limit: usize,
}

impl ListPostsQuery {
    pub fn validate(&self) -> Result<(), String> {
        pagination::validate_limit(self.limit, POST_MIN_LIMIT, POST_MAX_LIMIT)
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

#[derive(Debug, Serialize, ToSchema)]
pub struct PostDetailDto {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub status: String,
    pub created_at: String,
    pub comments: Vec<CommentThreadDto>,
    pub comment_count: u64,
    pub has_more_comments: bool,
}

impl PostDetailDto {
    pub fn new(
        post: Post,
        comments: Vec<CommentThreadDto>,
        comment_count: u64,
        has_more_comments: bool,
    ) -> Self {
        let post = PostDto::from(post);
        Self {
            id: post.id,
            title: post.title,
            content: post.content,
            status: post.status,
            created_at: post.created_at,
            comments,
            comment_count,
            has_more_comments,
        }
    }
}
