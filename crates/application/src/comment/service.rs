use std::sync::Arc;

use infrastructure::persistence::seaorm::comment_repository::CommentRepository;
use infrastructure::persistence::seaorm::post_repository::PostRepository;

use crate::notification::dto::CreateNotificationCmd;
use crate::notification::service::NotificationService;
use crate::pagination::CursorPage;

use super::dto::{CommentThreadDto, CreateCommentCmd, ListCommentsQuery, UpdateCommentCmd};
use super::entity::Comment;
use super::error::CommentError;
use super::mapper::domain_from_record;

pub struct CommentService {
    comment_repo: Arc<dyn CommentRepository>,
    post_repo: Arc<dyn PostRepository>,
    notification_service: Option<Arc<NotificationService>>,
}

impl CommentService {
    pub fn new(
        comment_repo: Arc<dyn CommentRepository>,
        post_repo: Arc<dyn PostRepository>,
        notification_service: Option<Arc<NotificationService>>,
    ) -> Self {
        Self {
            comment_repo,
            post_repo,
            notification_service,
        }
    }

    pub async fn create_root(
        &self,
        current_user_id: String,
        post_id: i32,
        cmd: CreateCommentCmd,
    ) -> Result<Comment, CommentError> {
        self.verify_post_exists(post_id).await?;
        let comment = Comment::new_root(post_id, current_user_id.clone(), cmd.content);
        let saved = self.create(comment).await?;
        self.notify_post_author(post_id, &current_user_id).await;
        Ok(saved)
    }

    pub async fn create_reply(
        &self,
        current_user_id: String,
        post_id: i32,
        parent_comment_id: i32,
        cmd: CreateCommentCmd,
    ) -> Result<Comment, CommentError> {
        self.verify_post_exists(post_id).await?;
        let parent = self.get_comment_in_post(post_id, parent_comment_id).await?;
        let comment = Comment::new_reply(
            post_id,
            parent_comment_id,
            parent.depth,
            current_user_id.clone(),
            cmd.content,
        )?;
        let saved = self.create(comment).await?;
        self.notify_post_author(post_id, &current_user_id).await;
        Ok(saved)
    }

    pub async fn list(
        &self,
        post_id: i32,
        query: ListCommentsQuery,
    ) -> Result<CursorPage<CommentThreadDto>, CommentError> {
        query.validate().map_err(CommentError::Domain)?;
        self.verify_post_exists(post_id).await?;

        let result = self
            .comment_repo
            .find_root_page(post_id, query.cursor, query.limit)
            .await
            .map_err(CommentError::repo)?;

        let roots = self.map_records(result.data)?;
        let root_ids = roots
            .iter()
            .map(|comment| comment.id.expect("persisted comment"))
            .collect::<Vec<_>>();
        let descendants = self
            .comment_repo
            .find_descendants_for_roots(post_id, &root_ids)
            .await
            .map_err(CommentError::repo)?;
        let descendants = self.map_records(descendants)?;

        Ok(CursorPage::new(
            CommentThreadDto::from_roots_and_descendants(roots, descendants),
            result.next_cursor,
            result.has_more,
        ))
    }

    pub async fn recent_threads(
        &self,
        post_id: i32,
        limit: usize,
    ) -> Result<(Vec<CommentThreadDto>, u64, bool), CommentError> {
        let roots = self
            .comment_repo
            .find_recent_roots(post_id, limit)
            .await
            .map_err(CommentError::repo)?;
        let count = self
            .comment_repo
            .count_by_post(post_id)
            .await
            .map_err(CommentError::repo)?;

        let root_comments = self.map_records(roots.data)?;
        let root_ids = root_comments
            .iter()
            .map(|comment| comment.id.expect("persisted comment"))
            .collect::<Vec<_>>();
        let descendants = self
            .comment_repo
            .find_descendants_for_roots(post_id, &root_ids)
            .await
            .map_err(CommentError::repo)?;
        let descendants = self.map_records(descendants)?;

        Ok((
            CommentThreadDto::from_roots_and_descendants(root_comments, descendants),
            count,
            roots.has_more,
        ))
    }

    pub async fn get(&self, post_id: i32, id: i32) -> Result<Comment, CommentError> {
        self.verify_post_exists(post_id).await?;
        self.get_comment_in_post(post_id, id).await
    }

    pub async fn update(
        &self,
        post_id: i32,
        id: i32,
        current_user_id: &str,
        cmd: UpdateCommentCmd,
    ) -> Result<Comment, CommentError> {
        self.verify_post_exists(post_id).await?;
        let mut comment = self.get_comment_in_post(post_id, id).await?;
        Self::verify_ownership(&comment, current_user_id)?;
        comment.apply_update(cmd.content);

        let saved = self
            .comment_repo
            .update(id, comment.content)
            .await
            .map_err(CommentError::repo)?;
        domain_from_record(saved)
    }

    pub async fn delete(
        &self,
        post_id: i32,
        id: i32,
        current_user_id: &str,
    ) -> Result<(), CommentError> {
        self.verify_post_exists(post_id).await?;
        let comment = self.get_comment_in_post(post_id, id).await?;
        Self::verify_ownership(&comment, current_user_id)?;

        let deleted = self
            .comment_repo
            .delete(id)
            .await
            .map_err(CommentError::repo)?;
        if !deleted {
            return Err(CommentError::NotFound(id));
        }
        Ok(())
    }

    async fn create(&self, comment: Comment) -> Result<Comment, CommentError> {
        let saved = self
            .comment_repo
            .create(
                comment.post_id,
                comment.parent_comment_id,
                comment.user_id,
                comment.content,
                comment.depth,
            )
            .await
            .map_err(CommentError::repo)?;
        domain_from_record(saved)
    }

    async fn verify_post_exists(&self, post_id: i32) -> Result<(), CommentError> {
        self.post_repo
            .find_by_id(post_id)
            .await
            .map_err(CommentError::repo)?
            .ok_or(CommentError::PostNotFound(post_id))?;
        Ok(())
    }

    async fn get_comment_in_post(&self, post_id: i32, id: i32) -> Result<Comment, CommentError> {
        let record = self
            .comment_repo
            .find_by_id(id)
            .await
            .map_err(CommentError::repo)?
            .ok_or(CommentError::NotFound(id))?;

        if record.post_id != post_id {
            return Err(CommentError::NotFound(id));
        }

        domain_from_record(record)
    }

    fn verify_ownership(comment: &Comment, current_user_id: &str) -> Result<(), CommentError> {
        if comment.user_id == current_user_id {
            Ok(())
        } else {
            Err(CommentError::OwnershipError)
        }
    }

    fn map_records(
        &self,
        records: Vec<infrastructure::persistence::seaorm::comment::Comment>,
    ) -> Result<Vec<Comment>, CommentError> {
        records.into_iter().map(domain_from_record).collect()
    }

    async fn notify_post_author(&self, post_id: i32, commenter_id: &str) {
        if let Some(ref svc) = self.notification_service {
            let post = match self.post_repo.find_by_id(post_id).await {
                Ok(Some(p)) => p,
                _ => return,
            };
            if post.user_id == commenter_id {
                return;
            }
            let _ = svc
                .create(CreateNotificationCmd {
                    user_id: post.user_id,
                    notification_type: "comment".to_string(),
                    title: "새 댓글".to_string(),
                    body: "게시글에 새 댓글이 달렸습니다.".to_string(),
                    data: Some(serde_json::json!({
                        "post_id": post_id,
                    })),
                })
                .await;
        }
    }
}
