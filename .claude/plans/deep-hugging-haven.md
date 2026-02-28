# Plan: Unified Contract Harness with Remote Docker Orchestration

## Context

`tools/contract-harness` has two binaries: the URL-based `contract-harness` (live deployments)
and `auth-harness` (in-process, requires local infra pre-running). The new goal: a single binary
that spins up its own PostgreSQL + Redis containers via a Docker daemon (local or remote), runs
selected service tests against an in-process service instance, then always tears everything down.
No pre-running infrastructure required.

Single command:
```bash
# Local Docker socket:
cargo run -p contract-harness --features auth

# Remote Docker:
DOCKER_HOST=tcp://192.168.1.100:2376 cargo run -p contract-harness --features auth
```

---

## Key Design Decisions

### Docker crate
`bollard` — standard async Rust Docker API client; supports Unix socket + TCP; reads
`DOCKER_HOST` automatically via `Docker::connect_with_defaults()`.

### Exclusive lock
Use an OS file lock (`flock`) at `$TMPDIR/madome-contract-harness.lock`:
- Try non-blocking exclusive write lock; if held → exit: `"another instance is running"`
- OS automatically releases on process exit, even on crash/panic (no stale lock)
- Independent of Docker — lock acquired before any Docker API call
- Crate: `fd-lock = "4"` (thin safe wrapper around `flock`/`LockFileEx`)

### Container lifecycle
1. Acquire file lock (fail fast if held)
2. Startup: cleanup any containers labeled `madome.role=contract-test` (crash recovery)
3. Create `postgres:18` + `redis:8` with label `madome.role=contract-test`
4. Wait for ports to accept TCP connections (probe loop, 30 s timeout)
5. Run each enabled service feature
6. **Always** stop + remove test containers (success or failure)
7. File lock auto-released when process exits

### Feature flags (compile-time)
```bash
cargo run -p contract-harness --features auth        # only auth
# future: --features auth,users,library
```
Each feature conditionally compiles service-specific code (migrations + in-process server).
Building with no features still compiles the URL-based `contract-harness` binary unchanged.

### DB / Redis URL construction
Postgres + Redis containers use `HostPort = ""` (random). After starting, inspect the
container to get the mapped port. Extract the Docker host from `DOCKER_HOST`:
- `unix://...` → `127.0.0.1`
- `tcp://HOST:PORT` → `HOST`

Result:
```
DATABASE_URL = postgres://postgres:postgres@{docker_host}:{mapped_pg_port}/madome_test
REDIS_URL    = redis://{docker_host}:{mapped_redis_port}
```

The service (e.g. auth) runs **in-process** on the test machine, connecting to remote containers.

### Cleanup pattern (no Drop)
```rust
let result = run_services(&infra, &config, &workspace_root).await;
orch.cleanup().await.ok();  // always, ignores secondary cleanup errors
result?;
```

---

## Changes

### 1. `tools/contract-harness/src/lib.rs` — MODIFY

Add new modules (existing `fixture`, `reporter`, `runner` unchanged):
```rust
pub mod config;
pub mod docker;
pub mod services;
```

### 2. `tools/contract-harness/src/docker.rs` — NEW

```rust
pub struct DockerOrchestrator {
    client: bollard::Docker,
    /// IP/hostname to reach containers from the host machine
    pub host: String,
    test_container_ids: Vec<String>,
}

impl DockerOrchestrator {
    /// Connect to the Docker daemon at the given URL.
    /// Parses scheme: "unix://" → local socket; "tcp://" → HTTP.
    /// Sets `self.host` to the IP/hostname used to reach containers.
    pub async fn connect(docker_host: &str) -> anyhow::Result<Self>

    /// Remove all NON-RUNNING containers labeled `madome.role=contract-test`.
    /// (Only removes exited/dead containers — running containers from other sessions untouched.)
    pub async fn cleanup_stale(&self) -> anyhow::Result<()>

    /// Start postgres:18 on a random port. Returns DATABASE_URL.
    pub async fn start_postgres(&mut self) -> anyhow::Result<String>

    /// Start redis:8 on a random port. Returns REDIS_URL.
    pub async fn start_redis(&mut self) -> anyhow::Result<String>

    /// Stop + remove all test containers. Always call this.
    pub async fn cleanup(&mut self) -> anyhow::Result<()>
}

/// TCP connect probe — waits until the port accepts connections (30 s max).
async fn wait_port_open(host: &str, port: u16, timeout_secs: u64) -> anyhow::Result<()>

/// Extract the addressable hostname from a Docker URL.
/// unix:// → "127.0.0.1";  tcp://HOST:PORT → "HOST"
fn docker_host_from_url(url: &str) -> String
```

