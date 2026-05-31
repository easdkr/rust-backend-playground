use sea_orm_migration::prelude::*;

use super::{
    m20240101_000001_create_users, m20240101_000002_create_posts,
    m20240101_000003_add_user_id_to_posts,
};

pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
        Box::new(m20240101_000001_create_users::Migration),
        Box::new(m20240101_000002_create_posts::Migration),
        Box::new(m20240101_000003_add_user_id_to_posts::Migration),
    ]
}
