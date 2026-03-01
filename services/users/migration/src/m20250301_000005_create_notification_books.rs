use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(NotificationBooks::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(NotificationBooks::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(NotificationBooks::UserId).uuid().not_null())
                    .col(
                        ColumnDef::new(NotificationBooks::BookId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(NotificationBooks::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(NotificationBooks::Table, NotificationBooks::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(NotificationBooks::Table)
                    .col(NotificationBooks::UserId)
                    .col((NotificationBooks::CreatedAt, IndexOrder::Desc))
                    .name("idx_notification_books_user_id_created_at")
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(NotificationBooks::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum NotificationBooks {
    Table,
    Id,
    UserId,
    BookId,
    CreatedAt,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}
