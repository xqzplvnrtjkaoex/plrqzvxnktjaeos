# contract-harness

HTTP contract test runner for Madome backend services. Two modes:

## URL mode (default — no service features compiled in)

Runs contract fixtures against an **already-running** service or gateway.

```bash
# All services via gateway
cargo run -p contract-harness -- --base-url http://localhost:3000

# Auth service only
cargo run -p contract-harness -- --base-url http://localhost:3112 --service auth
```

## Docker mode (service feature flags)

Pulls `postgres:18` and `redis:8` if not cached locally, spins up containers,
starts the service in-process, runs all fixtures, then always tears everything
down — pass or fail. Requires a running Docker daemon.

```bash
# Local Docker socket (default):
cargo run -p contract-harness --features auth

# Remote Docker daemon:
DOCKER_HOST=tcp://192.168.1.100:2376 cargo run -p contract-harness --features auth
```

Only one instance may run at a time. A second concurrent run exits immediately:
`another instance is running`.

## Environment variables (Docker mode)

| Variable          | Default                       | Description                     |
|-------------------|-------------------------------|---------------------------------|
| `DOCKER_HOST`     | `unix:///var/run/docker.sock` | Docker daemon URL               |
| `JWT_SECRET`      | `test-contract-secret`        | HMAC secret for token signing   |
| `WEBAUTHN_RP_ID`  | `localhost`                   | WebAuthn relying-party ID       |
| `WEBAUTHN_ORIGIN` | `http://localhost`            | WebAuthn relying-party origin   |
| `COOKIE_DOMAIN`   | `localhost`                   | Cookie domain attribute         |

Place overrides in a `.env` file at the workspace root — loaded automatically.

## Extending for new services

1. Add a `[feature]` entry in `Cargo.toml` listing the service's optional deps.
2. Create `src/services/<service>.rs` with a `run(infra, config, root) -> Result<bool>` function.
3. Add `#[cfg(feature = "<service>")] pub mod <service>;` in `src/services/mod.rs`.
4. Call `services::<service>::run(...)` inside `run_services()` in `src/main.rs`.
