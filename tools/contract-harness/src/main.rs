//! Contract harness — runs HTTP golden assertions against live services.
//!
//! # Usage
//!
//! ```bash
//! # Run all fixtures against a gateway
//! cargo run -p contract-harness -- --base-url http://localhost:3000 --env dev
//!
//! # Run only auth service fixtures
//! cargo run -p contract-harness -- --base-url http://localhost:3112 --service auth
//! ```
//!
//! Exits 0 when all assertions pass, exits 1 when any fail.

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

mod fixture;
mod reporter;
mod runner;

use fixture::Fixture;
use reporter::Reporter;
use runner::Runner;

#[derive(Parser)]
#[command(about = "Run HTTP contract assertions against live services")]
struct Args {
    /// Base URL of the service or gateway (e.g. http://localhost:3112)
    #[arg(long)]
    base_url: String,

    /// Run only fixtures for this service: auth, library, or users
    #[arg(long)]
    service: Option<String>,

    /// Environment name used to select the cookie contract file (dev or prod)
    #[arg(long, default_value = "dev")]
    env: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let workspace_root = workspace_root();
    let fixtures: Vec<Fixture> = fixture::load_all(&workspace_root, args.service.as_deref())?;

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

    let runner = Runner::new(&args.base_url);
    let mut reporter = Reporter::new();

    for f in &fixtures {
        let result = runner.run(f).await;
        reporter.record(f, result);
    }

    reporter.print_summary();

    if reporter.all_passed() {
        Ok(())
    } else {
        std::process::exit(1);
    }
}

/// Walk up from the binary's own manifest dir to find the workspace root
/// (the directory containing `Cargo.lock`).
fn workspace_root() -> PathBuf {
    let start = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    start
        .ancestors()
        .find(|p| p.join("Cargo.lock").exists())
        .unwrap_or(&start)
        .to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::workspace_root;

    #[test]
    fn workspace_root_has_cargo_lock() {
        let root = workspace_root();
        assert!(
            root.join("Cargo.lock").exists(),
            "workspace root should contain Cargo.lock"
        );
    }

    #[test]
    fn workspace_root_has_contracts_dir() {
        let root = workspace_root();
        assert!(
            root.join("contracts").exists(),
            "workspace root should contain contracts/"
        );
    }
}
