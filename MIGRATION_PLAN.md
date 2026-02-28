# MIGRATION_PLAN.md

Madome Monorepo Migration Execution Plan (Compat → Stabilize → vNext)
Date: 2026-02-25
Owner: Leader Agent (Claude Code Team Orchestration)

---

## 0) Prime Directive (Non‑Negotiables)

### 0.1 External contract is immutable (Compat)

The legacy system is the **source of truth**. During Compat:

- **MUST NOT change**
  - API paths (including prefixes, case, trailing slashes)
  - Response schemas (keys, types, nullability, enum values)
  - Status codes (including edge/invalid cases)
  - SPA routing and URL structure
- Gateway **MUST keep existing paths** (no rewrites).
- Gateway **MUST NOT**:
  - transform response bodies
  - modify cookies (including `Set-Cookie`)
  - spoof/forward externally supplied privileged headers

### 0.2 Auth model is immutable

- HttpOnly cookie-based auth.
- Access token: short TTL.
- Refresh token: long TTL.
- Refresh: **validate access + refresh together**, then reissue.
- Cookie attributes MUST match legacy **exactly**:
  - `Domain`, `Path`, `SameSite`, `Secure`, `HttpOnly`, expiry semantics

### 0.3 GitOps + secrets constraints

- Secrets: **SOPS + age** only.
- FluxCD remains.
- Flux MCP permitted actions: **read status / debug / trigger reconcile only**.
- All changes happen through **Git PRs** only (no imperative apply beyond reconcile triggers).

### 0.4 Deployment topology constraints

**prod**

- Web: `domain.com` (CloudFront)
- Backend: `api.domain.com`
- Images: `file.domain.com` (subdomain preserved)

**dev**

- Kubernetes deployment (web may be in-cluster).
- Dev-only domains (isolated from prod).
- Prefer same host split:
  - `dev-domain.com`
  - `api.dev-domain.com`
  - `file.dev-domain.com`
- Frontend config: **build-time env injection** (public config only; not secrets).

### 0.5 “Zero failing tests” is a hard gate

Compat requires:

- Unit + Integration + E2E-smoke in PR CI
- **0 failures allowed** (including skipped tests and flakiness)
- Merge is blocked if any test is failing or skipped

### 0.6 Team workflow hard gate

In team mode:

- Every agent must produce a **Plan**.
- **Leader approval is required before implementation. No exceptions.**
- Large work must be split into small PRs with explicit gates.

---

## 1) Objectives and Phase Scope

### 1.1 Phase A — Compat (Reimplement without breaking contract)

**Goal:** Replace implementation while keeping external behavior identical. Introduce Gateway and centralize auth/authorization enforcement **without** modifying cookies or responses.

**Compat deliverables**

- Monorepo established (workspace, CI gates, repo conventions).
- Gateway deployed (dev first, then prod) with strict guardrails.
- Services reimplemented behind Gateway:
  - users, library, auth, image, sync
- Outbox + Worker enabled **in Compat** (see §8), using existing schema semantics.
- Contract tests frozen and enforced:
  - status codes
  - response schemas
  - cookie `Set-Cookie` exactness
  - SPA routing parity
- GitOps K8s manifests in place (Flux + Kustomize). Helm removed.

**Compat exit criteria (all must be true)**

- Contract tests pass against new stack in dev and prod.
- Cookie contract tests pass byte-for-byte for `Set-Cookie` in each environment.
- Gateway guardrails verified by tests (no body transform, no cookie edits, spoofing blocked).
- Observability baseline exists (request-id propagation, structured logs, basic metrics).
- Rollback plan rehearsed (documented and executable).

### 1.2 Phase B — Stabilize (Harden + internal gRPC)

**Goal:** Improve internal architecture and operations without changing public contract.

**Stabilize deliverables**

- Internal HTTP service-to-service APIs removed/isolated.
- Internal communications migrated to **gRPC** (service mesh not required).
- Auth fully centralized:
  - Gateway is the single token verifier
  - services trust gateway identity headers
- Operational hardening:
  - timeouts, retries (bounded), backpressure, rate limits if contract-safe
- Expand observability (tracing optional) while keeping PII out of logs.

**Stabilize exit criteria**

- Public contract remains unchanged and fully covered by tests.
- gRPC interfaces versioned and documented; integration tests exist.
- Error budgets/SLOs defined; key metrics collected.

