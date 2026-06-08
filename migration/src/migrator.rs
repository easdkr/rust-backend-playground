use sea_orm_migration::prelude::*;

use super::{
    m20240101_000001_create_users, m20240101_000002_create_posts,
    m20240101_000003_add_user_id_to_posts, m20240101_000004_create_comments,
    m20240101_000005_enhance_posts, m20240101_000006_create_tags,
    m20240101_000007_create_post_revisions, m20240101_000008_create_comments,
    m20240101_000009_create_notifications,
};

pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
        Box::new(m20240101_000001_create_users::Migration),
        Box::new(m20240101_000002_create_posts::Migration),
        Box::new(m20240101_000003_add_user_id_to_posts::Migration),
        Box::new(m20240101_000004_create_comments::Migration),
        Box::new(m20240101_000005_enhance_posts::Migration),
        Box::new(m20240101_000006_create_tags::Migration),
        Box::new(m20240101_000007_create_post_revisions::Migration),
        Box::new(m20240101_000008_create_comments::Migration),
        Box::new(m20240101_000009_create_notifications::Migration),
    ]
}
