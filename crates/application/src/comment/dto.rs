use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use super::entity::Comment;
use crate::pagination;
use crate::validation;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateCommentCmd {
    #[validate(custom(function = "validation::non_empty_trimmed"))]
    pub content: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateCommentCmd {
    #[validate(custom(function = "validation::non_empty_trimmed"))]
    pub content: String,
}

pub const COMMENT_DEFAULT_LIMIT: usize = 20;
pub const COMMENT_MIN_LIMIT: usize = 1;
pub const COMMENT_MAX_LIMIT: usize = 100;

fn default_comment_limit() -> usize {
    COMMENT_DEFAULT_LIMIT
}

#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListCommentsQuery {
    pub cursor: Option<i32>,
    #[serde(default = "default_comment_limit")]
    pub limit: usize,
}

impl ListCommentsQuery {
    pub fn validate(&self) -> Result<(), String> {
        pagination::validate_limit(self.limit, COMMENT_MIN_LIMIT, COMMENT_MAX_LIMIT)
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CommentDto {
    pub id: i32,
    pub post_id: i32,
    pub parent_comment_id: Option<i32>,
    pub user_id: String,
    pub content: String,
    pub depth: i32,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Comment> for CommentDto {
    fn from(comment: Comment) -> Self {
        Self {
            id: comment.id.expect("persisted comment"),
            post_id: comment.post_id,
            parent_comment_id: comment.parent_comment_id,
            user_id: comment.user_id,
            content: comment.content,
            depth: comment.depth,
            created_at: comment.created_at.expect("persisted comment").to_rfc3339(),
            updated_at: comment.updated_at.expect("persisted comment").to_rfc3339(),
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CommentThreadDto {
    pub comment: CommentDto,
    pub replies: Vec<CommentReplyDto>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CommentReplyDto {
    pub comment: CommentDto,
    pub replies: Vec<CommentDto>,
}

impl CommentThreadDto {
    pub fn from_roots_and_descendants(roots: Vec<Comment>, descendants: Vec<Comment>) -> Vec<Self> {
        let mut by_parent = descendants.into_iter().fold(
            HashMap::<i32, Vec<Comment>>::new(),
            |mut grouped, comment| {
                if let Some(parent_id) = comment.parent_comment_id {
                    grouped.entry(parent_id).or_default().push(comment);
                }
                grouped
            },
        );

        roots
            .into_iter()
            .map(|root| Self::from_comment(root, &mut by_parent))
            .collect()
    }

    fn from_comment(comment: Comment, by_parent: &mut HashMap<i32, Vec<Comment>>) -> Self {
        let root_id = comment.id.expect("persisted comment");
        let replies = by_parent
            .remove(&root_id)
            .unwrap_or_default()
            .into_iter()
            .map(|reply| {
                let reply_id = reply.id.expect("persisted comment");
                let nested_replies = by_parent
                    .remove(&reply_id)
                    .unwrap_or_default()
                    .into_iter()
                    .map(CommentDto::from)
                    .collect();

                CommentReplyDto {
                    comment: reply.into(),
                    replies: nested_replies,
                }
            })
            .collect();

        CommentThreadDto {
            comment: comment.into(),
            replies,
        }
    }
}
