use serde::Serialize;
use utoipa::ToSchema;

/// limit 값이 유효한지 검증 (API별 min/max 지정)
pub fn validate_limit(limit: usize, min: usize, max: usize) -> Result<(), String> {
    if limit < min || limit > max {
        return Err(format!("limit must be between {} and {}", min, max));
    }
    Ok(())
}

/// limit 값을 유효한 범위로 클램핑 (API별 min/max 지정)
pub fn clamp_limit(limit: usize, min: usize, max: usize) -> usize {
    limit.clamp(min, max)
}

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
