use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::{Validate, ValidationError};

use super::entity::{Post, PostAuthor, PostStatus, PostWithAuthor};
use crate::pagination;
use crate::validation;

const SLUG_MAX_LEN: usize = 140;
const SLUG_REGEX_ERR: &str =
    "slug must be lowercase alphanumeric with optional '-', '_' separators (max 140 chars)";

fn validate_slug(value: &str) -> Result<(), ValidationError> {
    if value.is_empty() || value.len() > SLUG_MAX_LEN {
        let mut err = ValidationError::new("slug_length");
        err.message = Some(SLUG_REGEX_ERR.into());
        return Err(err);
    }
    let valid = value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
        && !value.starts_with('-')
        && !value.ends_with('-');
    if !valid {
        let mut err = ValidationError::new("slug_format");
        err.message = Some(SLUG_REGEX_ERR.into());
        return Err(err);
    }
    Ok(())
}

fn validate_update_post_cmd(cmd: &UpdatePostCmd) -> Result<(), ValidationError> {
    if let Some(ref title) = cmd.title {
        validation::non_empty_trimmed(title)?;
    }
    if let Some(ref slug) = cmd.slug {
        validate_slug(slug)?;
    }
    if matches!(cmd.excerpt.as_deref(), Some(e) if e.len() > 500) {
        let mut err = ValidationError::new("excerpt_length");
        err.message = Some("excerpt must be 500 characters or fewer".into());
        return Err(err);
    }
    Ok(())
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreatePostCmd {
    #[validate(custom(function = "validation::non_empty_trimmed"))]
    pub title: String,
    pub content: String,
    #[validate(length(max = 500))]
    pub excerpt: Option<String>,
    #[validate(custom(function = "validate_slug"))]
    pub slug: Option<String>,
}

impl CreatePostCmd {
    pub fn into_domain(self, user_id: String) -> Post {
        let mut post = Post::new(self.title, self.content, user_id);
        post.excerpt = self.excerpt;
        post.slug = self.slug;
        post
    }
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[validate(schema(function = "validate_update_post_cmd"))]
pub struct UpdatePostCmd {
    pub title: Option<String>,
    pub content: Option<String>,
    #[validate(length(max = 500))]
    pub excerpt: Option<String>,
    #[validate(custom(function = "validate_slug"))]
    pub slug: Option<String>,
}

/// Post 목록 정렬 옵션
#[derive(Debug, Clone, Copy, Default, Deserialize, Hash, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PostSort {
    #[default]
    Newest,
    Oldest,
    RecentlyUpdated,
    TitleAsc,
    MostViewed,
}

impl PostSort {
    pub fn as_str(self) -> &'static str {
        match self {
            PostSort::Newest => "newest",
            PostSort::Oldest => "oldest",
            PostSort::RecentlyUpdated => "recently_updated",
            PostSort::TitleAsc => "title_asc",
            PostSort::MostViewed => "most_viewed",
        }
    }
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
    /// 필터: 게시 상태
    pub status: Option<PostStatus>,
    /// 필터: 작성자 ID
    pub author_id: Option<String>,
    /// 검색어 (제목/요약/본문 대상 ILIKE + tsvector 매칭)
    pub q: Option<String>,
    /// 정렬 기준
    #[serde(default)]
    pub sort: PostSort,
    /// 소프트 삭제된 항목 포함 여부
    pub include_deleted: Option<bool>,
}

impl ListPostsQuery {
    pub fn validate(&self) -> Result<(), String> {
        pagination::validate_limit(self.limit, POST_MIN_LIMIT, POST_MAX_LIMIT)?;
        if let Some(q) = &self.q {
            if q.trim().is_empty() {
                return Err("q must not be empty".to_string());
            }
            if q.len() > 200 {
                return Err("q must be 200 characters or fewer".to_string());
            }
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PostAuthorDto {
    pub id: String,
    pub username: String,
}

impl From<PostAuthor> for PostAuthorDto {
    fn from(a: PostAuthor) -> Self {
        Self {
            id: a.id,
            username: a.username,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PostDto {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub status: PostStatus,
    pub slug: Option<String>,
    pub excerpt: Option<String>,
    pub author: Option<PostAuthorDto>,
    pub created_at: String,
    pub updated_at: String,
    pub published_at: Option<String>,
    pub deleted_at: Option<String>,
    pub view_count: i32,
}

impl From<PostWithAuthor> for PostDto {
    fn from(wa: PostWithAuthor) -> Self {
        let post = wa.post;
        Self {
            id: post.id.expect("persisted post"),
            title: post.title,
            content: post.content,
            status: post.status,
            slug: post.slug,
            excerpt: post.excerpt,
            author: wa.author.map(PostAuthorDto::from),
            created_at: post.created_at.expect("persisted post").to_rfc3339(),
            updated_at: post.updated_at.to_rfc3339(),
            published_at: post.published_at.map(|t| t.to_rfc3339()),
            deleted_at: post.deleted_at.map(|t| t.to_rfc3339()),
            view_count: post.view_count,
        }
    }
}

impl From<Post> for PostDto {
    fn from(post: Post) -> Self {
        Self::from(PostWithAuthor { post, author: None })
    }
}

/// 일괄 작업 명령
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct BulkPostCmd {
    #[validate(length(min = 1, max = 100))]
    pub ids: Vec<i32>,
    pub action: BulkPostAction,
}

#[derive(Debug, Clone, Copy, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum BulkPostAction {
    Publish,
    Unpublish,
    Archive,
    Restore,
    Delete,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BulkPostResultItem {
    pub id: i32,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BulkPostResult {
    pub succeeded: Vec<i32>,
    pub failed: Vec<BulkPostResultItem>,
}
