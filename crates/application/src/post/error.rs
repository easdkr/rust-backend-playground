#[derive(Debug)]
pub enum PostError {
    NotFound(i32),
    Domain(String),
    Internal(String),
}

impl PostError {
    pub fn repo(message: String) -> Self {
        Self::Internal(message)
    }
}
