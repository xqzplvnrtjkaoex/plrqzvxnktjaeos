# MIGRATION_PLAN.md

**Project:** Madome Monorepo + Gateway Centralization + Phased Migration (Compat → Stabilize → vNext)
**Org:** `xqzplvnrtjkaoex`
**Date:** 2026-02-23 (Asia/Seoul)
**Audience:** Claude Code Team Agents + Reviewers
**Status:** Execution Plan (authoritative for Compat constraints)

---

## 0. Non‑Negotiables (Read First)

### 0.1 External contract (MUST NOT change)

- [ ] **API paths MUST remain identical** (including trailing slashes, case, query semantics).
- [ ] **Response schema MUST remain identical** (JSON keys, types, enum values, nullability, field order if clients depend on it).
- [ ] **Status codes MUST remain identical** (including edge cases).
- [ ] **SPA routing + URL structure MUST remain identical** (no route renames, no base path changes).
- [ ] **Gateway MUST preserve existing external paths** (strangler routing is allowed, path rewriting is not).

### 0.2 Authentication model (MUST replicate 1:1)

- [ ] **HttpOnly cookie** based session model.
- [ ] **Access token:** short TTL (existing TTL).
- [ ] **Refresh token:** long TTL (existing TTL).
- [ ] **Refresh flow:** access + refresh MUST both be validated before reissuing.
- [ ] **Cookie attributes MUST be byte‑for‑byte compatible** with legacy per environment:
  - `Set-Cookie: Domain`
  - `Path`
  - `SameSite`
  - `Secure`
  - `HttpOnly`
  - expiration / max‑age semantics

### 0.3 Infrastructure (MUST)

- [ ] **Secrets:** SOPS + age (no plaintext secrets in Git).
- [ ] **Flux MCP:** read / debug / reconcile trigger only.
- [ ] **All changes MUST be PR-based** (no imperative cluster mutation beyond reconciles).

### 0.4 Deployment topology (MUST keep host structure)

**prod**

- [ ] `domain.com` → Web (CloudFront)
- [ ] `api.domain.com` → Backend
- [ ] `file.domain.com` → Image host (subdomain preserved)

**dev**

- [ ] Kubernetes deployment (web may be in-cluster).
- [ ] Dedicated dev domains (not prod domains).
- [ ] Prefer same host split as prod:
  - `dev-domain.com`
  - `api.dev-domain.com`
  - `file.dev-domain.com`
- [ ] Web config uses **build-time env injection** (public config only).

### 0.5 Gateway in Compat (guardrails)

Gateway MUST:

- [ ] Centralize authN/authZ.
- [ ] **MUST NOT transform response bodies** (no JSON rewrites, no schema changes).
- [ ] **MUST NOT change paths**.
- [ ] **MUST NOT modify cookies** (including `Set-Cookie` pass-through).
- [ ] Block header spoofing:
  - [ ] remove/overwrite external `x-madome-*` and other reserved headers
  - [ ] enforce `x-madome-public` (and related forced headers)
- [ ] Block external routing to internal-only APIs.
- [ ] Create and propagate `request-id`.
- [ ] Emit structured logs and baseline metrics.

### 0.6 sync (MUST)

- [ ] Called from external EC2.
- [ ] Enforce **IP allowlist + HMAC**.
- [ ] Separate sync-only endpoints from public API surface.

---

## 1. Migration Phases: Goals, Rules, Exit Criteria

### 1.1 Phase A — **Compat**

**Goal:** Replace implementation while keeping _identical external behavior_. Introduce Gateway while preserving contract.

**Compat deliverables**

- [ ] Monorepo bootstrapped (workspace, tooling, CI).
- [ ] Gateway deployed in dev, then prod, keeping same public paths.
- [ ] Service reimplementations (Rust axum) behind Gateway, path-by-path cutover.
- [ ] Contract tests locked and running in CI (cookie and status-code exactness included).
- [ ] GitOps in place (Flux + Kustomize). Helm removed from this repo.

**Compat exit criteria (MUST all be true)**