### 1.3 Phase C — vNext (Additive expansion)

**Goal:** Add new features without breaking Compat endpoints.

**vNext rules**

- Existing endpoints remain unchanged and contract-tested.
- New capabilities are **additive** (new endpoints or new optional fields only).
- Breaking changes require a new versioned surface (e.g., `/v2/...`) and do not alter existing paths.

---

## 2) Monorepo Layout (Selected Option + Rationale)

### 2.1 Selected top-level structure (normative)

We standardize on:

```text
apps/       # deployable user-facing applications (SPA)
services/   # deployable backend services (gateway, users, library, auth, image, sync, workers)
packages/   # shared libraries (Rust crates, TS packages, proto definitions)
tools/      # codegen, contract harness, CI scripts, migration helpers
k8s/        # GitOps (Flux + Kustomize overlays), SOPS secrets, Husako assets
contracts/  # immutable external contract fixtures + golden outputs
```

### 2.2 Rationale (why this works in Compat)

- Minimizes cross-cutting churn: deployables vs shared code are explicit.
- Keeps non-obvious constraints (contracts, secrets, GitOps) centralized and reviewable.
- Enables “affected-only” CI by path filtering.
- Provides a stable structure across Compat → Stabilize → vNext (avoid reorganizing later).

### 2.3 Required service placement (normative)

```text
services/
  gateway/
  auth/
  users/
  library/
  image/
  sync/
  worker/              # outbox processor(s) (Compat+)
tools/
  openapi-aggregate/
contracts/
  http/
  cookies/
  spa-routing/
  openapi/
k8s/
  flux/
  base/
  clusters/dev/
  clusters/prod/
  sops/
  husako/              # introduced after Compat for prod (see §10)
```

---

## 3) Work Breakdown Strategy (Small PRs, Explicit Gates)

### 3.1 Work item flow (Epic → Issue → PR)

- **Epic**: Phase-level objective (Compat Gateway, Compat Auth, Compat Users, etc.)
- **Issue**: Small, testable outcome (single route group, single guardrail, single pipeline change)
- **PR**: Implements exactly one Issue, includes:
  - Plan link
  - tests added first (or alongside, with “fails on current” evidence)
  - docs update when behavior/ops changes

### 3.2 “Plan → Annotate → Revise” loop (mandatory)

Each Issue must include a Plan that is refined before code changes:

1. **Plan (v1)**: assumptions + files to read + tests to add + smallest diff.
2. **Annotate**: record unknowns discovered while reading code/contracts.
3. **Revise (v2)**: update plan to remove unknowns and define gates.
4. **Leader approves** (team mode): implementation begins only after approval.

### 3.3 PR size limits (enforced socially; optionally via CI)

- One PR should change **one bounded unit**:
  - one service, or one gateway guardrail, or one CI pipeline rule
- Avoid “mega PRs” that mix infra + gateway + multiple services.
- If diff must be large, split into:
  - preparatory PR (tests + scaffolding)
  - implementation PR (minimum logic)
  - cleanup PR (refactor / docs)

### 3.4 Stop conditions

Implementation must stop (and the plan must be revised) if:

- contract tests reveal mismatched status codes or cookies
- gateway is observed mutating response bodies or `Set-Cookie`
- flakiness appears in CI (must be treated as failure)

---

## 4) Gateway Centralization (Compat Guardrails + Stabilize Transition)

### 4.1 Gateway responsibilities (Compat)

Gateway is policy + routing, **not** transformation.

MUST:

- Centralize authN/authZ checks (fail fast where legacy would fail).
- Preserve request path and query exactly.
- Pass response bodies byte-for-byte.
- Pass `Set-Cookie` byte-for-byte.
- Block header spoofing.
- Block routing to internal-only APIs.
- Generate and propagate `request-id`.
- Emit structured logs and baseline metrics.

MUST NOT:

- rewrite paths
- mutate response body
- mutate cookies
- accept privileged headers from the outside world

### 4.2 Reserved headers and spoofing rules (normative)

Gateway must treat the following as **reserved** and must drop/overwrite any inbound copies:

- `x-madome-*` (entire namespace)
- `x-request-id` (sanitize; replace if missing/invalid)
- `x-forwarded-*` (recompute)
- any “identity” header used between gateway and services

Gateway must add/force:

- `x-request-id: <generated>`
- `x-madome-public: <forced value>` for public routes
- internal identity headers for upstreams (only after successful auth), e.g.:
  - `x-madome-auth-sub`
  - `x-madome-auth-scope`
  - `x-madome-auth-session`

### 4.3 Routing model (strangler, path-preserving)

- Gateway initially routes 100% of paths to legacy backends.
- Cutover is done **per path group**, controlled by Git-managed config:
  - `services/gateway/config/routes.dev.yaml`
  - `services/gateway/config/routes.prod.yaml`

Rules:

- No rewrite: upstream receives the same path.
- No response transform: gateway is transparent.
- Each cutover must add/extend contract tests for that path group.

### 4.4 Stabilize transition (single enforcement + gRPC)

In Stabilize:

- Services stop verifying tokens directly.
- Gateway becomes the single token verifier.
- Services trust gateway identity headers **only** when requests originate from:
  - cluster network, and/or
  - a network policy boundary, and/or
  - a gateway-only shared secret header (internal only)

Internal HTTP is phased out:

- Replace service-to-service HTTP calls with gRPC clients.
- gRPC proto definitions live in `packages/proto/`.
- gRPC integration tests are required (see §6).

---

## 5) Testing Strategy (Unit + Integration + E2E, Zero Failures)

### 5.1 The test pyramid (required)

- Unit tests: domain logic, parsing, validation, error mapping.
- Integration tests: DB interactions, outbox semantics, gateway header handling.
- E2E-smoke tests: critical user flows across gateway + services + SPA.

**All three are required in PR CI.**

### 5.2 Contract tests are the “kill switch” for drift

Contracts are stored under `contracts/` and are immutable in Compat unless:

- the legacy behavior was mis-captured, and
- the fix is validated against legacy and approved

Contract categories:

- `contracts/http/**`: request/response golden assertions (headers + bodies)
- `contracts/cookies/**`: exact `Set-Cookie` strings for dev/prod
- `contracts/spa-routing/**`: SPA route parity (playwright)
- `contracts/openapi/**`: frozen public OpenAPI (Compat)

### 5.3 Cookie contract tests (non-negotiable)

- Assert **full `Set-Cookie` string equality**, including attributes and ordering when relevant.
- Validate refresh flow behavior: access+refresh must both be present/valid.

### 5.4 Gateway guardrail tests (non-negotiable)

- Body pass-through test: response bytes equal to upstream for representative endpoints.
- Cookie pass-through test: `Set-Cookie` unchanged.
- Spoofing tests: inbound `x-madome-auth-sub` must not reach upstream unchanged.
- Internal route block tests: internal-only endpoints denied as expected.

### 5.5 Flake policy

- Flaky tests are treated as failing tests.
- No `@ignore`, no “temporary skip” in PR CI.
- If a test is flaky, the next PR must:
  - quarantine via deterministic reproduction (same seed/container)
  - fix root cause
  - remove quarantine immediately

---

## 6) Observability (Logging, Request-ID, Metrics) — No Secrets

### 6.1 Request-ID rules

- Gateway generates `x-request-id` if missing.
- Propagate to all services and worker logs.
- Include request-id in all structured logs for request-scoped events.

### 6.2 Structured logging (normative schema)

All services and gateway must log JSON with stable keys:

- `timestamp`
- `level`
- `service`
- `env`
- `request_id`
- `route`
- `status`
- `latency_ms`
- `error_code` (internal stable code)
- `msg`

### 6.3 Sensitive data rules

MUST NOT log:

- access/refresh tokens
- cookies
- full request/response bodies unless explicitly redacted and required for debugging
- secrets from env

PII must be redacted or excluded (emails/phones/usernames depending on policy).

### 6.4 Baseline metrics (gateway + services)

At minimum:

- request count by route/status
- latency histogram by route
- upstream error counts
- outbox worker: processed/succeeded/failed/retried counts

---

## 7) Dev Image Stub Strategy (Deterministic + Cached + Range 206)

### 7.1 Contract constraints

- Image URL paths and host separation must remain consistent with prod.
- Dev uses a stub server; prod serves from local disk (3TB).

### 7.2 Stub capabilities (Compat minimum)

- Formats: avif/png/jpg/webp
- Deterministic generation: based on request path + seed
- Multiple ratios/resolutions
- Generate on demand + cache to disk
- HTTP Range (minimum):
  - `Range: bytes=start-end` → `206 Partial Content`
  - invalid ranges → `416`
