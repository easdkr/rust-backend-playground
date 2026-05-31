pub use sea_orm_migration::*;

mod m20240101_000001_create_users;
mod m20240101_000002_create_posts;
mod m20240101_000003_add_user_id_to_posts;
#[path = "migrator.rs"]
mod migration_registry;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        migration_registry::migrations()
    }
}
