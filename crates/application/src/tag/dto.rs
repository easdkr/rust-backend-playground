use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct TagDto {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateTagCmd {
    #[validate(length(min = 1, max = 50))]
    pub name: String,
    /// 비워두면 name에서 slugify하여 자동 생성
    pub slug: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct AttachTagsCmd {
    #[validate(length(min = 1, max = 50))]
    pub tag_ids: Vec<i32>,
}
