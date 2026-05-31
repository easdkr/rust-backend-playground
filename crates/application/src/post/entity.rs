use chrono::{DateTime, Utc};

use super::error::PostError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostStatus {
    Draft,
    Published,
}

impl PostStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            PostStatus::Draft => "draft",
            PostStatus::Published => "published",
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "draft" => Ok(PostStatus::Draft),
            "published" => Ok(PostStatus::Published),
            other => Err(format!("Unknown post status: {other}")),
        }
    }
}

/// 도메인 엔티티 — 발행·상태 전환 등 비즈니스 규칙만 담습니다.
#[derive(Debug, Clone)]
pub struct Post {
    pub id: Option<i32>,
    pub title: String,
    pub content: String,
    pub status: PostStatus,
    pub user_id: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
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
        }
    }

    /// 초안 상태의 포스트만 발행할 수 있습니다.
    pub fn publish(&mut self) -> Result<(), PostError> {
        if self.status == PostStatus::Published {
            return Err(PostError::Domain("Post is already published".to_string()));
        }
        self.status = PostStatus::Published;
        Ok(())
    }

    pub fn apply_update(&mut self, title: Option<String>, content: Option<String>) {
        if let Some(t) = title {
            self.title = t;
        }
        if let Some(c) = content {
            self.content = c;
        }
    }
}
