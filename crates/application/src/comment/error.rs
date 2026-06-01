use crate::error::DomainError;

#[derive(Debug)]
pub enum CommentError {
    PostNotFound(i32),
    NotFound(i32),
    OwnershipError,
    Domain(String),
    Internal(String),
}

impl std::fmt::Display for CommentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommentError::PostNotFound(id) => write!(f, "Post with id {id} not found"),
            CommentError::NotFound(id) => write!(f, "Comment with id {id} not found"),
            CommentError::OwnershipError => write!(f, "Not the owner of this comment"),
            CommentError::Domain(msg) => write!(f, "{msg}"),
            CommentError::Internal(msg) => write!(f, "Internal error: {msg}"),
        }
    }
}

impl DomainError for CommentError {
    fn internal(msg: impl Into<String>) -> Self {
        CommentError::Internal(msg.into())
    }

    fn is_not_found(&self) -> bool {
        matches!(
            self,
            CommentError::PostNotFound(_) | CommentError::NotFound(_)
        )
    }
}

impl CommentError {
    pub fn repo(message: String) -> Self {
        Self::Internal(message)
    }
}
