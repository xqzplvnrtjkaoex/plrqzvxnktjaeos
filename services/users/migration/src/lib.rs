use sea_orm_migration::prelude::*;

mod m20250301_000001_create_users;
mod m20250301_000002_create_taste_books;
mod m20250301_000003_create_taste_book_tags;
mod m20250301_000004_create_history_books;
mod m20250301_000005_create_notification_books;
mod m20250301_000006_create_notification_book_tags;
mod m20250301_000007_create_fcm_tokens;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250301_000001_create_users::Migration),
            Box::new(m20250301_000002_create_taste_books::Migration),
            Box::new(m20250301_000003_create_taste_book_tags::Migration),
            Box::new(m20250301_000004_create_history_books::Migration),
            Box::new(m20250301_000005_create_notification_books::Migration),
            Box::new(m20250301_000006_create_notification_book_tags::Migration),
            Box::new(m20250301_000007_create_fcm_tokens::Migration),
        ]
    }
}
