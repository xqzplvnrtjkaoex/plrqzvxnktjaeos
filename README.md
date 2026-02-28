# Madome

A web-based comic reader focused on browsing, organizing, and reading large collections smoothly.

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Backend services | Rust (tokio + axum) |
| Gateway | Rust (pingora) |
| Frontend | Vue 3 SPA |
| Database | PostgreSQL (sea-orm) |
| Auth | HttpOnly cookie-based JWT |
| Internal RPC | gRPC (tonic) |
| Notifications | FCM + SES (outbox pattern) |
| Infra | Kubernetes, FluxCD, SOPS + age |

## Quickstart

```bash
# Check all crates compile
cargo check --workspace

# Format (must be clean in CI)
cargo fmt --all

# Lint (zero warnings required)
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Tests
cargo test --workspace --all-features
```

## Monorepo Structure

```
apps/        # deployable user-facing applications (Vue 3 SPA)
services/    # deployable backend services
crates/      # shared Rust library crates
packages/    # shared cross-language packages (proto definitions)
tools/       # codegen, contract harness, e2e smoke
contracts/   # immutable external contract fixtures
k8s/         # GitOps (Flux + Kustomize), SOPS secrets
docs/        # multi-service topology, operational runbooks
previous/    # legacy source — READ-ONLY reference
```

See `CLAUDE.md` for team operating rules and contribution guidelines.