- Progressive hardening plan:
  1. GET + 206 range
  2. HEAD support
  3. `Accept-Ranges: bytes`
  4. correct `Content-Length` and `Content-Range` for all variants
  5. caching headers parity if legacy specifies

### 7.3 Deterministic seed (normative)

- `seed = SHA256(<normalized_path> + ":" + <global_salt>)`
- Cache key includes:
  - path
  - negotiated format
  - requested size parameters (if any)
  - seed salt version

---

## 8) Outbox + Worker (Compat: at-least-once + idempotency)

### 8.1 Compat rule (schema preservation)

Compat must implement outbox + worker **using the existing legacy schema and semantics**.

Non-negotiable Compat checklist:

- [ ] Identify legacy outbox table(s) and columns (source of truth: DB + legacy code).
- [ ] Capture legacy semantics in tests:
  - when events are written
  - retry behavior
  - status transitions
  - idempotency strategy
- [ ] Implement worker to match, with at-least-once delivery.

If the legacy system does **not** have an outbox schema:

- Compat is **blocked** until the decision is made.
- The only acceptable path is an explicit, approved Alternative Proposal (see §14 A1).

### 8.2 Minimum worker semantics (must exist in Compat)

- Delivery guarantee: **at-least-once**
- Concurrency: safe multi-worker using `FOR UPDATE SKIP LOCKED` (or legacy equivalent)
- Idempotency:
  - enforce an idempotency key per event (use legacy key if exists)
  - downstream (FCM/SES) sends must be deduplicated by that key
- Retry policy:
  - bounded exponential backoff
  - max attempts
  - final failure state (dead-letter equivalent) with error record
- Observability:
  - structured logs with event_id/event_type
  - metrics for processed/succeeded/failed/retried

---

## 9) OpenAPI: Service Specs + Aggregate Static Deployment

### 9.1 Source layout

Each service owns its specs:

- `services/<svc>/openapi/public.yaml`
- `services/<svc>/openapi/internal.yaml` (must not be publicly published)

### 9.2 Aggregation pipeline

- Aggregator tool under `tools/openapi-aggregate/`.
- Output:
  - `contracts/openapi/public.yaml` (Compat: frozen to legacy behavior)
  - `dist/openapi/*` (static bundle, e.g., Swagger UI/Redoc)

### 9.3 Publication rules

- Public spec only.
- Internal spec never exposed on public domains.
- Location must not break SPA routing:
  - Prefer `api.domain.com/openapi/` unless confirmed safe under `domain.com`.

---

## 10) K8s / FluxCD / SOPS + Husako Adoption

### 10.1 GitOps baseline (Compat)

- Kustomize-only manifests in repo (`k8s/base`, `k8s/clusters/*`).
- Flux reconciles from Git.
- Helm is removed from this repo.

### 10.2 Secrets (SOPS + age)

- Secrets committed only as SOPS-encrypted files under `k8s/sops/`.
- No plaintext secrets in:
  - repo
  - CI logs
  - rendered manifests committed by accident

### 10.3 Husako adoption timing (hard rule)

- Husako is introduced to **prod only after Compat is complete**.
- Dev adoption is optional, but must still be PR-based and reproducible.

---

## 11) Data Migration: Postgres + Image Files (Checklist + Rollback)

### 11.1 Postgres migration (pg_dump)

**Preparation**

- [ ] Confirm Postgres versions and extensions compatibility.
- [ ] Prepare deterministic validation queries (row counts, checksums for critical tables).
- [ ] Document downtime and rollback triggers.

**Cutover**

- [ ] Freeze writes (maintenance mode or DB-level write block).
- [ ] `pg_dump` from source with consistent options.
- [ ] Transfer dump securely.
- [ ] Restore with `pg_restore` into target.
- [ ] Run validation checks:
  - schema diff sanity
  - row counts for critical tables
  - canary queries for key endpoints
- [ ] Switch backend traffic.
- [ ] Keep old DB read-only for rollback window.

**Rollback**

- [ ] Repoint traffic to old DB/backend.
- [ ] Preserve new DB state for forensic analysis.

### 11.2 Image file migration (3TB local disk)

**Preparation**

- [ ] Confirm target mount paths and permissions.
- [ ] Pre-sync with rsync (no downtime).
- [ ] Validate file counts and spot-check hashes.

**Cutover**

