use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            r#"
            CREATE TABLE IF NOT EXISTS comments (
                id         SERIAL PRIMARY KEY,
                post_id    INTEGER NOT NULL REFERENCES posts (id) ON DELETE CASCADE,
                author_id  VARCHAR(255) NOT NULL REFERENCES users (id) ON DELETE CASCADE,
                content    TEXT NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                deleted_at TIMESTAMPTZ
            );
            CREATE INDEX IF NOT EXISTS comments_post_id_idx ON comments (post_id) WHERE deleted_at IS NULL;
            CREATE INDEX IF NOT EXISTS comments_author_id_idx ON comments (author_id);
            "#,
        )
        .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("DROP TABLE IF EXISTS comments;")
            .await?;
        Ok(())
    }
}
