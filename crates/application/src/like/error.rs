use std::fmt;

#[derive(Debug, Clone)]
pub enum LikeError {
    NotFound(i32),
    AlreadyExists,
    Internal(String),
}

impl fmt::Display for LikeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LikeError::NotFound(id) => write!(f, "Like not found for post {id}"),
            LikeError::AlreadyExists => write!(f, "Already liked"),
            LikeError::Internal(msg) => write!(f, "Like internal error: {msg}"),
        }
    }
}

impl std::error::Error for LikeError {}
