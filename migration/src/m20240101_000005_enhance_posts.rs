use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // 1) 컬럼 추가 (NOT NULL 컬럼은 안전한 default 부여)
        db.execute_unprepared(
            r#"
            ALTER TABLE posts
                ADD COLUMN IF NOT EXISTS slug         VARCHAR(140),
                ADD COLUMN IF NOT EXISTS excerpt      TEXT,
                ADD COLUMN IF NOT EXISTS published_at TIMESTAMPTZ,
                ADD COLUMN IF NOT EXISTS updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                ADD COLUMN IF NOT EXISTS deleted_at   TIMESTAMPTZ,
                ADD COLUMN IF NOT EXISTS view_count   INTEGER     NOT NULL DEFAULT 0,
                ADD COLUMN IF NOT EXISTS search_vector tsvector
                    GENERATED ALWAYS AS (
                        setweight(to_tsvector('simple', coalesce(title,   '')), 'A') ||
                        setweight(to_tsvector('simple', coalesce(excerpt, '')), 'B') ||
                        setweight(to_tsvector('simple', coalesce(content, '')), 'C')
                    ) STORED;
            "#,
        )
        .await?;

        // 2) 기존 행 slug 백필 — 안전하게 `post-{id}` 사용 (앱이 다음 업데이트 시 자연스러운 slug로 교체)
        db.execute_unprepared(
            r#"
            UPDATE posts
               SET slug = 'post-' || id::text
             WHERE slug IS NULL;
            "#,
        )
        .await?;

        // 3) 인덱스
        db.execute_unprepared(
            r#"
            CREATE UNIQUE INDEX IF NOT EXISTS posts_slug_unique_idx
                ON posts (slug) WHERE slug IS NOT NULL;
            CREATE INDEX IF NOT EXISTS posts_user_id_idx
                ON posts (user_id);
            CREATE INDEX IF NOT EXISTS posts_status_active_idx
                ON posts (status) WHERE deleted_at IS NULL;
            CREATE INDEX IF NOT EXISTS posts_deleted_at_idx
                ON posts (deleted_at) WHERE deleted_at IS NOT NULL;
            CREATE INDEX IF NOT EXISTS posts_published_at_idx
                ON posts (published_at) WHERE published_at IS NOT NULL;
            CREATE INDEX IF NOT EXISTS posts_search_vector_idx
                ON posts USING GIN (search_vector);
            "#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            r#"
            DROP INDEX IF EXISTS posts_search_vector_idx;
            DROP INDEX IF EXISTS posts_published_at_idx;
            DROP INDEX IF EXISTS posts_deleted_at_idx;
            DROP INDEX IF EXISTS posts_status_active_idx;
            DROP INDEX IF EXISTS posts_user_id_idx;
            DROP INDEX IF EXISTS posts_slug_unique_idx;
            ALTER TABLE posts
                DROP COLUMN IF EXISTS search_vector,
                DROP COLUMN IF EXISTS view_count,
                DROP COLUMN IF EXISTS deleted_at,
                DROP COLUMN IF EXISTS updated_at,
                DROP COLUMN IF EXISTS published_at,
                DROP COLUMN IF EXISTS excerpt,
                DROP COLUMN IF EXISTS slug;
            "#,
        )
        .await?;
        Ok(())
    }
}
