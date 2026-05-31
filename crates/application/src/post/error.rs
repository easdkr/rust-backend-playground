use crate::error::DomainError;

#[derive(Debug)]
pub enum PostError {
    NotFound(i32),
    OwnershipError,
    Domain(String),
    Internal(String),
}

impl std::fmt::Display for PostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PostError::NotFound(id) => write!(f, "Post with id {id} not found"),
            PostError::OwnershipError => write!(f, "Not the owner of this post"),
            PostError::Domain(msg) => write!(f, "{msg}"),
            PostError::Internal(msg) => write!(f, "Internal error: {msg}"),
        }
    }
}

impl DomainError for PostError {
    fn internal(msg: impl Into<String>) -> Self {
        PostError::Internal(msg.into())
    }

    fn is_not_found(&self) -> bool {
        matches!(self, PostError::NotFound(_))
    }
}

impl PostError {
    pub fn repo(message: String) -> Self {
        Self::Internal(message)
    }
}
