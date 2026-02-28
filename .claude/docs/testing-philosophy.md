# Testing Philosophy Reference

> Reference doc for CLAUDE.md §4.2. Read before writing any test.

---

## The standard

If all tests pass, no manual check is needed. A test suite that requires a human to additionally
"just verify it works" has failed its purpose.

---

## Assert on values, not just absence of errors

```rust
// WRONG — only proves "didn't crash"
let resp = app.post("/auth/token").json(&body).await;
assert_eq!(resp.status(), 201);

// RIGHT — proves the actual contract
assert_eq!(resp.status(), 201);
let cookies = resp.headers().get_all("set-cookie");
assert_cookie_attr(cookies, "madome_access_token", "Path=/");
assert_cookie_attr(cookies, "madome_access_token", "Max-Age=604800");
assert_cookie_attr(cookies, "madome_access_token", "HttpOnly");
assert_cookie_attr(cookies, "madome_access_token", "SameSite=Lax");
assert!(resp.headers().contains_key("x-madome-access-token-expires"));
```

---

## Cover all observable behaviors

For every function/handler:

- Happy path: correct input → correct output (assert the full output, not a subset)
- Each error path: one test per distinct error case (404, 401, 403, 409, etc.)
- Boundary conditions: min/max values, empty lists, optional fields absent

---

## Verify side effects, not just response codes

State-changing operations must assert that the side effect actually occurred:

```rust
// WRONG — only proves response was 201, not that the outbox event was inserted
let resp = app.post("/books").json(&book_data).await;
assert_eq!(resp.status(), 201);

// RIGHT — also proves the outbox event was created for downstream processing
assert_eq!(resp.status(), 201);
let event = db.query_one("SELECT * FROM outbox_events WHERE kind = 'book_added'").await?;
assert_eq!(event.kind, "book_added");
assert!(event.processed_at.is_none()); // pending, not yet processed by worker
```

Concrete side effects to assert per layer:

- **Handlers**: response headers, cookies set/cleared attribute-by-attribute
- **Write use cases**: DB row inserted/updated — query it back and assert fields
- **Outbox events**: row in `outbox_events` with correct `kind`, `payload`, `idempotency_key`; `processed_at IS NULL`
- **Worker jobs**: mock external client (FCM, SMTP) received correct request with correct arguments
- **Cache writes**: key exists in Redis with correct value and TTL
- **gRPC calls**: mock gRPC server recorded the correct request message

Rule: if a use case has a side effect, there is a test that asserts it happened.

---

## Auth and role tests are not optional

Every auth-gated route must have:

- Test with valid token → succeeds
- Test with no token → 401
- Test with insufficient role → 403
- Test with expired/invalid token → 401

---

## Integration tests use real HTTP (TestApp), not mocked handlers

Unit test the use case logic; integration test the full HTTP path.
Do not mock the handler itself — that tests nothing real.

---

## Test naming: `should_` prefix, describe the contract

```rust
// WRONG
fn test_get_book() { }

// RIGHT — reads as a specification; failure message is self-explaining
fn should_return_404_when_book_does_not_exist() { }
fn should_return_401_when_token_missing() { }
fn should_set_both_cookies_with_max_age_604800_on_login() { }
```

Rule: all `#[test]` functions must start with `should_`. No exceptions.

**If a behavior has no test, it does not exist.**

---

## Self-contained E2E harnesses start the service themselves

E2E and contract harness binaries must start the service under test in-process — never rely
on an externally running server. This makes the full suite runnable with a single command.

**Pattern:**

```rust
// Random port — avoids conflict with a running service instance
let listener = TcpListener::bind("127.0.0.1:0").await?;
let port     = listener.local_addr()?.port();
let base_url = format!("http://127.0.0.1:{port}");

// Start service as a background task
tokio::spawn(async move {
    axum::serve(listener, build_router(state)).await.unwrap();
});
```

- Call `dotenv::dotenv().ok()` at startup for dev convenience (`.env` at workspace root).
- Run migrations automatically (`Migrator::up(&db, None)`) — idempotent, safe to repeat.
- Build `AppState` directly from `std::env::var()` — no intermediate config struct needed in
  test binaries.
- Service-startup deps (`madome-auth`, `madome-auth-migration`, `sea-orm`, `deadpool-redis`,
  `webauthn-rs`) belong in the harness crate only — do not add them to the service's own
  `Cargo.toml`.

**Migration lib extraction:** When a migration crate is `[[bin]]`-only, extract the
`Migrator` struct into a `[lib]` target so harness binaries can import and call
`Migrator::up(&db, None)` programmatically. The `[[bin]]` target then delegates to the lib:

```rust
// migration/src/main.rs — thin wrapper after lib extraction
use madome_auth_migration::Migrator;

#[tokio::main]
async fn main() {
    sea_orm_migration::cli::run_cli(Migrator).await;
}
```

---

## Docker-orchestrated harnesses (for services requiring a real database/cache)

When the service under test needs PostgreSQL + Redis, spin them up as Docker containers
instead of requiring manual infra setup. This keeps the single-command guarantee.

Pattern (using `bollard` + `fd-lock`):

```rust
// 1. Exclusive lock — only one harness instance at a time.
//    OS auto-releases even on crash/panic (no stale lock).
let lock_path = std::env::temp_dir().join("madome-contract-harness.lock");
let lock_file = std::fs::File::create(&lock_path)?;
let mut lock  = fd_lock::RwLock::new(lock_file);
let _guard    = lock.try_write().map_err(|_| anyhow!("another instance is running"))?;

// 2. Connect to Docker daemon (DOCKER_HOST env var → local socket or remote TCP).
let mut orch = DockerOrchestrator::connect(&config.docker_host).await?;

// 3. Crash recovery — remove non-running test containers from a previous run.
orch.cleanup_stale().await?;

// 4. Start infra containers on random host ports.
let database_url = orch.start_postgres().await?;
let redis_url    = orch.start_redis().await?;

// 5. Run services in-process against the containers.
let result = run_services(&infra, &config, &workspace_root).await;

// 6. Always tear down (success or failure — never use Drop for this).
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
- Pull the image with `docker.create_image()` before `create_container()` — Docker returns
  404 if the image is not already present locally.
- Containers use `HostPort = ""` (random port); inspect after start to obtain the mapped
  port. `tcp://HOST:PORT` → use HOST as the container address; `unix://...` → use `127.0.0.1`.
