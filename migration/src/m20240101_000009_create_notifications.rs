use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Notification::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Notification::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Notification::UserId).string().not_null())
                    .col(ColumnDef::new(Notification::Type).string().not_null())
                    .col(ColumnDef::new(Notification::Title).string().not_null())
                    .col(
                        ColumnDef::new(Notification::Body)
                            .text()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Notification::Data).json())
                    .col(
                        ColumnDef::new(Notification::IsRead)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Notification::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Notification::ReadAt)
                            .timestamp_with_time_zone(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_notifications_user_id_created_at")
                    .table(Notification::Table)
                    .col(Notification::UserId)
                    .col(Notification::CreatedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_notifications_user_id_is_read")
                    .table(Notification::Table)
                    .col(Notification::UserId)
                    .col(Notification::IsRead)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Notification::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Notification {
    Table,
    Id,
    UserId,
    Type,
    Title,
    Body,
    Data,
    IsRead,
    CreatedAt,
    ReadAt,
}
