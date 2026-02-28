use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(OutboxEvents::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(OutboxEvents::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(OutboxEvents::Kind).string().not_null())
                    .col(
                        ColumnDef::new(OutboxEvents::Payload)
                            .json_binary()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OutboxEvents::IdempotencyKey)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(OutboxEvents::Attempts)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(OutboxEvents::LastError).string())
                    .col(
                        ColumnDef::new(OutboxEvents::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OutboxEvents::NextAttemptAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(OutboxEvents::ProcessedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(OutboxEvents::FailedAt).timestamp_with_time_zone())
                    .to_owned(),
            )
            .await?;

        // Index for worker poll queries (unprocessed, unfailed, by next_attempt_at).
        manager
            .create_index(
                Index::create()
                    .table(OutboxEvents::Table)
                    .col(OutboxEvents::NextAttemptAt)
                    .name("idx_outbox_events_next_attempt_at")
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(OutboxEvents::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum OutboxEvents {
    Table,
    Id,
    Kind,
    Payload,
    IdempotencyKey,
    Attempts,
    LastError,
    CreatedAt,
    NextAttemptAt,
    ProcessedAt,
    FailedAt,
}
