use chrono::{DateTime, Utc};

use super::error::CommentError;

pub const MAX_COMMENT_DEPTH: i32 = 2;

#[derive(Debug, Clone)]
pub struct Comment {
    pub id: Option<i32>,
    pub post_id: i32,
    pub parent_comment_id: Option<i32>,
    pub user_id: String,
    pub content: String,
    pub depth: i32,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Comment {
    pub fn new_root(post_id: i32, user_id: String, content: String) -> Self {
        Self {
            id: None,
            post_id,
            parent_comment_id: None,
            user_id,
            content,
            depth: 0,
            created_at: None,
            updated_at: None,
        }
    }

    pub fn new_reply(
        post_id: i32,
        parent_comment_id: i32,
        parent_depth: i32,
        user_id: String,
        content: String,
    ) -> Result<Self, CommentError> {
        let depth = parent_depth + 1;
        if depth > MAX_COMMENT_DEPTH {
            return Err(CommentError::Domain(format!(
                "Comment depth cannot exceed {MAX_COMMENT_DEPTH}"
            )));
        }

        Ok(Self {
            id: None,
            post_id,
            parent_comment_id: Some(parent_comment_id),
            user_id,
            content,
            depth,
            created_at: None,
            updated_at: None,
        })
    }

    pub fn apply_update(&mut self, content: String) {
        self.content = content;
    }
}
