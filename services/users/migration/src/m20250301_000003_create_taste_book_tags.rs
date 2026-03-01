use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TasteBookTags::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(TasteBookTags::UserId).uuid().not_null())
                    .col(ColumnDef::new(TasteBookTags::TagKind).string().not_null())
                    .col(ColumnDef::new(TasteBookTags::TagName).string().not_null())
                    .col(
                        ColumnDef::new(TasteBookTags::IsDislike)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(TasteBookTags::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .primary_key(
                        Index::create()
                            .col(TasteBookTags::UserId)
                            .col(TasteBookTags::TagKind)
                            .col(TasteBookTags::TagName),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(TasteBookTags::Table, TasteBookTags::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TasteBookTags::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum TasteBookTags {
    Table,
    UserId,
    TagKind,
    TagName,
    IsDislike,
    CreatedAt,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}
