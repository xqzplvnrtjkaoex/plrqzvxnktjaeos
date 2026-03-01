use madome_users_migration::Migrator;

#[tokio::main]
async fn main() {
    sea_orm_migration::cli::run_cli(Migrator).await;
}
