//! Auth service contract runner (requires `--features auth`).

use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use deadpool_redis::Runtime;
use madome_auth::{router::build_router, state::AppState};
use madome_auth_migration::Migrator;
use sea_orm::Database;
use sea_orm_migration::MigratorTrait;
use tokio::net::TcpListener;
use url::Url;
use webauthn_rs::prelude::WebauthnBuilder;

use crate::{
    config::ContractHarnessConfig, fixture, reporter, runner::Runner, services::InfraUrls,
};

/// Run auth migrations, start the auth service in-process, run all auth fixtures.
///
/// Returns `true` if every fixture passed.
pub async fn run(
    infra: &InfraUrls,
    config: &ContractHarnessConfig,
    workspace_root: &Path,
) -> Result<bool> {
    // ── DB + migrations ────────────────────────────────────────────────────
    let db = Database::connect(&infra.database_url).await?;
    Migrator::up(&db, None).await?;

    // ── Redis pool ─────────────────────────────────────────────────────────
    let redis =
        deadpool_redis::Config::from_url(&infra.redis_url).create_pool(Some(Runtime::Tokio1))?;

    // ── WebAuthn ───────────────────────────────────────────────────────────
    let origin = Url::parse(&config.webauthn_origin)?;
    let webauthn = Arc::new(
        WebauthnBuilder::new(&config.webauthn_rp_id, &origin)?
            .rp_name("Madome")
            .build()?,
    );

    // ── Start auth service on a random OS-assigned port ────────────────────
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    let base_url = format!("http://127.0.0.1:{port}");

    let state = AppState {
        db,
        redis,
        webauthn,
        jwt_secret: config.jwt_secret.clone(),
        cookie_domain: config.cookie_domain.clone(),
    };
    tokio::spawn(async move {
        axum::serve(listener, build_router(state)).await.unwrap();
    });

    // ── Load fixtures and run ──────────────────────────────────────────────
    let fixtures = fixture::load_all(workspace_root, Some("auth"))?;
    let runner = Runner::new(&base_url);
    let mut rep = reporter::Reporter::new();

    for f in &fixtures {
        let result = runner.run(f).await;
        rep.record(f, result);
    }

    rep.print_summary();
    Ok(rep.all_passed())
}
