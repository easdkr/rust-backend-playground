use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct CursorPage<T: ToSchema> {
    pub data: Vec<T>,
    pub next_cursor: Option<i32>,
    pub has_more: bool,
}

impl<T: ToSchema> CursorPage<T> {
    pub fn new(data: Vec<T>, next_cursor: Option<i32>, has_more: bool) -> Self {
        Self {
            data,
            next_cursor,
            has_more,
        }
    }
}
