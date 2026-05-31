pub trait IntoStringErr<T> {
    fn map_err_string(self) -> Result<T, String>;
}

impl<T, E: std::fmt::Display> IntoStringErr<T> for Result<T, E> {
    fn map_err_string(self) -> Result<T, String> {
        self.map_err(|e| e.to_string())
    }
}
