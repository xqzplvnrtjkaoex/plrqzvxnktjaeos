use sea_orm_migration::prelude::*;

mod m20260301_000001_create_users;
mod m20260301_000002_create_auth_codes;
mod m20260301_000003_create_passkeys;
mod m20260301_000004_create_outbox_events;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260301_000001_create_users::Migration),
            Box::new(m20260301_000002_create_auth_codes::Migration),
            Box::new(m20260301_000003_create_passkeys::Migration),
            Box::new(m20260301_000004_create_outbox_events::Migration),
        ]
    }
}

#[tokio::main]
async fn main() {
    cli::run_cli(Migrator).await;
}
