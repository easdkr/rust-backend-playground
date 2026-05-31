use crate::error::DomainError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthError {
    InvalidCredentials,
    UserNotFound,
    TokenExpired,
    TokenInvalid(String),
    TokenBlacklisted,
    InsufficientPermissions,
    Internal(String),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::InvalidCredentials => write!(f, "Invalid username or password"),
            AuthError::UserNotFound => write!(f, "User not found"),
            AuthError::TokenExpired => write!(f, "Token has expired"),
            AuthError::TokenInvalid(msg) => write!(f, "Invalid token: {msg}"),
            AuthError::TokenBlacklisted => write!(f, "Token has been revoked"),
            AuthError::InsufficientPermissions => write!(f, "Insufficient permissions"),
            AuthError::Internal(msg) => write!(f, "Internal error: {msg}"),
        }
    }
}

impl DomainError for AuthError {
    fn internal(msg: impl Into<String>) -> Self {
        AuthError::Internal(msg.into())
    }

    fn is_not_found(&self) -> bool {
        matches!(self, AuthError::UserNotFound)
    }
}

impl AuthError {
    pub fn internal(msg: impl Into<String>) -> Self {
        AuthError::Internal(msg.into())
    }
}
