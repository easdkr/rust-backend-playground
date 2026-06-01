use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "comments")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub post_id: i32,
    pub parent_comment_id: Option<i32>,
    pub user_id: String,
    #[sea_orm(column_type = "Text")]
    pub content: String,
    pub depth: i32,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

pub type Comment = Model;

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