### 3. `tools/contract-harness/src/services/mod.rs` — NEW

```rust
pub struct InfraUrls {
    pub database_url: String,
    pub redis_url: String,
}

#[cfg(feature = "auth")]
pub mod auth;
```

### 4. `tools/contract-harness/src/services/auth.rs` (feature = "auth") — NEW

```rust
/// Run auth migrations, start auth service in-process, run auth fixtures.
/// Returns `true` if all fixtures passed.
pub async fn run(
    infra: &InfraUrls,
    config: &ContractHarnessConfig,
    workspace_root: &Path,
) -> anyhow::Result<bool>
```

Internally:
- `Database::connect(&infra.database_url)` → `Migrator::up(&db, None)`
- `deadpool_redis::Config::from_url(&infra.redis_url).create_pool(...)`
- Build `AppState` + `TcpListener::bind("127.0.0.1:0")`
- `tokio::spawn(async move { axum::serve(...).await.unwrap(); })`
- `fixture::load_all(workspace_root, Some("auth"))` → run → report

### 5. `tools/contract-harness/src/config.rs` — NEW

```rust
/// All configuration for the Docker-based contract harness.
/// Loaded from env vars after dotenv::dotenv().ok(); no CLI parsing.
#[derive(Debug)]
pub struct ContractHarnessConfig {
    /// Docker daemon URL — DOCKER_HOST env var
    /// default: "unix:///var/run/docker.sock"
    pub docker_host: String,

    /// JWT_SECRET — default: "test-contract-secret"
    pub jwt_secret: String,

    /// WEBAUTHN_RP_ID — default: "localhost"
    pub webauthn_rp_id: String,

    /// WEBAUTHN_ORIGIN — default: "http://localhost"
    pub webauthn_origin: String,

    /// COOKIE_DOMAIN — default: "localhost"
    pub cookie_domain: String,
}

impl ContractHarnessConfig {
    pub fn from_env() -> Self {
        Self {
            docker_host: std::env::var("DOCKER_HOST")
                .unwrap_or_else(|_| "unix:///var/run/docker.sock".to_owned()),
            jwt_secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "test-contract-secret".to_owned()),
            webauthn_rp_id: std::env::var("WEBAUTHN_RP_ID")
                .unwrap_or_else(|_| "localhost".to_owned()),
            webauthn_origin: std::env::var("WEBAUTHN_ORIGIN")
                .unwrap_or_else(|_| "http://localhost".to_owned()),
            cookie_domain: std::env::var("COOKIE_DOMAIN")
                .unwrap_or_else(|_| "localhost".to_owned()),
        }
    }
}
```

Called once at startup after `dotenv::dotenv().ok()`. Passed by reference throughout.

### 6. `tools/contract-harness/src/main.rs` — REWRITE

