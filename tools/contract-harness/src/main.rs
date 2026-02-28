//! Contract harness entry point.
//!
//! ## URL mode (default — no service features compiled in)
//!
//! Runs contract fixtures against an already-running service:
//!
//! ```bash
//! cargo run -p contract-harness -- \
//!   --base-url http://localhost:3112 \
//!   --service auth
//! ```
//!
//! ## Docker mode (service feature flags)
//!
//! Spins up PostgreSQL + Redis containers, runs the service in-process,
//! then always tears the containers down:
//!
//! ```bash
//! cargo run -p contract-harness --features auth
//! DOCKER_HOST=tcp://192.168.1.100:2376 cargo run -p contract-harness --features auth
//! ```
//!
//! Exits 0 when all assertions pass, exits 1 when any fail.

use anyhow::Result;

// ── Docker mode ────────────────────────────────────────────────────────────

#[cfg(feature = "auth")]
mod docker_mode {
    use anyhow::{Result, anyhow};
    use contract_harness::{config::ContractHarnessConfig, docker::DockerOrchestrator, services};

    pub async fn run() -> Result<()> {
        dotenv::dotenv().ok();
        tracing_subscriber::fmt::init();

        let config = ContractHarnessConfig::from_env();

        // Exclusive file lock — only one harness instance at a time.
        // OS auto-releases on process exit, even on crash/panic.
        let lock_path = std::env::temp_dir().join("madome-contract-harness.lock");
        let lock_file = std::fs::File::create(&lock_path)?;
        let mut lock = fd_lock::RwLock::new(lock_file);
        let _guard = lock
            .try_write()
            .map_err(|_| anyhow!("another instance is running"))?;

        let mut orch = DockerOrchestrator::connect(&config.docker_host).await?;

        // Crash recovery: remove non-running test containers from a previous run.
        orch.cleanup_stale().await?;

        let database_url = orch.start_postgres().await?;
        let redis_url = orch.start_redis().await?;

        let infra = services::InfraUrls {
            database_url,
            redis_url,
        };
        let workspace_root = contract_harness::fixture::workspace_root();

        let result = run_services(&infra, &config, &workspace_root).await;

        // Always tear down containers regardless of test outcome.
        orch.cleanup().await.ok();

        let all_passed = result?;
        std::process::exit(if all_passed { 0 } else { 1 });
    }

    async fn run_services(
        infra: &services::InfraUrls,
        config: &ContractHarnessConfig,
        workspace_root: &std::path::Path,
    ) -> Result<bool> {
        let mut all_passed = true;

        #[cfg(feature = "auth")]
        {
            all_passed &= services::auth::run(infra, config, workspace_root).await?;
        }

        Ok(all_passed)
    }
}

// ── URL mode ───────────────────────────────────────────────────────────────

#[cfg(not(feature = "auth"))]
mod url_mode {
    use anyhow::Result;
    use clap::Parser;
    use contract_harness::{fixture, reporter, runner};

    #[derive(Parser)]
    #[command(about = "Run HTTP contract assertions against live services")]
    pub struct Args {
        /// Base URL of the service or gateway (e.g. http://localhost:3112)
        #[arg(long)]
        pub base_url: String,

        /// Run only fixtures for this service: auth, library, or users
        #[arg(long)]
        pub service: Option<String>,

        /// Environment name used to select the cookie contract file (dev or prod)
        #[arg(long, default_value = "dev")]
        pub env: String,
    }

    pub async fn run() -> Result<()> {
        let args = Args::parse();

        let workspace_root = fixture::workspace_root();
        let fixtures = fixture::load_all(&workspace_root, args.service.as_deref())?;

        if fixtures.is_empty() {
            eprintln!("No fixtures found.");
            return Ok(());
        }

        println!(
            "Running {} fixture(s) against {}",
            fixtures.len(),
            args.base_url
        );
        println!();

        let runner = runner::Runner::new(&args.base_url);
        let mut rep = reporter::Reporter::new();

        for f in &fixtures {
            let result = runner.run(f).await;
            rep.record(f, result);
        }

        rep.print_summary();

        if rep.all_passed() {
            Ok(())
        } else {
            std::process::exit(1);
        }
    }
}

// ── Entry point ────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(feature = "auth")]
    {
        docker_mode::run().await
    }

    #[cfg(not(feature = "auth"))]
    {
        url_mode::run().await
    }
}
