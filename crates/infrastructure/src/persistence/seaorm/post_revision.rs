use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "post_revisions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub post_id: i32,
    pub editor_id: String,
    pub title: String,
    #[sea_orm(column_type = "Text")]
    pub content: String,
    pub excerpt: Option<String>,
    pub status: String,
    pub version: i32,
    pub created_at: DateTimeUtc,
}

pub type PostRevision = Model;

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
