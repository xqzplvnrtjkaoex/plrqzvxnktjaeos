# CLAUDE.md

## Writing Docs

- Always respond in English and write documents in English.
- Before writing docs, see <https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing> and avoid these patterns.

## 0. Shared Rules (Apply to EVERY Agent)

### 0.1 Immutable constraints (MUST NOT violate)

- External API paths, response schemas, and status codes MUST NOT change in Compat.
- SPA routing and URL structure MUST NOT change in Compat.
- Gateway MUST NOT transform response bodies, rewrite paths, or modify cookies (including `Set-Cookie`).
- Auth model is HttpOnly cookie-based with access + refresh tokens; refresh validates both.
- Secrets MUST be SOPS + age; all changes via Git PR.
- Flux MCP is read/debug/reconcile-trigger only; no imperative apply changes.

### 0.2 TDD rule (mandatory)

Every agent MUST follow:

1. Write/adjust test first
2. Observe failure
3. Implement minimum change
4. Pass tests
5. Refactor without changing behavior

### 0.3 “Interface freeze” rule

Anything that affects cross-team integration MUST be treated as an interface:

- Gateway routing config
- `contracts/**` artifacts
- OpenAPI public spec
- gRPC proto definitions (Stabilize+)
- DB schema (Compat: frozen)
- Cookie contract files

Interface changes REQUIRE:

- [ ] A dedicated PR
- [ ] Review from Leader Agent + impacted Service Agents
- [ ] Updated contract tests

### 0.4 Definition of Done (per PR)

- [ ] Tests added/updated first (evidence: failing commit or PR description)
- [ ] Unit tests pass
- [ ] Relevant contract tests pass
- [ ] Lint/format pass
- [ ] Docs updated if behavior/ops changed (MIGRATION_PLAN or service README)

---

## 1. Leader Agent (Orchestrator)

### Responsibilities

- Own the phased plan execution (Compat → Stabilize → vNext).
- Enforce non-negotiables and resolve conflicts between agents.
- Approve interface changes and coordinate sequencing.
- Maintain “single source of truth” docs:
  - `MIGRATION_PLAN.md`
  - `AGENTS.md`
  - repo-level conventions (workspace, linting, CI)

### Inputs (read)

- `/contracts/**`
- `/services/**`
- `/apps/**`
- `/packages/**`
- `/tools/**`
- `/k8s/**`
- CI results and PR diffs

### Outputs (write)

- `MIGRATION_PLAN.md` updates
- `AGENTS.md` updates
- `tools/ci/*` conventions (only when needed to unblock)
- Repo-level governance files (CODEOWNERS, PR templates)

### Prohibitions

- MUST NOT implement business logic inside services unless acting as a temporary unblocker (then transfer ownership).
- MUST NOT bypass PR review gates.

### Test ownership

- End-to-end orchestration checks:
  - Contract suite completeness
  - Golden artifact hygiene
  - “affected tests” CI correctness

### Phase scope

- **Compat:** sequencing + cutover approvals + contract locking
- **Stabilize:** gRPC migration governance + auth single-enforcement switch
- **vNext:** versioning policy enforcement and feature flag strategy

---

## 2. Service Agents (users / library / auth / image)

Each Service Agent owns one service implementation under `services/<svc>/`.

### Common responsibilities (all service agents)

- Reimplement legacy endpoints in Rust (tokio + axum) with identical behavior.
- Maintain service OpenAPI specs:
  - `services/<svc>/openapi/public.yaml`
  - `services/<svc>/openapi/internal.yaml`
- Maintain service contract tests:
  - add/adjust tests under `contracts/http/<svc>/`
- Maintain DB interactions against **existing schema** (Compat: no migrations unless legacy already has them).
- Emit structured logs and propagate `x-request-id`.

### Common inputs

- `services/<svc>/src/**`
- `services/<svc>/openapi/**`
- `packages/rust/madome-core/**`
- `contracts/http/**` (relevant to service)
- DB schema docs (if present) and fixtures

### Common outputs

- Service code under `services/<svc>/`
- Service unit tests under `services/<svc>/tests/**`
- Contract tests or golden updates under `contracts/http/<svc>/**`
- OpenAPI specs under `services/<svc>/openapi/**`

### Common prohibitions

- MUST NOT change external routes, payload schemas, or status codes in Compat.
- MUST NOT introduce new DB schema in Compat (unless reproducing existing legacy tables exactly).
- MUST NOT rely on client-provided `x-madome-*` headers (Gateway overwrites them).
- MUST NOT embed environment secrets in code or config.

### Common test responsibilities

- Unit tests for domain logic and edge cases.
- Integration tests using fixture Postgres where applicable.
- Contract tests for every public endpoint (hurl assertions).

