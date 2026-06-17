use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Likes::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Likes::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Likes::PostId).integer().not_null())
                    .col(ColumnDef::new(Likes::UserId).string().not_null())
                    .col(
                        ColumnDef::new(Likes::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_likes_post_id_user_id")
                    .table(Likes::Table)
                    .col(Likes::PostId)
                    .col(Likes::UserId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_likes_post_id")
                    .table(Likes::Table)
                    .col(Likes::PostId)
                    .to_owned(),
            )
            .await?;

        // posts 테이블에 like_count 컬럼 추가
        let db = manager.get_connection();
        db.execute_unprepared(
            r#"
            ALTER TABLE posts
            ADD COLUMN IF NOT EXISTS like_count INTEGER NOT NULL DEFAULT 0;
            "#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            r#"
            ALTER TABLE posts
            DROP COLUMN IF EXISTS like_count;
            "#,
        )
        .await?;

        manager
            .drop_table(Table::drop().table(Likes::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Likes {
    Table,
    Id,
    PostId,
    UserId,
    CreatedAt,
}
