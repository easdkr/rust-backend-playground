#[derive(Debug, Clone)]
pub enum NotificationError {
    NotFound(i32),
    Domain(String),
    Internal(String),
}

impl std::fmt::Display for NotificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotificationError::NotFound(id) => write!(f, "Notification with id {id} not found"),
            NotificationError::Domain(msg) => write!(f, "{msg}"),
            NotificationError::Internal(msg) => write!(f, "Internal error: {msg}"),
        }
    }
}

impl std::error::Error for NotificationError {}

impl NotificationError {
    pub fn repo<E: std::fmt::Display>(err: E) -> Self {
        NotificationError::Internal(err.to_string())
    }
}
