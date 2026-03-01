use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TasteBooks::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(TasteBooks::UserId).uuid().not_null())
                    .col(ColumnDef::new(TasteBooks::BookId).integer().not_null())
                    .col(
                        ColumnDef::new(TasteBooks::IsDislike)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(TasteBooks::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .primary_key(
                        Index::create()
                            .col(TasteBooks::UserId)
                            .col(TasteBooks::BookId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(TasteBooks::Table, TasteBooks::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TasteBooks::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum TasteBooks {
    Table,
    UserId,
    BookId,
    IsDislike,
    CreatedAt,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}
