use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(NotificationBookTags::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(NotificationBookTags::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(NotificationBookTags::NotificationBookId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(NotificationBookTags::TagKind)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(NotificationBookTags::TagName)
                            .string()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                NotificationBookTags::Table,
                                NotificationBookTags::NotificationBookId,
                            )
                            .to(NotificationBooks::Table, NotificationBooks::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(NotificationBookTags::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum NotificationBookTags {
    Table,
    Id,
    NotificationBookId,
    TagKind,
    TagName,
}

#[derive(Iden)]
enum NotificationBooks {
    Table,
    Id,
}