- [ ] Final rsync pass (minimal delta).
- [ ] Switch `file.domain.com` target.
- [ ] Monitor 404/5xx metrics.

**Rollback**

- [ ] Switch DNS/LB back to old image host.
- [ ] Keep old disk available until confidence threshold met.

---

## 12) sync External Access (EC2) — IP Allowlist + HMAC

### 12.1 Surface separation (hard rule)

- Sync endpoints must be on a dedicated path prefix (legacy-defined).
- Sync endpoints must not be mixed into public OpenAPI.
- Gateway must block access to sync endpoints for non-allowlisted IPs.

### 12.2 HMAC requirements (minimum)

HMAC signature should cover at least:

- timestamp
- nonce
- request method + path
- body hash (or canonical request hash)

Validation rules:

- Reject stale timestamps.
- Reject nonce replays (store recent nonces for a window).
- Use constant-time signature compare.

---

## 13) GitHub Governance Recommendations (Why + What)

### 13.1 Recommended branch protection (default)

For the default branch (e.g., `main`):

- Require PRs (no direct push).
- Require status checks:
  - unit
  - integration
  - e2e-smoke
  - lint/format
- Require code review (at least 1–2 approvals; include CODEOWNERS for interfaces).
- Block force pushes.
- Require up-to-date branch before merge.
- Prefer merge queue (reduces CI race conditions; preserves “tests green at merge”).

**Reason:** Compat is contract-sensitive; governance prevents accidental drift and enforces repeatable gates.

### 13.2 Interface ownership (CODEOWNERS)

Mark these as “interface files” requiring explicit review:

- `contracts/**`
- `services/gateway/config/**`
- `services/*/openapi/public.yaml`
- `packages/proto/**`
- `k8s/**` (especially `k8s/sops/**`)

---

## 14) Documentation Policy (Definition of Done)

Documentation is not optional.

### 14.1 DoD checklist (must be in every PR)

- [ ] Behavior changes captured in tests (unit/integration/e2e as applicable).
- [ ] Operational notes updated (runbooks, env vars, deployment notes).
- [ ] If public surface touched: OpenAPI updated + aggregate pipeline verified.
- [ ] If infra touched: k8s overlays documented (dev/prod differences).

### 14.2 What not to document

Do not restate what code makes obvious. Document only:

- non-obvious constraints
- sharp edges/pitfalls
- gates and how to satisfy them
- rollback steps

---

## 15) Risk Register and Mitigations

1. **Contract drift (paths/schemas/status codes/cookies)**
   - Mitigation: golden contract tests + cookie string equality + PR gate

2. **Gateway accidentally mutates responses or cookies**
   - Mitigation: gateway byte-for-byte pass-through tests + header/cookie assertions

3. **Auth regressions during centralization**
   - Mitigation: Compat dual enforcement; Stabilize single enforcement via staged rollout + monitoring

4. **Dev/prod cookie attribute mismatch**
   - Mitigation: per-env cookie contract files; environment-specific CI runs

5. **Outbox semantics mismatch**
   - Mitigation: capture legacy semantics in integration tests before reimplementation

6. **3TB image migration integrity**
   - Mitigation: staged rsync + validation + rollback window + monitoring

7. **Flaky CI leading to unsafe merges**
   - Mitigation: flake policy = failure; quarantine only with deterministic reproduction; remove quickly

---

## 16) Alternative Proposal (Only if constraints conflict)

### A1) If legacy has no outbox schema but Compat requires outbox

**Conflict:** “existing schema 유지” vs “outbox from Compat”.

Proposal (requires explicit approval):

- Introduce a minimal additive outbox table in Compat, behind a feature flag.
- Keep public API behavior identical (no contract changes).
- Add strict migration + rollback steps.

Pros:

- Enables required asynchronous processing immediately.
- Cleaner reliability story.

Cons/Risks:

- Violates “existing schema 유지” policy intent.
- Requires careful ops review and migration discipline.

Stepwise transition:

1. Add schema + worker in dev only
2. Validate with integration/e2e
3. Roll to prod with rollback plan
4. Freeze schema again until Compat exit

### A2) Gateway engine choice

Default: implement gateway as a controlled reverse proxy with explicit pass-through tests.
Alternative (requires ops buy-in): adopt a mature proxy with an external auth service.

- Constraints remain: no path rewrite, no body transform, no cookie modification.
- Requires stronger configuration/testing to guarantee invariants.

---