### Compat scope (all services)

- Implement endpoints with legacy semantics.
- Keep existing auth verification logic (dual enforcement with Gateway).
- Return identical error models (status + body).

### Stabilize scope (all services)

- Remove direct token verification; trust gateway identity headers.
- Replace internal HTTP calls with gRPC clients.
- Introduce idempotency and retries where required (no public contract change).

### vNext scope (all services)

- Add additive endpoints or optional fields behind feature flags.
- Keep legacy endpoints intact and contract-tested.

---

## 3. users Service Agent

### Responsibilities (service-specific)

- Implement user identity/profile endpoints, user settings, and any user-scoped resources.
- Ensure compatibility with legacy pagination, filtering, and sorting semantics.

### Inputs

- `services/users/**`
- `contracts/http/users/**`
- DB tables used by users domain (existing)

### Outputs

- `services/users/src/**`
- `services/users/openapi/**`
- `contracts/http/users/**`

### Prohibitions

- MUST NOT re-shape user JSON payloads (field names/types fixed).
- MUST NOT change error status code mapping.

### Test ownership

- Golden tests for:
  - unauthenticated vs authenticated behavior
  - user not found / forbidden / validation errors
  - pagination edge cases

---

## 4. library Service Agent

### Responsibilities

- Implement library/catalog endpoints (whatever legacy defines as “library”).
- Maintain consistent caching headers if they exist in legacy.

### Inputs/Outputs

- `services/library/**`
- `contracts/http/library/**`
- DB tables used by library domain (existing)

### Prohibitions

- MUST NOT change caching semantics if clients depend on them.

### Test ownership

- Contract tests for list/detail endpoints and cache-related headers.

---

## 5. auth Service Agent

### Responsibilities

- Reimplement legacy auth endpoints:
  - login
  - refresh
  - logout
  - session checks (if present)
- Ensure cookie attributes are identical to legacy per environment.

### Inputs

- `services/auth/**`
- `contracts/cookies/dev.yaml`, `contracts/cookies/prod.yaml`
- `contracts/http/auth/**`

### Outputs

- `services/auth/src/**`
- `services/auth/tests/**`
- Cookie contract tests updates

### Prohibitions

- MUST NOT change token validation rules:
  - refresh validates access + refresh together
- MUST NOT change cookie names or attributes.

### Test ownership

- Cookie tests:
  - byte-for-byte `Set-Cookie` equality
- Refresh tests:
  - missing access token → expected legacy status
  - missing refresh token → expected legacy status
  - mismatched pair → expected legacy status
  - valid pair → new pair issued with exact cookie attributes

---

## 6. image Service Agent

### Responsibilities

- **Prod:** serve images from local disk (3TB store) without changing URL shape.
- **Dev:** provide deterministic stub server behavior (generation + caching + Range 206).

### Inputs

- `services/image/**`
- `contracts/http/image/**`
- (Prod) file store mount conventions (documented in `services/image/README.md`)

### Outputs

- `services/image/src/**`
- `services/image/README.md`
- Contract tests for Range, content-type, caching headers

### Prohibitions

- MUST NOT change public URL paths or file naming semantics.
- Stub mode MUST NOT ship to prod builds.

### Test ownership

- Range tests:
  - `Range: bytes=0-99` returns 206 with correct headers
  - invalid range returns 416
- Format tests:
  - correct `Content-Type` per extension/negotiation

---

## 7. Gateway Agent

### Responsibilities

- Implement Gateway service under `services/gateway/` with strict Compat guardrails:
  - centralized authN/authZ
  - routing without path rewrite
  - cookie pass-through (including `Set-Cookie`)
  - spoofing prevention
  - internal route blocking
  - request-id generation/propagation
  - structured logs + baseline metrics

### Inputs

- `services/gateway/**`
- `contracts/http/**` (gateway-related)
- Routing config files:
  - `services/gateway/config/routes.dev.yaml`
  - `services/gateway/config/routes.prod.yaml`

### Outputs

- Gateway code
- Routing config
- Gateway-focused contract tests

### Prohibitions

- MUST NOT transform response bodies.
- MUST NOT modify cookies or `Set-Cookie`.
- MUST NOT change public paths.

### Test ownership

- Proxy correctness tests:
  - header pass-through except reserved headers
  - body byte-for-byte pass-through for representative endpoints
- Spoofing tests:
  - inbound `x-madome-*` dropped/overwritten
- Internal path blocking tests

### Phase scope

- **Compat:** reverse proxy + dual enforcement (gateway blocks, services still verify)
- **Stabilize:** single enforcement (services trust gateway identity headers)
- **vNext:** advanced policies (rate limits, canary routing) only if contract-safe

---

