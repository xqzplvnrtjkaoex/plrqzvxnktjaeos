use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AuthCodes::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AuthCodes::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AuthCodes::UserId).uuid().not_null())
                    .col(ColumnDef::new(AuthCodes::Code).string().not_null())
                    .col(
                        ColumnDef::new(AuthCodes::ExpiresAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(AuthCodes::UsedAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(AuthCodes::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(AuthCodes::Table, AuthCodes::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(AuthCodes::Table)
                    .col(AuthCodes::UserId)
                    .name("idx_auth_codes_user_id")
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AuthCodes::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum AuthCodes {
    Table,
    Id,
    UserId,
    Code,
    ExpiresAt,
    UsedAt,
    CreatedAt,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}
