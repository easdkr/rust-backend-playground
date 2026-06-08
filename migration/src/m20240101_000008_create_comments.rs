use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        // comments 테이블이 이미 존재할 수 있으므로 (m20240101_000004에서 생성)
        // deleted_at 컬럼과 인덱스만 추가
        db.execute_unprepared(
            r#"
            ALTER TABLE comments
                ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;
            CREATE INDEX IF NOT EXISTS comments_post_id_idx ON comments (post_id) WHERE deleted_at IS NULL;
            CREATE INDEX IF NOT EXISTS comments_user_id_idx ON comments (user_id);
            "#,
        )
        .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            r#"
            DROP INDEX IF EXISTS comments_post_id_idx;
            DROP INDEX IF EXISTS comments_author_id_idx;
            ALTER TABLE comments DROP COLUMN IF EXISTS deleted_at;
            "#,
        )
        .await?;
        Ok(())
    }
}
