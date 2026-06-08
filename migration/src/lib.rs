pub use sea_orm_migration::*;

mod m20240101_000001_create_users;
mod m20240101_000002_create_posts;
mod m20240101_000003_add_user_id_to_posts;
mod m20240101_000004_create_comments;
mod m20240101_000005_enhance_posts;
mod m20240101_000006_create_tags;
mod m20240101_000007_create_post_revisions;
mod m20240101_000008_create_comments;
mod m20240101_000009_create_notifications;
#[path = "migrator.rs"]
mod migration_registry;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        migration_registry::migrations()
    }
}