## 8. Web Agent

### Responsibilities

- Maintain SPA under `apps/web/` with routing parity.
- Ensure build-time public config injection (no runtime secret fetching).
- Ensure deploy shape matches environments:
  - prod served via CloudFront under `domain.com`
  - dev may be served from k8s under `dev-domain.com`

### Inputs

- `apps/web/**`
- `packages/ts/madome-public-config/**`
- Contract expectations for routes (playwright tests)

### Outputs

- `apps/web/src/**`
- Build pipeline config (`apps/web/vite.config.ts` or equivalent)
- Playwright route compatibility tests under `tools/contract-tests/playwright/**`

### Prohibitions

- MUST NOT change route paths or SPA URL semantics.
- MUST NOT embed secrets in build outputs.
- MUST NOT change API base URL shape (host split must remain).

### Test ownership

- Playwright smoke tests:
  - direct navigation to deep routes returns SPA entrypoint
  - refresh on nested route works
  - static asset caching behavior matches legacy (if contract-captured)

### Phase scope

- **Compat:** parity + deploy correctness
- **Stabilize:** performance, bundle size, DX improvements (no routing changes)
- **vNext:** new UI features behind feature flags

---

## 9. Infra Agent (k8s + Husako + SOPS)

### Responsibilities

- Own `k8s/` GitOps structure:
  - Flux objects
  - Kustomize bases/overlays
  - SOPS-encrypted secrets
- Enforce “no Helm” in repo.
- Introduce Husako **after Compat** for prod; dev optional.

### Inputs

- `k8s/**`
- Service deployment needs (ports, env vars, probes)
- Secrets requirements (names and keys only; values provided via SOPS files)

### Outputs

- `k8s/flux/**`
- `k8s/base/**`
- `k8s/clusters/dev/**`
- `k8s/clusters/prod/**`
- `k8s/sops/**` (encrypted)
- `k8s/husako/**` (templates and/or generated output)

### Prohibitions

- MUST NOT store plaintext secrets.
- MUST NOT use Helm charts.
- MUST NOT apply changes directly to clusters (PR-only).

### Test ownership

- Kustomize build validation in CI
- Flux dry-run checks (where possible)
- SOPS decryption checks in CI (without leaking secret values)

### Phase scope

- **Compat:** Kustomize-first GitOps; stable deployments in dev/prod
- **Stabilize:** network policies, gRPC service discovery, security hardening
- **vNext:** multi-cluster patterns if needed

---

## 10. OpenAPI Agent (aggregate static deployment)

### Responsibilities

- Own OpenAPI aggregation tool and output artifacts.
- Maintain linting rules and ensure public/internal separation.

### Inputs

- `services/*/openapi/public.yaml`
- `services/*/openapi/internal.yaml`
- `tools/openapi-aggregate/**`

### Outputs

- `contracts/openapi/public.yaml` (frozen in Compat)
- `dist/openapi/**` (generated)
- CI steps for validation and artifact publication

### Prohibitions

- MUST NOT publish internal specs publicly.
- MUST NOT change public spec in Compat unless it is correcting a mismatch with legacy (requires Leader approval).

### Test ownership

- Spectral lint
- OpenAPI merge determinism
- “Spec matches contract tests” sanity checks (spot validation)

### Phase scope

- **Compat:** lock public spec to legacy behavior
- **Stabilize:** add internal gRPC docs (separate from public OpenAPI)
- **vNext:** versioned spec outputs for new endpoints

---

## 11. Sync Agent (External EC2 + IP allowlist + HMAC)

### Responsibilities

- Implement `services/sync/` endpoints under a dedicated internal path.
- Enforce IP allowlist and HMAC verification.
- Ensure endpoints are not mixed with public API surface and are blocked for non-allowlisted IPs.

### Inputs

- `services/sync/**`
- Gateway routing rules for sync paths
- `contracts/http/sync/**` (internal contract tests)
- Allowed IP list source (Git-managed config in `k8s/` or gateway config)

### Outputs

- Sync service code
- HMAC verification library code (may live in `packages/rust/madome-core`)
- Contract tests for signature validation

### Prohibitions

- MUST NOT expose sync endpoints on public OpenAPI.
- MUST NOT accept unsigned or stale requests.
- MUST NOT allow header spoofing to bypass HMAC checks.

### Test ownership

- HMAC tests:
  - correct signature accepted
  - incorrect signature rejected
  - replay (same nonce) rejected
  - stale timestamp rejected
- IP allowlist tests (gateway-level + service-level defense in depth)

### Phase scope

- **Compat:** implement allowlist + HMAC + dedicated route
- **Stabilize:** improve replay protection storage and observability
- **vNext:** expand capabilities only if still internal and controlled

---