- [ ] Contract test suite passes against new stack.
- [ ] Cookie contract tests pass byte-for-byte on `Set-Cookie` (per env).
- [ ] Gateway guardrails verified (no body transform; no cookie edits; spoofing blocked).
- [ ] Observability baseline (request-id end-to-end + structured logs + metrics).
- [ ] Prod cutover completed with rollback plan validated.

### 1.2 Phase B — **Stabilize**

**Goal:** Reduce tech debt and harden operations without breaking Compat contract.

**Stabilize deliverables**

- [ ] Internal HTTP APIs removed/isolated.
- [ ] Internal service-to-service moved to **gRPC**.
- [ ] Auth logic fully centralized (services stop duplicating auth verification; they trust Gateway identity headers).
- [ ] Operational hardening: rate limits (if already present), retries, timeouts, circuit breakers.
- [ ] Outbox + Worker introduced **only if it does not violate “existing schema” constraints** (see §7).

**Stabilize exit criteria**

- [ ] Public contract unchanged and still fully covered by contract tests.
- [ ] gRPC call graph documented and integration-tested.
- [ ] Error budgets / SLOs defined with metrics present.

### 1.3 Phase C — **vNext**

**Goal:** Add new capabilities while keeping Compat endpoints stable.

**vNext principles**

- [ ] New external features MUST be additive (new endpoints / new optional fields only).
- [ ] Legacy endpoints remain supported with contract tests unchanged.
- [ ] Any breaking change MUST be introduced as a new versioned surface (e.g., `/v2/...`) and **MUST NOT** alter existing paths.

---

## 2. Monorepo Directory Layout (Selected)

### 2.1 Chosen layout

We will use **all** of these top-level directories, with strict meanings:

```text
apps/       # deployable user-facing apps (SPA, docs site if needed)
services/   # deployable backend services (Rust axum, workers)
packages/   # shared libraries (Rust crates, TS packages, proto definitions)
tools/      # codegen, contract test harness, CI helpers, local dev utilities
k8s/        # GitOps: Flux + Kustomize overlays, SOPS secrets, Husako outputs
```

### 2.2 Rationale (why this layout)

- **Clear separation** between deployables (`apps/`, `services/`) and shared code (`packages/`).
- **Tooling is isolated** in `tools/` so CI and dev flows don’t pollute runtime artifacts.
- **GitOps and secrets are centralized** in `k8s/` for Flux-driven reconciliation.
- This layout scales to **Compat → Stabilize → vNext** without reorganizing paths (avoid churn).

### 2.3 Required repo skeleton (paths are normative)

```text
.
├─ Cargo.toml                 # Rust workspace root
├─ package.json               # JS workspace root (SPA + tooling)
├─ pnpm-workspace.yaml        # (or yarn workspaces) - pick one and standardize
├─ .github/workflows/         # CI
├─ contracts/                 # locked external contract tests + golden data
│  ├─ http/                   # request/response golden snapshots
│  ├─ cookies/                # cookie attribute expectations (dev/prod)
│  └─ openapi/                # baseline public OpenAPI (frozen in Compat)
├─ apps/
│  └─ web/                    # SPA
├─ services/
│  ├─ gateway/                # reverse proxy + auth centralization
│  ├─ users/
│  ├─ library/
│  ├─ auth/
│  ├─ image/
│  └─ sync/
├─ packages/
│  ├─ rust/
│  │  ├─ madome-core/         # shared types, error model, tracing helpers
│  │  ├─ madome-auth/         # token verify helpers (Compat: dual-use)
│  │  └─ madome-grpc/         # gRPC client/server utilities (Stabilize)
│  ├─ ts/
│  │  ├─ madome-public-config/ # typed build-time config for SPA
│  │  └─ madome-api-client/    # generated client (optional)
│  └─ proto/                  # gRPC protos (Stabilize)
├─ tools/
│  ├─ contract-tests/         # hurl / k6 / playwright harness
│  ├─ openapi-aggregate/      # merges service specs -> static artifact
│  └─ ci/                     # path filters, shared scripts
└─ k8s/
   ├─ flux/                   # Flux bootstrap manifests (no secrets)
   ├─ base/                   # kustomize bases
   ├─ clusters/
   │  ├─ dev/
   │  └─ prod/
   ├─ sops/                   # encrypted secrets (age)
   └─ husako/                 # Husako templates and/or generated output
```

