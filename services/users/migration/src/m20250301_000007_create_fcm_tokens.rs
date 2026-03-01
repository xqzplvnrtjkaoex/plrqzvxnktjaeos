use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(FcmTokens::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(FcmTokens::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(FcmTokens::UserId).uuid().not_null())
                    .col(ColumnDef::new(FcmTokens::Token).string().not_null())
                    .col(
                        ColumnDef::new(FcmTokens::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(FcmTokens::Table, FcmTokens::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(FcmTokens::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum FcmTokens {
    Table,
    Id,
    UserId,
    Token,
    UpdatedAt,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}
