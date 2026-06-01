use infrastructure::persistence::seaorm::comment::Comment as CommentRecord;

use super::entity::Comment;
use super::error::CommentError;

pub fn domain_from_record(record: CommentRecord) -> Result<Comment, CommentError> {
    if record.depth < 0 {
        return Err(CommentError::Internal(format!(
            "Invalid persisted comment depth: {}",
            record.depth
        )));
    }

    Ok(Comment {
        id: Some(record.id),
        post_id: record.post_id,
        parent_comment_id: record.parent_comment_id,
        user_id: record.user_id,
        content: record.content,
        depth: record.depth,
        created_at: Some(record.created_at),
        updated_at: Some(record.updated_at),
    })
}