---

## 3. Workstreams and Execution Order (with Parallelism)

### 3.1 Workstream overview

**WS0 — Contract capture & locking (starts first)**

- Establish contract tests and “golden” outputs for:
  - API status codes and payload schemas
  - Cookie `Set-Cookie` exact attributes
  - SPA routing expectations (client-side routes + entrypoint behavior)

**WS1 — Monorepo bootstrap**

- Workspaces, linting, formatting, shared libs, CI scaffolding.

**WS2 — Gateway (Compat guardrails)**

- Reverse proxy + centralized auth check + spoofing prevention + request-id.

**WS3 — Service reimplementations (Compat)**

- `users`, `library`, `auth`, `image`, `sync` implemented in Rust axum.

**WS4 — Web app**

- Build-time config injection + routing parity + deploy shape parity.

**WS5 — Infra / GitOps**

- Flux + Kustomize overlays for dev, then prod; SOPS + age integrated.
- Husako introduced **after Compat** for prod (dev optional).

**WS6 — Data migration**

- Postgres migration via `pg_dump` and restore.
- Image file migration for prod disk store.

### 3.2 Dependency graph (hard dependencies)

- WS0 → (everything)
- WS1 → WS2/WS3/WS4/WS5 (workspace conventions)
- WS2 (Gateway) ↔ WS3 (Services) are iterative, but:
  - First Gateway skeleton MUST exist before any path cutover.
- WS5 (Infra) required before dev deployments of WS2/WS3/WS4.
- WS6 (Migration) starts after service behavior is contract-passing in dev.

### 3.3 Parallelization strategy (optimized for M2 16GB dev laptops)

Run these in parallel immediately after WS0:

- [ ] WS1 Monorepo bootstrap
- [ ] WS2 Gateway skeleton + tests (unit + integration)
- [ ] WS4 Web build pipeline (build-time env injection + static hosting shape)
- [ ] WS5 Dev GitOps scaffolding (Flux + Kustomize + SOPS wiring)

Run WS3 services in parallel **per service**, but only after:

- [ ] shared error/logging primitives exist in `packages/rust/madome-core`
- [ ] contract tests exist for that service’s routes

Keep local dev lightweight:

- [ ] Prefer `docker compose` for Postgres only.
- [ ] Avoid running full E2E locally; use PR CI for heavy suites.
- [ ] Provide `tools/ci` scripts to run _only changed area_ tests.

---

## 4. Gateway Centralization Strategy (Compat → Stabilize)

### 4.1 Gateway architecture (Compat)

**Gateway = policy enforcement + routing**, not a transformer.

Gateway responsibilities:

- **AuthN/AuthZ pre-check** for protected routes.
- **Header sanitization** to prevent spoofing.
- **Routing** to legacy or new services without changing external paths.
- **Request-ID**: generate if missing; propagate to upstream; echo in logs.
- **Structured logs**: JSON logs with stable keys.
- **Baseline metrics**: request count, latency, upstream errors by route/service.

Gateway MUST NOT:

- Modify response bodies.
- Modify `Set-Cookie` headers.
- Rewrite paths or query strings.

### 4.2 Strangler routing plan (path-by-path cutover)

- [ ] Start with Gateway in dev routing **100%** to legacy backends.
- [ ] Introduce **route toggles** (config file in Git) to move a path prefix to a new service.
- [ ] Cut over one bounded context at a time:
  1. `auth` endpoints (to validate cookie contract early)
  2. `users`
  3. `library`
  4. `image` (dev stub first, prod disk later)
  5. `sync` (internal path only)

**Routing config MUST be versioned in Git**, e.g.:

- `services/gateway/config/routes.dev.yaml`
- `services/gateway/config/routes.prod.yaml`

### 4.3 Header spoofing protection (Compat baseline)

Define a **reserved header set**; Gateway MUST:

- Drop any inbound header matching:
  - `x-madome-*`
  - `x-request-id` (sanitize; accept only if valid UUID/ULID, else replace)
  - `x-forwarded-*` (recompute)
