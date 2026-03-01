use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Passkeys::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Passkeys::CredentialId)
                            .binary()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Passkeys::UserId).uuid().not_null())
                    .col(ColumnDef::new(Passkeys::Aaguid).uuid().not_null())
                    .col(ColumnDef::new(Passkeys::Credential).binary().not_null())
                    .col(
                        ColumnDef::new(Passkeys::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Passkeys::Table)
                    .col(Passkeys::UserId)
                    .name("idx_passkeys_user_id")
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Passkeys::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Passkeys {
    Table,
    CredentialId,
    UserId,
    Aaguid,
    Credential,
    CreatedAt,
}
