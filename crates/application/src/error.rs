pub trait DomainError: std::fmt::Display {
    fn internal(msg: impl Into<String>) -> Self;
    fn is_not_found(&self) -> bool;
}
