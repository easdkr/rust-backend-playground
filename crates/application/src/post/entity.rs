use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::error::PostError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum PostStatus {
    Draft,
    Published,
    Archived,
}

impl PostStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            PostStatus::Draft => "draft",
            PostStatus::Published => "published",
            PostStatus::Archived => "archived",
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "draft" => Ok(PostStatus::Draft),
            "published" => Ok(PostStatus::Published),
            "archived" => Ok(PostStatus::Archived),
            other => Err(format!("Unknown post status: {other}")),
        }
    }
}

/// 도메인 엔티티 — 발행·상태 전환 등 비즈니스 규칙만 담습니다.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: Option<i32>,
    pub title: String,
    pub content: String,
    pub status: PostStatus,
    pub user_id: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
    pub slug: Option<String>,
    pub excerpt: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub view_count: i32,
    pub like_count: i32,
}

impl Post {
    pub fn new(title: String, content: String, user_id: String) -> Self {
        Self {
            id: None,
            title,
            content,
            status: PostStatus::Draft,
            user_id: Some(user_id),
            created_at: None,
            updated_at: Utc::now(),
            slug: None,
            excerpt: None,
            published_at: None,
            deleted_at: None,
            view_count: 0,
            like_count: 0,
        }
    }

    /// 초안 상태의 포스트만 발행할 수 있습니다.
    pub fn publish(&mut self) -> Result<(), PostError> {
        if self.status == PostStatus::Published {
            return Err(PostError::Domain("Post is already published".to_string()));
        }
        if self.status == PostStatus::Archived {
            return Err(PostError::Domain(
                "Cannot publish an archived post. Restore it first.".to_string(),
            ));
        }
        self.status = PostStatus::Published;
        self.published_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    /// 발행된 포스트를 초안으로 되돌립니다.
    pub fn unpublish(&mut self) -> Result<(), PostError> {
        if self.status != PostStatus::Published {
            return Err(PostError::Domain(
                "Only published posts can be unpublished".to_string(),
            ));
        }
        self.status = PostStatus::Draft;
        self.published_at = None;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// 발행된 포스트를 보관(Archived) 상태로 전환합니다.
    pub fn archive(&mut self) -> Result<(), PostError> {
        if self.status != PostStatus::Published {
            return Err(PostError::Domain(
                "Only published posts can be archived".to_string(),
            ));
        }
        self.status = PostStatus::Archived;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// 보관 상태의 포스트를 다시 발행 상태로 복원합니다.
    pub fn restore(&mut self) -> Result<(), PostError> {
        if self.status != PostStatus::Archived {
            return Err(PostError::Domain(
                "Only archived posts can be restored".to_string(),
            ));
        }
        self.status = PostStatus::Published;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// 도메인 단의 소프트 삭제 마킹. 실제 DB 반영은 리포지토리에서 처리합니다.
    pub fn mark_soft_deleted(&mut self) {
        self.deleted_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    pub fn apply_update(
        &mut self,
        title: Option<String>,
        content: Option<String>,
        excerpt: Option<String>,
        slug: Option<String>,
    ) {
        if let Some(t) = title {
            self.title = t;
        }
        if let Some(c) = content {
            self.content = c;
        }
        if let Some(e) = excerpt {
            self.excerpt = Some(e);
        }
        if let Some(s) = slug {
            self.slug = Some(s);
        }
        self.updated_at = Utc::now();
    }

    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
}

/// 작성자 정보 — Post와 left join된 결과를 표현합니다.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostAuthor {
    pub id: String,
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostWithAuthor {
    pub post: Post,
    pub author: Option<PostAuthor>,
}