- Add/force:
  - `x-request-id: <id>`
  - `x-madome-public: 1` (or exact expected value) for public routes
  - `x-madome-auth-sub`, `x-madome-auth-scope`, `x-madome-auth-sid` (internal identity headers)

**Upstreams MUST treat these as authoritative only when**:

- `x-madome-public` is present AND
- a second internal header is present (e.g., `x-madome-gw: 1`) AND
- requests originate from the cluster network (enforced via network policy when available)

### 4.4 Gateway auth modes (Compat → Stabilize)

**Compat (“dual enforcement”)**

- Gateway validates cookies/tokens and blocks obvious failures early.
- Upstream services KEEP existing auth logic (transition mode).

**Stabilize (“single enforcement”)**

- Gateway is the only token verifier.
- Upstreams trust gateway identity headers and enforce authorization via claims.

### 4.5 Gateway contract tests (required)

- [ ] Body pass-through tests: byte-for-byte equality against golden fixtures for representative endpoints.
- [ ] Cookie pass-through tests: `Set-Cookie` attributes unchanged.
- [ ] Spoofing tests: inbound `x-madome-auth-sub` must be dropped and replaced.
- [ ] Internal route blocking tests: internal endpoints return expected deny behavior (403/404 per policy).

---

## 5. Cookie/Auth Contract Testing Strategy (TDD + Golden)

This section defines the **non-negotiable** method to prevent accidental contract drift.

### 5.1 Source of truth

- The **legacy implementation behavior** is the contract for Compat.
- Contract tests are stored in Git and run on every PR.

### 5.2 Contract artifacts (must live in `contracts/`)

- `contracts/http/**`:
  - request definitions
  - golden responses (headers + body) with deterministic fixtures
- `contracts/cookies/dev.yaml` and `contracts/cookies/prod.yaml`:
  - explicit `Set-Cookie` strings for access/refresh cookies
  - NOTE: domain differs per env; everything else must match legacy for that env

### 5.3 Test harness

**Required tooling**

- [ ] `hurl` for HTTP contract assertions (cookies, headers, bodies).
- [ ] `playwright` for SPA route compatibility smoke tests.
- [ ] (Optional) `k6` for lightweight gateway load smoke.

**Rule:** Every new/ported endpoint MUST add or update a contract test first:

1. Write/adjust test → it fails on new stack.
2. Implement endpoint → test passes.

### 5.4 One-time capture procedure (how to generate golden responses)

This procedure must be run in a controlled environment with a known fixture DB:

- [ ] Step 1: Create a deterministic fixture dataset in Postgres.
- [ ] Step 2: Run the legacy stack against that dataset.
- [ ] Step 3: Execute `tools/contract-tests/capture.sh`:
  - sends canonical requests
  - saves response status, headers, and body under `contracts/http/golden/`
- [ ] Step 4: Commit the golden artifacts in a PR.
- [ ] Step 5: CI uses the same fixture dataset and compares new stack output to golden.

**PII rule:** No production personal data may be stored in golden artifacts.

---

## 6. dev Stub Image Strategy (dev-only)

**Goal:** Enable front-end and API flows in dev without relying on prod disk images.

### 6.1 Requirements checklist (must meet)

- [ ] Formats: `avif`, `png`, `jpg`, `webp`
- [ ] Multiple aspect ratios and resolutions
- [ ] Deterministic generation based on path + seed
- [ ] Generate-on-demand + caching
- [ ] Implement minimal HTTP Range support:
  - [ ] Accept `Range: bytes=<start>-<end>`
  - [ ] Return `206 Partial Content` with `Content-Range`
  - [ ] Return `416` for invalid ranges

### 6.2 Proposed endpoint shape

- Host: `file.dev-domain.com`
- Path: **MUST match prod external contract** for image URLs (no route changes).
- Behavior: If requested asset missing, stub server generates and serves a deterministic placeholder.

### 6.3 Implementation notes (Rust)

- Use `image` crate + format encoders (avif may require feature flags / external libs).
- Deterministic seed:
  - `seed = sha256(path + ":" + global_salt)`
