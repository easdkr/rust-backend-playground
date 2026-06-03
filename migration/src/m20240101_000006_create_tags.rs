use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(
            r#"
            CREATE TABLE IF NOT EXISTS tags (
                id    SERIAL PRIMARY KEY,
                name  VARCHAR(50)  NOT NULL,
                slug  VARCHAR(80)  NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            CREATE UNIQUE INDEX IF NOT EXISTS tags_slug_unique_idx ON tags (slug);
            CREATE UNIQUE INDEX IF NOT EXISTS tags_name_unique_idx ON tags (name);

            CREATE TABLE IF NOT EXISTS post_tags (
                post_id INTEGER NOT NULL REFERENCES posts (id) ON DELETE CASCADE,
                tag_id  INTEGER NOT NULL REFERENCES tags  (id) ON DELETE CASCADE,
                PRIMARY KEY (post_id, tag_id)
            );
            CREATE INDEX IF NOT EXISTS post_tags_tag_id_idx ON post_tags (tag_id);
            CREATE INDEX IF NOT EXISTS post_tags_post_id_idx ON post_tags (post_id);
            "#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            r#"
            DROP TABLE IF EXISTS post_tags;
            DROP TABLE IF EXISTS tags;
            "#,
        )
        .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Tags {
    Table,
}
