//! Users service contract runner (requires `--features users`).

use std::path::Path;

use anyhow::Result;
use madome_users::{router::build_router, state::AppState};
use madome_users_migration::Migrator;
use sea_orm::Database;
use sea_orm_migration::MigratorTrait;
use tokio::net::TcpListener;

use crate::{fixture, reporter, runner::Runner, services::InfraUrls};

/// Run users migrations, start the users service in-process, run all users fixtures.
///
/// Returns `true` if every fixture passed.
pub async fn run(infra: &InfraUrls, workspace_root: &Path) -> Result<bool> {
    // ── DB + migrations ────────────────────────────────────────────────────
    let db = Database::connect(&infra.database_url).await?;
    Migrator::up(&db, None).await?;

    // ── Start users service on a random OS-assigned port ────────────────────
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    let base_url = format!("http://127.0.0.1:{port}");

    let state = AppState {
        db,
        library_client: madome_users::infra::grpc::GrpcLibraryClient::lazy(
            "http://127.0.0.1:0",
        ),
    };
    tokio::spawn(async move {
        axum::serve(listener, build_router(state)).await.unwrap();
    });

    // ── Load fixtures and run ──────────────────────────────────────────────
    let fixtures = fixture::load_all(workspace_root, Some("users"))?;
    let runner = Runner::new(&base_url);
    let mut rep = reporter::Reporter::new();

    for f in &fixtures {
        let result = runner.run(f).await;
        rep.record(f, result);
    }

    rep.print_summary();
    Ok(rep.all_passed())
}