```
Flow (Docker-feature mode, i.e. at least one service feature enabled):
  1. dotenv::dotenv().ok()
  2. let config = ContractHarnessConfig::from_env()
  3. acquire_file_lock()               ← flock on $TMPDIR/madome-contract-harness.lock
                                         fail fast if held; auto-released on exit/crash
  4. let mut orch = DockerOrchestrator::connect(&config.docker_host).await?
  5. orch.cleanup_stale().await        ← crash recovery (label-based)
  6. database_url = orch.start_postgres().await
  7. redis_url    = orch.start_redis().await
  8. let result = run_services(&infra, &config, &workspace_root).await
  9. orch.cleanup().await.ok()         ← always
  10. exit 0 / 1

run_services(infra, config, workspace_root):
  let mut all_passed = true;
  #[cfg(feature = "auth")]  { all_passed &= services::auth::run(infra, config, workspace_root).await?; }
  Ok(all_passed)
```

Note: `run_services` does NOT take `&mut orch` — orch is only used in main for cleanup.

Without service features compiled in, binary works as before (URL-based mode with existing
`--base-url` / `--service` / `--env` args). Gate Docker flow on
`cfg(any(feature = "auth", ...))`.

### 7. `tools/contract-harness/Cargo.toml` — MODIFY

```toml
[features]
auth = [
    "dep:madome-auth",
    "dep:madome-auth-migration",
    "dep:axum",
    "dep:sea-orm",
    "dep:sea-orm-migration",
    "dep:deadpool-redis",
    "dep:webauthn-rs",
    "dep:url",
]

[dependencies]
# always present (needed for Docker orchestration even without service features)
bollard            = { workspace = true }
fd-lock            = "4"
tokio              = { workspace = true }
reqwest            = { workspace = true }
serde              = { workspace = true }
serde_json         = { workspace = true }
clap               = { workspace = true }
anyhow             = { workspace = true }
tracing-subscriber = { workspace = true }
dotenv             = "0.15"

# auth feature only
madome-auth           = { path = "../../services/auth",           optional = true }
madome-auth-migration = { path = "../../services/auth/migration", optional = true }
axum                  = { workspace = true, optional = true }
sea-orm               = { workspace = true, optional = true }
sea-orm-migration     = { workspace = true, optional = true }
deadpool-redis        = { workspace = true, optional = true }
webauthn-rs           = { workspace = true, optional = true }
url                   = { version = "2",    optional = true }
```

Remove: `[[bin]] auth-harness` and `src/bin/auth_harness.rs`.

### 8. Workspace `Cargo.toml` — MODIFY

```toml
bollard = { version = "0.20" }
```

---

## Files touched

| Action | Path |
|--------|------|
| Remove | `tools/contract-harness/src/bin/auth_harness.rs` |
| Modify | `Cargo.toml` (workspace — add bollard) |
| Modify | `tools/contract-harness/Cargo.toml` |
| Modify | `tools/contract-harness/src/lib.rs` |
| Modify | `tools/contract-harness/src/main.rs` |
| Create | `tools/contract-harness/src/config.rs` |
| Create | `tools/contract-harness/src/docker.rs` |
| Create | `tools/contract-harness/src/services/mod.rs` |
| Create | `tools/contract-harness/src/services/auth.rs` |
| Create | `tools/contract-harness/README.md` |
| Modify | `.claude/docs/testing-philosophy.md` |

---

## Verification

```bash
# 1. Build without features (URL-mode binary must still compile)
cargo build -p contract-harness

# 2. Build with auth feature
cargo build -p contract-harness --features auth

# 3. Lint
cargo clippy --workspace --all-targets --all-features -- -D warnings

# 4. Unit tests
cargo test --workspace --all-features

# 5. End-to-end (needs Docker running)
cargo run -p contract-harness --features auth
# Expected: postgres+redis containers appear, fixtures run, containers removed, exit 0

# 6. Concurrent-run lock test
cargo run -p contract-harness --features auth &
sleep 2
cargo run -p contract-harness --features auth
# Expected: second run exits immediately "another instance is running"

# 7. Remote Docker (if available)
DOCKER_HOST=tcp://remote:2376 cargo run -p contract-harness --features auth
```

---

## Docs to update

### `.claude/docs/testing-philosophy.md` — extend self-contained harness section

Append a new subsection after the existing "Migration lib extraction" block:

```markdown
**Docker-orchestrated harnesses** (for services requiring a real database/cache)

When the service under test needs PostgreSQL + Redis, spin them up as Docker containers
instead of requiring manual infra setup. This keeps the single-command guarantee.

Pattern (using `bollard` + `fd-lock`):

```rust
// 1. Exclusive lock — only one harness instance at a time
//    OS auto-releases even on crash/panic (no stale lock)
let lock_path = std::env::temp_dir().join("madome-contract-harness.lock");
let lock_file = std::fs::File::create(&lock_path)?;
let mut lock  = fd_lock::RwLock::new(lock_file);
let _guard    = lock.try_write().map_err(|_| anyhow!("another instance is running"))?;

// 2. Connect to Docker daemon (reads DOCKER_HOST env var)
let mut orch = DockerOrchestrator::connect(&config.docker_host).await?;

// 3. Crash recovery — remove non-running test containers from previous run
orch.cleanup_stale().await?;

// 4. Start infra containers on random ports
let database_url = orch.start_postgres().await?;
let redis_url    = orch.start_redis().await?;

// 5. Run services in-process against the containers
let result = run_services(&infra, &config, &workspace_root).await;

// 6. Always tear down (success or failure — never use Drop for this)
orch.cleanup().await.ok();
result?;
```

Rules:
- Label every test container (`madome.role=contract-test`) for safe targeted cleanup.
- `cleanup_stale()` removes only **non-running** containers (exited/dead) — never kills
  containers from a concurrently running instance.
- Use **Cargo feature flags** to compile service-specific code into the harness binary:
  `--features auth` enables only the auth service runner; `--features auth,users` enables
  both. Building without features keeps the URL-mode binary unchanged.
- Service-specific deps (`madome-auth`, migrations, etc.) must be `optional = true` in
  `Cargo.toml` and gated behind their feature. Never add them to `[dependencies]` directly.
- `bollard` and `fd-lock` are always-present deps (needed for Docker orchestration even
  without service features).
- Containers use `HostPort = ""` (random); inspect after start to obtain the mapped port.
  `tcp://HOST:PORT` → use HOST as the container address; `unix://...` → use `127.0.0.1`.
```

### `tools/contract-harness/README.md` — create

Content outline:

```markdown
# contract-harness

Two modes:

## URL mode (default — no service features)

Runs contract fixtures against an **already-running** service.

```bash
cargo run -p contract-harness -- \
  --base-url http://localhost:3112 \
  --service auth \
  --env dev
```

## Docker mode (service feature flags)

Spins up PostgreSQL + Redis containers automatically, runs the service in-process,
then tears everything down. Requires Docker.

```bash
# Local Docker socket (default):
cargo run -p contract-harness --features auth

# Remote Docker daemon:
DOCKER_HOST=tcp://192.168.1.100:2376 cargo run -p contract-harness --features auth
```

## Environment variables (Docker mode)

| Variable         | Default                        | Description                          |
|------------------|--------------------------------|--------------------------------------|
| `DOCKER_HOST`    | `unix:///var/run/docker.sock`  | Docker daemon URL                    |
| `JWT_SECRET`     | `test-contract-secret`         | HMAC secret for token signing        |
| `WEBAUTHN_RP_ID` | `localhost`                    | WebAuthn relying-party ID            |
| `WEBAUTHN_ORIGIN`| `http://localhost`             | WebAuthn relying-party origin        |
| `COOKIE_DOMAIN`  | `localhost`                    | Cookie domain attribute              |

Place overrides in a `.env` file at the workspace root — loaded automatically.

## Extending for new services

1. Add a `[feature]` entry in `Cargo.toml` listing the service's optional deps.
2. Create `src/services/<service>.rs` with a `run(infra, config, root)` function.
3. Add `#[cfg(feature = "<service>")] pub mod <service>;` in `src/services/mod.rs`.
4. Call `services::<service>::run(...)` inside `run_services()` in `main.rs`.
```
