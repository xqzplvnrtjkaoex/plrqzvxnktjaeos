use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .table(FcmTokens::Table)
                    .col(FcmTokens::UserId)
                    .name("idx_fcm_tokens_user_id")
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .table(NotificationBookTags::Table)
                    .col(NotificationBookTags::NotificationBookId)
                    .name("idx_notification_book_tags_notification_book_id")
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_notification_book_tags_notification_book_id")
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(Index::drop().name("idx_fcm_tokens_user_id").to_owned())
            .await
    }
}

#[derive(Iden)]
enum FcmTokens {
    Table,
    UserId,
}

#[derive(Iden)]
enum NotificationBookTags {
    Table,
    NotificationBookId,
}
