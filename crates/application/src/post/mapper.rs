use infrastructure::persistence::seaorm::post::Post as PostRecord;

use super::entity::{Post, PostStatus};
use super::error::PostError;

pub fn domain_from_record(record: PostRecord) -> Result<Post, PostError> {
    let status = PostStatus::parse(&record.status)
        .map_err(|e| PostError::Internal(format!("Invalid persisted status: {e}")))?;

    Ok(Post {
        id: Some(record.id),
        title: record.title,
        content: record.content,
        status,
        created_at: Some(record.created_at),
    })
}
