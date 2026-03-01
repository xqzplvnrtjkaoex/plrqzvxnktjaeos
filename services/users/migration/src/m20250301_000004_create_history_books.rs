use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(HistoryBooks::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(HistoryBooks::UserId).uuid().not_null())
                    .col(ColumnDef::new(HistoryBooks::BookId).integer().not_null())
                    .col(
                        ColumnDef::new(HistoryBooks::Page)
                            .integer()
                            .not_null()
                            .default(1),
                    )
                    .col(
                        ColumnDef::new(HistoryBooks::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(HistoryBooks::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .primary_key(
                        Index::create()
                            .col(HistoryBooks::UserId)
                            .col(HistoryBooks::BookId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(HistoryBooks::Table, HistoryBooks::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(HistoryBooks::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum HistoryBooks {
    Table,
    UserId,
    BookId,
    Page,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}