- Cache strategy:
  - local filesystem cache under a writable dir (e.g., `/var/cache/madome-image-stub`)
  - key = sha256(request_path + accept/format + size params)
- Range support:
  - Serve cached bytes; slice range; set `Content-Length` and `Content-Range` correctly.

### 6.4 Compat boundary

- Stub behavior is **dev-only**.
- Prod image service uses **local disk (3TB)** and does not generate assets.

---

## 7. Notifications: DB Outbox + Worker (Minimal Viable Plan)

**Constraint:** “Existing schema 유지 (Compat 완료 후 개선)” means **Compat must not introduce schema changes** unless the legacy already has the outbox table.

### 7.1 Compat

- [ ] If legacy already uses an outbox table: replicate table semantics and worker behavior exactly.
- [ ] If legacy does not: do not introduce new tables in Compat. Stub notifications with no-ops **only if** the external API contract is unaffected.

### 7.2 Stabilize (introduce outbox if missing; additive)

**Minimum viable schema (additive)**

- Table `outbox_events` (or existing name) with:
  - `id` (PK)
  - `aggregate_type`, `aggregate_id`
  - `event_type`
  - `payload_json`
  - `idempotency_key`
  - `status` (`pending|processing|sent|failed`)
  - timestamps

**Worker loop**

- [ ] poll with `FOR UPDATE SKIP LOCKED`
- [ ] publish to:
  - FCM (push)
  - AWS SES (email)
- [ ] idempotency enforced by `idempotency_key`
- [ ] retry with bounded backoff and dead-letter semantics (failed status + last_error)

**Observability**

- [ ] metrics: processed count, failures, retry count, latency
- [ ] logs: include `event_id`, `event_type`, `request-id` if present

---

## 8. OpenAPI Aggregation + Static Deployment

**Goal:** Provide a single canonical public OpenAPI artifact while keeping Compat contract frozen.

### 8.1 Source inputs

Each service maintains:

- `services/<svc>/openapi/public.yaml` (public surface only)
- `services/<svc>/openapi/internal.yaml` (internal-only; NOT externally routed)

### 8.2 Aggregation pipeline

- Tool: `tools/openapi-aggregate`
- Steps:
  1. Validate each spec (`spectral` ruleset in repo)
  2. Merge public specs into `contracts/openapi/public.yaml`
  3. Emit a static bundle:
     - `dist/openapi/public.yaml`
     - `dist/openapi/index.html` (Swagger UI or Redoc)

### 8.3 Deployment strategy

- CI builds the static bundle as an artifact.
- GitOps deploy option (recommended):
  - Serve from `domain.com/openapi/` (CloudFront behavior) **only if it does not conflict with SPA routes**.
  - Otherwise serve from `api.domain.com/openapi/`.
- Internal-only specs MUST NOT be published on public hosts.

---

## 9. GitOps: Flux + Kustomize (No Helm) + Husako Rollout

### 9.1 Kustomize-first baseline (Compat)

- [ ] `k8s/base/*` contains Kustomize bases per app/service.
- [ ] `k8s/clusters/dev/*` overlays include dev domains, replicas, resource limits.
- [ ] `k8s/clusters/prod/*` overlays include prod domains and prod resource profiles.
- [ ] Flux `GitRepository` + `Kustomization` objects in `k8s/flux/`.

### 9.2 Husako adoption rule (MUST)

- **Prod:** Husako is applied **after Compat exit** and only via PRs.
- **Dev:** Husako use is optional; if used, it MUST still generate Kustomize-consumable output committed in Git.

### 9.3 SOPS + age secret strategy

- [ ] All secrets stored encrypted under `k8s/sops/**`.
- [ ] Age public keys can be committed; private keys MUST NOT.
- [ ] Flux decrypts secrets during reconciliation (SOPS integration).
- [ ] Secret naming conventions:
  - `madome-<svc>-secrets` per namespace
- [ ] Rotation procedure documented and executed via PR (update encrypted secret + rotate key outside Git).

---

## 10. CI Strategy (Monorepo)

### 10.1 PR checks (fast path)

**Rule:** Only run what changed when feasible.

