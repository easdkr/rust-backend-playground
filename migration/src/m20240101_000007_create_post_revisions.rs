use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            r#"
            CREATE TABLE IF NOT EXISTS post_revisions (
                id         SERIAL PRIMARY KEY,
                post_id    INTEGER NOT NULL REFERENCES posts (id) ON DELETE CASCADE,
                editor_id  VARCHAR(255) NOT NULL REFERENCES users (id) ON DELETE CASCADE,
                title      VARCHAR(255) NOT NULL,
                content    TEXT NOT NULL,
                excerpt    TEXT,
                status     VARCHAR(20) NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            CREATE INDEX IF NOT EXISTS post_revisions_post_id_idx
                ON post_revisions (post_id, created_at DESC);
            "#,
        )
        .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("DROP TABLE IF EXISTS post_revisions;")
            .await?;
        Ok(())
    }
}