- [ ] Lint/format (Rust + TS)
- [ ] Unit tests for changed crates/packages
- [ ] Contract tests relevant to changed service (hurl subset)
- [ ] Lightweight E2E smoke (1–3 critical flows)

Implementation approach:

- GitHub Actions `paths-filter` to detect changed areas.
- `tools/ci/run-affected.sh` to map files → test targets.

### 10.2 main / nightly (heavy path)

- [ ] Full unit test suite
- [ ] Full contract suite
- [ ] Full E2E suite
- [ ] `kind` cluster bring-up + GitOps reconcile + smoke
- [ ] Optional k6 gateway smoke (short duration)

### 10.3 Local developer constraint (M2 16GB)

- Local default command MUST be lightweight:
  - `just test` runs unit + small contract subset
  - heavy suites only in CI by default

---

## 11. DB + File Migration Strategy (pg_dump + file copy)

### 11.1 Postgres migration (pg_dump)

**Pre-cutover**

- [ ] Agree on downtime window and rollback triggers.
- [ ] Verify target Postgres version compatibility.
- [ ] Create target DB roles/users matching legacy privileges.
- [ ] Prepare restore scripts in `tools/db-migrate/`.

**Cutover procedure**

- [ ] Freeze writes (application maintenance mode) OR enforce write block at DB.
- [ ] Run `pg_dump --format=custom --no-owner --no-privileges` from source.
- [ ] Transfer dump to target via secure channel.
- [ ] Restore with `pg_restore --clean --if-exists` as needed.
- [ ] Run verification checks:
  - schema checksum (pg_catalog diff)
  - row counts for critical tables
  - application health checks
- [ ] Switch DNS / routing to new backend.
- [ ] Keep source DB read-only for rollback window.

### 11.2 File migration (prod image disk)

**Goal:** Move 3TB disk-backed image store without breaking URLs.

- [ ] Preserve directory structure and filenames (URLs must remain stable).
- [ ] Use `rsync -aHAX --numeric-ids` (or equivalent) to copy to new host.
- [ ] Validate:
  - file count parity
  - spot-check hashes
  - permission parity
- [ ] Cut over `file.domain.com` to new host (DNS or LB target switch).
- [ ] Keep old disk available for rollback until confidence threshold met.

---

## 12. Risk Register (with mitigations)

### 12.1 Highest risks

1. **Contract drift (API and cookies)**
   - Mitigation: locked golden contract tests; cookie string equality tests; PR blocking.

2. **Gateway accidentally mutates responses**
   - Mitigation: gateway pass-through tests + byte comparison for representative payloads.

3. **Auth regression during centralization**
   - Mitigation: dual enforcement in Compat; switch to single enforcement only in Stabilize with staged rollout.

4. **Dev/prod domain differences affecting cookies**
   - Mitigation: explicit per-env cookie contract files (`contracts/cookies/*.yaml`) and environment-aware tests.

5. **3TB image migration time / integrity**
   - Mitigation: staged rsync (pre-sync + final sync), checksums, rollback window.

6. **Flux reconciliation drift**
   - Mitigation: all config in Git; enforce PR review; use Flux diff + reconcile triggers.

---

## 13. Alternative Proposal (MUST NOT violate Non‑Negotiables)

### A1. Gateway implementation choice (proxy engine)

**Default (in-scope):** Rust (axum) Gateway for maximum control over cookie/header pass-through.
**Alternative:** Use Envoy as L7 proxy with external auth (ext_authz) service implemented in Rust.

Constraints honored:

- No path rewrite.
- No response body transform.
- No cookie modifications.
- Spoofing prevention via header normalization rules.

Tradeoffs:

- Envoy adds operational complexity but provides mature routing and metrics.
- Rust-only gateway is simpler but requires careful proxy correctness testing.

### A2. Husako unknowns (contingency)

If Husako adoption is blocked due to missing tool availability or unclear spec:

- Continue Kustomize-only in dev and prod while **keeping the `k8s/husako/` contract**.
- When Husako becomes available, introduce it via PRs that generate Kustomize-compatible outputs.

This preserves:

- “no Helm” requirement
- Flux + Git PR workflow
- Compat constraints

---
