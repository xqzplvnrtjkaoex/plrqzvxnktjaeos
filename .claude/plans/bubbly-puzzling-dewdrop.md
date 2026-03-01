# Plan: Compat Phase — Full Implementation Sequence (v2)

---

## Unit A Execution Plan (current focus)

### Starting state (verified)
- `rust-toolchain.toml` exists (channel = "stable")
- `.github/` exists but empty
- Everything else (Cargo.toml, crates/, packages/, services/, tools/, contracts/) needs to be created

### Step 0 — pre-flight (once, before any dev commits)

```bash
# 1. Rename plan files (replaces placeholders, see "Plan file step 0" note in Context below)
cp .claude/plans/deep-hugging-haven.md .claude/plans/bubbly-puzzling-dewdrop.md
# Verify no actual hostnames leaked in, then:
rm .claude/plans/deep-hugging-haven.md

# 2. Open 10 GitHub Issues (A–J) — commands in "GitHub Issues to Open" section below

# 3. Create and switch to dev branch
git checkout -b dev
```

### Implementation sequence (all commits on `dev`)

**Commit 1 — Root Cargo.toml**
- `Cargo.toml` workspace (members: `crates/*`, `packages/proto`, `tools/*`; services/ added per-unit)
- `[workspace.package]` with edition = "2021", license, rust-version

**Commit 2 — `crates/madome-domain`**
- `src/id.rs`: `UserId(Uuid)`, `BookId(u32)`, `AuthcodeId(Uuid)` (newtypes with derive Display, FromStr via uuid/std)
- `src/user.rs`: `UserRole` enum — `Normal=0 | Developer=1 | Bot=2` (u8 wire, PartialOrd+Ord, `from_u8() → Option<Self>`)
- `src/book.rs`: `BookKind` (kebab serde), `BookSortBy` (custom `Deserialize` matching strings), `SearchBookSortBy` (custom `Deserialize`)
- `src/book_tag.rs`: `BookTagKind` (kebab, Display, FromStr, `UnknownTagKind` error)
- `src/activity.rs`: `TasteKind`, `HistoryKind`, `NotificationKind` (snake_case serde)
- `src/pagination.rs`: `Sort { Desc, Asc }`, `PageRequest { per_page: u32, page: u32 }` (defaults 25/1, bounds 1–100/≥1)

  Tests (`#[cfg(test)]` inline):
  - Serde round-trip for every enum in both directions
  - `BookSortBy`: `"id-desc"` → `Id(Sort::Desc)`, `"random"` → `Random`, all 9 variants
  - `SearchBookSortBy`: `"rank-desc"` → `RankDesc`, all 3 variants
  - `UserRole`: `Normal < Developer < Bot` (PartialOrd)
  - `PageRequest`: default values; `per_page` clamped to 1–100

**Commit 3 — `crates/madome-auth-types`**
- `TokenInfo { user_id: Uuid, user_role: u8, access_token_exp: u64 }`
- Constants: `MADOME_ACCESS_TOKEN`, `MADOME_REFRESH_TOKEN`, `ACCESS_TOKEN_EXP = 14400`, `REFRESH_TOKEN_EXP = 604800`
- Cookie builders (doc tests required, use `"example.com"` as dummy domain):
  - `set_access_token_cookie(jar, value, domain)` → Path=/, Max-Age=604800, HttpOnly, Secure, SameSite=Lax
  - `set_refresh_token_cookie(jar, value, domain)` → Path=/auth/token, Max-Age=604800, HttpOnly, Secure, SameSite=Lax
  - `clear_cookies(jar, domain)` → Max-Age=0 for both
- `validate_access_token(cookie_value: &str, secret: &Secret) → Result<TokenInfo, AuthError>` (pure fn, no Tower dep)
- `IdentityHeaders` axum `FromRequestParts`: reads `x-madome-user-id` + `x-madome-user-role`; 401 if absent/unparseable

  Tests (inline + doc tests):
  - Doc tests for all three cookie builders (attribute-by-attribute assertions)
  - `validate_access_token`: valid JWT → TokenInfo; expired → Err; invalid sig → Err
  - `IdentityHeaders`: missing header → 401; valid headers → Ok

**Commit 4 — `crates/madome-core`**
- `AppError` enum + `IntoResponse` (Unauthorized→401, Forbidden→403, NotFound→404, Conflict→409, Internal→500)
- `init_tracing()` — structured stdout logs (tracing-subscriber JSON)
- `Config::from_env()` using `std::env::var()` — fail-fast on missing env vars
- Health check handlers: `GET /healthz`, `GET /readyz`
- Request ID middleware — propagates `x-request-id`

  Tests (inline):
  - `AppError` → correct HTTP status code for each variant
  - `init_tracing()` doesn't panic

**Commit 5 — `packages/proto/`**
- `user.proto`: `UserService { GetUser, GetTastes, RenewBook }` + `NotificationService { CreateNotification }`
- `library.proto`: `LibraryService { RenewBook }`
- `notification.proto`: notification message types
- `build.rs`: `tonic_build::compile_protos(...)` for all three
- `Cargo.toml`: `tonic`, `prost`, `tokio` (build-dep: `tonic-build`, `prost-build`)

**Commit 6 — `crates/madome-testing`**
- `MockAuthServer` — configurable `TokenInfo`; injects `x-madome-user-id` + `x-madome-user-role` into test requests
- `MockGrpcServer` — in-process tonic server with configurable responses (skeleton; expanded per-service in later units)
- Fixture loader: `Fixture::load("contracts/http/auth/create_token.json")` — parses golden file format
- `TestApp` wrapper for `axum_test::TestServer`
- `TestDb` — stub for now (deferred to Unit C when sea-orm migrations exist); marks compile-time placeholder

**Commit 7 — CI (workflows/ + gaji)**
- Research gaji npm package before implementing (check https://www.npmjs.com/package/gaji or GitHub)
- `workflows/package.json` + `workflows/src/ci.ts` — TypeScript source defining fmt/clippy/test jobs
- Run `gaji build` → `.github/workflows/ci.yml`
- Jobs: `fmt` (cargo fmt --all --check), `clippy` (-D warnings), `test` (cargo test --workspace --all-features)
- Triggers: push/PR on both `master` and `dev`

**Commit 8 — Docs**
- `README.md`: monorepo structure, quickstart, dev setup
- `contracts/README.md`: placeholder (full content in Unit B)

### Gates before master PR

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
# CI green on open PR
```

### Master PR

```bash
git push origin dev
gh pr create \
  --base master --head dev \
  --title "workspace: Cargo skeleton, shared crates, CI" \
  --body "closes #1  ..."
```

---

## Context

This is a full rewrite of `bubbly-puzzling-dewdrop.md`. The external interface map (Part 1)
and architecture design (Part 2) are preserved verbatim from v1 — all data was verified
against legacy source code. What changed:

- **New GitHub workflow** (master+dev branches, GitHub Issues per PR unit, CI via gaji)
- **Consolidated PR units** — original 19 PRs merged into 10 master PR units; WIP commits
  flow on `dev`, master PRs open only when a unit is fully tested and gate-complete
- **Current workspace state** — nothing built yet: no Cargo.toml, no services/, crates/

Part 5 from v1 (CLAUDE.md updates) is complete and no longer included.

**`previous/` usage rule**: `previous/` may be read to verify external contracts
(API response schemas, cookie attributes, status codes, behavioral rules like renewal call order).
However, the new implementation's internal structure is designed fresh following Part 2's
architecture — do not mirror or copy legacy internal patterns. The goal is an improved system,
not a rewrite of the same structure.

**Domain names**: actual hostnames (`{API_HOST}`, `{FILE_HOST}`, root cookie domain) must not
appear in committed files. Use `{API_HOST}` / `{FILE_HOST}` as placeholders in docs; actual
values live only in k8s SOPS secrets and runtime env vars. In doc-test examples, use
`"example.com"` as a dummy domain.

**Plan file step 0** (first action on implementation): overwrite `bubbly-puzzling-dewdrop.md`
with this file's content (which uses placeholders), then delete `deep-hugging-haven.md`.

---

## GitHub Issues to Open Before Implementation

Claude opens one issue per master PR unit before any implementation begins.
PRs reference issues with `closes #N` in the PR body.

```bash
gh issue create --title "workspace: Cargo skeleton, shared crates, CI" \
  --body "Unit A: Cargo.toml workspace, crates/madome-{domain,auth-types,core,testing}, packages/proto/, gaji CI workflows."

gh issue create --title "contracts: fixture files + contract-harness" \
  --body "Unit B: contracts/ directory with all fixture files (all fail). tools/contract-harness Rust binary."

gh issue create --title "auth service: HTTP handlers, passkeys, WebAuthn" \
  --body "Unit C: services/auth — full hexagonal stack. Auth contracts pass."

gh issue create --title "users service: @me routes, tastes, histories, gRPC" \
  --body "Unit D: services/users — @me CRUD, tastes, histories, notifications, FCM token. gRPC UserService + NotificationService."

gh issue create --title "library service: /books routes, search, tags, OG" \
  --body "Unit E: services/library — full /books API, Meilisearch, tag search, open-graph. gRPC LibraryService."

gh issue create --title "image + sync services" \
  --body "Unit F: services/image (file serving, Range, PUT, path completion) + tools/image-gen + services/sync (HMAC, SSE)."

gh issue create --title "worker service: outbox processor (FCM, email, renewal)" \
  --body "Unit G: services/worker — outbox pattern, LISTEN/NOTIFY, FCM push, email authcodes, book renewal."

gh issue create --title "gateway service: JWT enrichment, pingora proxy" \
  --body "Unit H: services/gateway — pingora, JWT validation, header stripping/injection, upstream routing."

gh issue create --title "frontend + GitOps: Vue SPA, k8s manifests, SOPS secrets" \
  --body "Unit I: apps/web Vue 3 SPA scaffold + k8s/ base/overlays/sops + FluxCD."

gh issue create --title "e2e smoke tests + rollback rehearsal" \
  --body "Unit J: tools/e2e-smoke Rust binary (full-stack integration vs. dev env). Rollback rehearsal. Compat exit criteria."
```

---

## Part 1: External Interface Map (Source of Truth)

### 1.1 Cookie Contract — MOST CRITICAL

Both cookies use **Max-Age = 604800 (7 days)** even though the access JWT expires in 4 hours.
Cookie lifetime ≠ JWT lifetime. This is intentional — the browser keeps the cookie alive
so the client can detect expiry via the header and proactively refresh.

| Cookie                 | Path          | Domain      | SameSite | HttpOnly | Secure | Max-Age |
| ---------------------- | ------------- | ----------- | -------- | -------- | ------ | ------- |
| `madome_access_token`  | `/`           | root domain | Lax      | ✓        | ✓      | 604800  |
| `madome_refresh_token` | `/auth/token` | root domain | Lax      | ✓        | ✓      | 604800  |

- **Prod domain**: root domain shared across api + file subdomains (value in k8s SOPS secrets)
- `DELETE /auth/token` clears both cookies with `Max-Age=0`
- `POST/PATCH /auth/token` set both cookies + respond with `x-madome-access-token-expires` header

Constants (from `previous/sdk` + `previous/auth-madome-app`):

```rust
// previous/auth-madome-app/src/entity/token.rs:
pub const ACCESS_TOKEN_EXP: i64  = 3600 * 4;         // 4 h  — JWT exp claim
pub const REFRESH_TOKEN_EXP: i64 = 3600 * 24 * 7;    // 7 d  — cookie Max-Age (BOTH cookies)
// previous/sdk/src/auth.rs:
pub const MADOME_ACCESS_TOKEN:  &str = "madome_access_token";
pub const MADOME_REFRESH_TOKEN: &str = "madome_refresh_token";
```

Token refresh logic:

1. Read both cookies from request jar
2. Validate `refresh_token` JWT signature + exp
3. Ignore `access_token` exp (access token may be expired — refresh handles it)
4. Issue new JWT pair; set both cookies again with Max-Age=604800

---

### 1.2 API Routes

#### Auth Service — `{API_HOST}/auth → auth:3112`

| Method | Path                             | Auth    | Status          | Description                                      |
| ------ | -------------------------------- | ------- | --------------- | ------------------------------------------------ |
| POST   | `/auth/code`                     | No      | 201/404/429     | Create authcode                                  |
| GET    | `/auth/token`                    | access  | 200/401/403     | Check token; query: `role?`                      |
| POST   | `/auth/token`                    | No      | 201/404         | Login → sets both cookies                        |
| PATCH  | `/auth/token`                    | refresh | 201/401         | Refresh → new cookies                            |
| DELETE | `/auth/token`                    | access  | 204/401         | Logout → clears cookies                          |
| GET    | `/auth/passkeys`                 | access  | 200/401         | List passkeys                                    |
| DELETE | `/auth/passkeys/{credential_id}` | access  | 204/400/401/404 | Delete passkey                                   |
| POST   | `/auth/passkey/registration`     | access  | 200/401         | Start WebAuthn registration                      |
| PATCH  | `/auth/passkey/registration`     | access  | 201/401         | Finish registration; query: `registration-id`    |
| POST   | `/auth/passkey/authentication`   | No      | 200/401/404     | Start WebAuthn auth; query: `email`              |
| PATCH  | `/auth/passkey/authentication`   | No      | 201/401         | Finish auth; query: `authentication-id`, `email` |

**Request bodies:**

```
POST /auth/code              { email: String }
POST /auth/token             { email: String, code: String (12 chars) }
PATCH /auth/passkey/registration  body: RegisterPublicKeyCredential (WebAuthn)
PATCH /auth/passkey/authentication body: PublicKeyCredential (WebAuthn)
```

**Response headers (on token issue/refresh/check):**

```
x-madome-access-token-expires: <epoch_seconds: u64>
```

**GET /auth/token response body:**

```json
{ "user_id": "<uuid>", "user_role": 0|1|2, "access_token_exp": <u64> }
```

---

#### Library Service — `{API_HOST}/books → library:3112`

Auth middleware applied to **all** `/books` routes (401 if missing). `/open-graph/*` separate router with no middleware.

| Method | Path                                      | Auth        | Status       | Description                    |
| ------ | ----------------------------------------- | ----------- | ------------ | ------------------------------ |
| GET    | `/books`                                  | access      | 200          | List books                     |
| GET    | `/books` (with `ids[]`)                   | access      | 200          | Get books by IDs               |
| GET    | `/books` (with `tags[]`)                  | access      | 200          | Get books by tags              |
| GET    | `/books/{book_id}`                        | access      | 200/404      | Get book by ID                 |
| GET    | `/books/search`                           | access      | 200          | Full-text search (Meilisearch) |
| GET    | `/books/tags/search`                      | access      | 200          | Search tags                    |
| GET    | `/books/tags/{tag}`                       | access      | 200/404      | Get tag; `{kind}-{name}`; hyphen fallback (deprecated) |
| GET    | `/books/currently-renewing`               | access      | 200          | All books being renewed        |
| GET    | `/books/{existing_id}/currently-renewing` | access      | 204/404      | Check if renewing              |
| GET    | `/books/need-check`                       | role ≥ 1    | 200          | Books needing check            |
| GET    | `/books/ids`                              | role ≥ 1    | 200          | Book IDs by tags               |
| POST   | `/books`                                  | role ≥ 2    | 201/409      | Add book                       |
| PATCH  | `/books`                                  | role ≥ 2    | 200/409      | Update book                    |
| PATCH  | `/books/{book_id}/release`                | role ≥ 2    | 204/404      | Release book                   |
| PATCH  | `/books/{book_id}/check`                  | role ≥ 2    | 204          | Update checked_at              |
| POST   | `/books/{existing_id}/renew/{new_id}`     | role ≥ 2    | 204/404      | Prepare renewal                |
| GET    | `/open-graph/books/{book_id}`             | **No auth** | 200 HTML/404 | OG metadata                    |
| GET    | `/open-graph/books/tags/{kind}/{name}`    | **No auth** | 200 HTML/404 | OG tag metadata                |

**Notable query params for GET /books:**

- `per-page` (1-100, default 25), `page` (default 1)
- `sort-by`: `id-desc` (default), `id-asc`, `published-at-desc`, `published-at-asc`, `checked-at-desc`, `checked-at-asc`, `updated-at-desc`, `updated-at-asc`, `random`
- `kinds[]`, `released`, `deduplicate`, `legacy-only`, `exclude-dislikes`
- Dispatch: if `ids[]` present → get by IDs; if `tags[]` present → get by tags

Deprecated params (Compat: still supported, `/// deprecated` doc comment on field):
- `kind` (singular): single `BookKind` filter — replaced by `kinds[]` (deprecated 20240120)

`legacy-only` (bool, default `false`): when `true`, returns only books with `legacy=true`
(superseded by renewal). Disables deduplication. When `false`, `legacy=true` books excluded from listings.

**GET /books/tags/{tag} backwards-compat**: hyphen fallback — if lookup fails, retries with
hyphens converted to spaces. Log with `tracing::warn!` + `// deprecated` comment. Keep for Compat.

**Book response schema**: includes `legacy: bool` field.

**GET /books/search query:** `q`, `filter`, `kinds[]`, `per-page`, `page`, `sort-by` (`rank-desc` (default), `id-desc`, `id-asc`), `completion`, `exclude-dislikes`

---

#### Users Service — `{API_HOST}/users → users:3112`

| Method | Path                                    | Auth     | Status      | Description                      |
| ------ | --------------------------------------- | -------- | ----------- | -------------------------------- |
| POST   | `/users`                                | role ≥ 2 | 201/409     | Create user account (admin only) |
| GET    | `/users/@me`                            | access   | 200/404     | Get current user                 |
| PATCH  | `/users/@me`                            | access   | 204/409     | Update name/handle               |
| GET    | `/users/@me/tastes`                     | access   | 200         | List tastes                      |
| GET    | `/users/@me/tastes` (with `book-ids[]`) | access   | 200         | Tastes by book IDs               |
| GET    | `/users/@me/tastes/{kind}/{value}`      | access   | 200/404     | Get single taste                 |
| POST   | `/users/@me/tastes`                     | access   | 201/404/409 | Add/update taste                 |
| DELETE | `/users/@me/tastes`                     | access   | 204/404     | Delete taste (JSON body)         |
| GET    | `/users/@me/histories`                  | access   | 200         | List histories                   |
| GET    | `/users/@me/histories/{kind}/{value}`   | access   | 200/404     | Get single history               |
| POST   | `/users/@me/histories`                  | access   | 201         | Create/update history            |
| DELETE | `/users/@me/histories`                  | access   | 204/404     | Delete history (JSON body)       |
| GET    | `/users/@me/notifications`              | access   | 200         | List notifications               |
| POST   | `/users/@me/fcm-token`                  | access   | 201         | Create/update FCM token          |

**Note**: Path uses `@me` (not `me`). DELETE taste/history take JSON body, not path params.

**POST /users body:** `{ name, handle, email, role?: 0|1 }`

**Taste types** (`#[serde(tag = "kind", rename_all = "snake_case")]`):

```
// Response
Taste::Book    { book_id: u32, is_dislike: bool, created_at: RFC3339ms }
Taste::BookTag { tag_kind: String, tag_name: String, is_dislike: bool, created_at: RFC3339ms }

// Extended (list response)
ExtendTaste::Book    { ...Taste::Book..., book?: Book }
ExtendTaste::BookTag { ...Taste::BookTag..., books: Vec<Book> }

// Request (POST)
kind: "book"     → { book_id: u32 (≥1), is_dislike: bool }
kind: "book_tag" → { tag_kind: String, tag_name: String, is_dislike: bool }

// Request (DELETE)
kind: "book"     → { book_id: u32 (≥1) }
kind: "book_tag" → { tag_kind: String, tag_name: String }
```

**History types** (`#[serde(tag = "kind", rename_all = "snake_case")]`):

```
// Response
History::Book    { book_id: u32, page: usize, created_at: RFC3339ms, updated_at: RFC3339ms }
ExtendHistory::Book { ...History::Book..., book?: Book }

// Request (POST)
kind: "book" → { book_id: u32 (≥1), page: usize (≥1) }

// Request (DELETE)
kind: "book" → { book_id: u32 (≥1) }
```

**Notification type**:

```
Notification::Book { book_id: u32, book_tags: Vec<(String, String)>, created_at: RFC3339ms }
```

**Kind enums** (kebab-case in query params): `TasteKind`: `book`, `book-tag`; `HistoryKind`: `book`; `NotificationKind`: `book`

---

#### Image Service — `{FILE_HOST} → image:3112`

Auth middleware applied globally. PUT additionally requires role ≥ 2.

| Method | Path       | Auth     | Status          | Description                                 |
| ------ | ---------- | -------- | --------------- | ------------------------------------------- |
| GET    | `/*path`   | access   | 200/206/403/404 | Read file (with path completion + Range)    |
| GET    | `/*path/@` | access   | 200/403/404     | List directory → `Vec<String>` (JSON)       |
| PUT    | `/*path`   | role ≥ 2 | 204/400/403     | Upload file (multipart/form-data, max 20MB) |

**Path completion**: if exact file not found, lists parent directory and returns highest-match filename starting with keyword.

**Range requests**: `tower_http::ServeFile` handles → 206 + `Content-Range`, `Accept-Ranges: bytes`.

**Directory listing** (`/*path/@`): `Vec<String>` sorted by char count then alphabetically. Excludes `image_list`, `image_list.txt`. Excludes `thumbnail` unless `?visible-thumbnail=true`.

**Dev image stub** (`tools/image-gen/`): Rust binary. `seed = SHA256(path)` → deterministic solid-color PNG/AVIF.
Run once during dev setup; output directory is volume-mounted into the image service pod in k8s dev overlay.
Image service serves stub files normally — no special dev mode needed.

---

#### Sync Service — internal only (not in OpenAPI)

- HMAC-authenticated endpoints for EC2 sync operations
- HMAC: timestamp + nonce + method + path + body_hash
- Nonce replay prevention (Redis TTL)
- IP allowlist enforced at middleware
- SSE stream for sync events

---

### 1.3 Internal Communication

`/_/` HTTP routes from legacy are **not external contracts** and will not be replicated.
Internal service-to-service communication is replaced with **gRPC (tonic)** from day 1.

Proto definitions live in `packages/proto/`.

**gRPC services replacing legacy `/_/` calls:**

| Proto service                      | RPC                                                     | Replaces                         |
| ---------------------------------- | ------------------------------------------------------- | -------------------------------- |
| `user.UserService`                 | `GetUser(GetUserRequest) → User`                        | `GET /_/users/:id`               |
| `user.UserService`                 | `GetTastes(GetTastesRequest) → TasteList`               | `GET /_/users/:id/likes\|dislikes` |
| `user.UserService`                 | `RenewBook(RenewBookRequest) → Empty`                   | `PATCH /_/users/books/:old/:new` |
| `notification.NotificationService` | `CreateNotification(CreateNotificationRequest) → Empty` | `POST /_/users/notifications`    |
| `library.LibraryService`           | `RenewBook(RenewBookRequest) → Empty`                   | `PATCH /_/books/:old/:new`       |

---

### 1.4 SPA Routes — All must return 200 from nginx

```
/
/books
/books/:bookId
/books/:bookId/reader
/books/tags/:tagKind/:tagName
/tastes
/histories
/search/books
/settings
/profile
/profile/edit
/profile/passkey
```

Auth: No per-route guards. 401 → app-level conditional `<SignIn>` component.

---

### 1.5 Ingress Rules

```
nginx → {API_HOST}/auth     →  gateway (port 3113) → auth svc (port 3112)
nginx → {API_HOST}/users    →  gateway (port 3113) → users svc (port 3112)
nginx → {API_HOST}/books    →  gateway (port 3113) → library svc (port 3112)
nginx → {API_HOST}/openapi/ →  static files (nginx direct — no gateway)
nginx → {FILE_HOST}/        →  gateway (port 3113) → image svc (port 3112)
```

gRPC ports are cluster-internal only — not exposed through nginx ingress.

**OpenAPI generation** (code-first, replaces legacy `previous/madome-openapi/` service):
- Legacy had a running `previous/madome-openapi/` service aggregating specs at runtime.
- New design: build-time `tools/openapi/` binary → `dist/openapi/public.yaml`, served statically by nginx. No separate service needed.
- Each service annotates handlers with `#[utoipa::path(...)]`
- `tools/openapi/` merges per-service specs → `dist/openapi/public.yaml`
- nginx serves `dist/openapi/` at `{API_HOST}/openapi/`
- `contracts/openapi/public.yaml` = frozen Compat spec (CI assertion)

**k8s network policy**: services only accept traffic from gateway. Direct external access blocked.

---

### 1.6 Internal Headers

| Header                               | Direction          | Description                      |
| ------------------------------------ | ------------------ | -------------------------------- |
| `x-madome-access-token-expires`      | auth → client      | Access token expiry (epoch secs) |
| `x-madome-passkey-registration-id`   | auth → client      | WebAuthn registration session ID |
| `x-madome-passkey-authentication-id` | auth → client      | WebAuthn auth session ID         |
| `x-madome-user-id`                   | gateway → service  | Authenticated user UUID (injected after JWT validation) |
| `x-madome-user-role`                 | gateway → service  | Authenticated user role as u8 (0/1/2)                  |

---

## Part 2: Architecture Design

### 2.1 Design Principles

1. **Compat-first**: External behavior byte-for-byte identical to legacy.
2. **Hexagonal layers**: `handlers → usecase → domain → infra`. Domain has zero framework imports.
3. **Fine-grained shared crates**: Replace monolithic `previous/sdk/` with focused crates by concern.
4. **Cookie building centralized**: One place sets cookie attributes — eliminates drift.
5. **Contract-first**: Fixture files land and fail CI before any service implementation PR.
6. **Observability from day 1**: Structured tracing + logs in every service.
7. **gRPC from day 1**: Internal service calls via tonic — no `/_/` HTTP routes.
8. **Gateway as single JWT enrichment point**: Validates cookie → injects identity headers → services trust injected headers (k8s NetworkPolicy prevents spoofing).

---

### 2.2 Monorepo Layout

```
apps/web/                 # Vue 3 SPA (from madome-vue)
services/
  gateway/                # JWT enrichment + proxy (pingora)
  auth/                   # {API_HOST}/auth
  users/                  # {API_HOST}/users
  library/                # {API_HOST}/books
  image/                  # {FILE_HOST}
  sync/                   # internal EC2 sync
  worker/                 # outbox processor
crates/
  madome-core/
  madome-auth-types/
  madome-domain/
  madome-testing/
packages/proto/           # protobuf; tonic-build generates stubs
tools/
  contract-harness/       # Rust bin: contract assertions
  e2e-smoke/              # Rust bin: full-stack integration
  openapi/                # merges utoipa specs → dist/openapi/public.yaml
  image-gen/              # Rust bin: dev stub images (run once)
contracts/
  http/                   # per-endpoint golden files
  cookies/dev.txt, prod.txt
  spa-routing/routes.txt
  openapi/public.yaml     # frozen Compat OpenAPI spec (CI assertion)
k8s/base/, overlays/{dev,prod}/, sops/
docs/                     # created as needed: multi-service topology, operational runbooks
Cargo.toml                # workspace root
```

---

### 2.3 Service Internal Structure (Hexagonal)

Every Rust service follows:

```
services/{name}/
  Cargo.toml
  schema/Cargo.toml + src/{entity}.rs      # one file per table (DeriveEntityModel)
  migration/Cargo.toml + src/main.rs       # sea-orm-migration binary
  src/
    main.rs, lib.rs, config.rs, router.rs
    grpc_server.rs     # omit if no gRPC server
    error.rs           # AppError: IntoResponse
    handlers/          # HTTP only: extract → usecase → response
    usecase/           # Business logic (no axum, sea-orm, reqwest)
    domain/            # Pure types + repository traits
    infra/db.rs        # sea-orm impl; converts Model → domain type
    infra/cache.rs     # Redis
    infra/grpc.rs      # tonic clients for OTHER services
  tests/integration/   # axum_test::TestServer tests
```

**sea-orm entity pattern** (one file per table in `schema/src/`):

```rust
// schema/src/book.rs
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "books")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: u32,
    pub title: String,
    // ...
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
```

**Dependency graph**: `schema/` ← `migration/` (migration references entity definitions),
`schema/` ← `src/infra/db.rs` (queries use entity models).

---

#### UseCase struct pattern

UseCase is always a struct wrapping its dependencies as fields.
Shared as `Arc<UseCase>` in axum State; swapped for mocks in tests.
Use generics (`<R: Trait>`) for static dispatch — not `Box<dyn Trait>` unless runtime selection is needed.

```rust
// Generic over trait → static dispatch, zero-cost
pub struct GetBookUseCase<R: BookRepository> { repo: R }

// Production wiring (in main.rs or router.rs)
let uc = Arc::new(GetBookUseCase { repo: DbBookRepository { db: pool.clone() } });

// Test wiring
let uc = GetBookUseCase { repo: MockBookRepo::returning(fake_book) };
```

Use `Box<dyn Trait>` only when the concrete type is chosen at runtime (rare).

#### Input / Output models

Each non-trivial use case defines its own I/O types in the same file:

```rust
pub struct GetBookInput  { pub book_id: u32, pub requester_role: UserRole }
pub struct GetBookOutput { pub book: Book }
```

- `XxxInput`: validated inputs. No axum types (`Path`, `Query`, etc.).
- `XxxOutput`: result shape. No sea-orm types. Handler converts to HTTP response model.
- Handler only sees I/O types — not domain internals it doesn't need.

**When I/O models are required:**
- Use case has validation logic
- Use case touches more than one repository
- Use case has side effects (writes, cache invalidation, gRPC calls)

**When I/O models may be skipped:**
- Simple single-repo read with no transformation (`// simple read: no I/O wrapper` comment)

#### Dependency rule (hard enforcement)

```
handlers/   ──→  usecase/  ←──  infra/
                    │
                    ↓
                domain/        crates/madome-*
```

| Layer       | May import from                         | Must NOT import               |
| ----------- | --------------------------------------- | ----------------------------- |
| `domain/`   | `crates/madome-domain`, std             | Everything else               |
| `usecase/`  | `domain/`, `crates/madome-*`            | axum, sea-orm, reqwest, tonic |
| `handlers/` | `usecase/` I/O types, axum extractors   | sea-orm, domain internals     |
| `infra/`    | `domain/` traits, sea-orm, tonic, redis | axum, `usecase/`              |

---

### 2.4 Shared Crates

#### `crates/madome-core`

- `AppError` + common variants: `Unauthorized`, `Forbidden`, `NotFound`, `Conflict`, `Internal`
- `init_tracing()` — OTLP/stdout structured logs
- `Config::from_env()` using `std::env::var()` — fail-fast on missing vars
- Health check handlers (`GET /healthz`, `GET /readyz`)
- Request ID middleware — propagates `x-request-id`

**Structured log schema** (all services + gateway emit these fields):

```json
{
  "timestamp": "...",
  "level": "INFO",
  "service": "library",
  "env": "prod",
  "request_id": "...",
  "route": "GET /books",
  "status": 200,
  "latency_ms": 12,
  "error_code": null,
  "msg": "..."
}
```

**MUST NOT log**: access/refresh tokens, cookies, full request/response bodies, secrets from env.
PII (emails) must be redacted.

**Baseline metrics** (all services must expose):

- request count by route + status
- latency histogram by route
- upstream error counts
- (worker only) outbox processed / succeeded / failed / retried counts

#### `crates/madome-auth-types`

- `TokenInfo { user_id: Uuid, user_role: u8, access_token_exp: u64 }`
- Constants: `MADOME_ACCESS_TOKEN`, `MADOME_REFRESH_TOKEN`, `ACCESS_TOKEN_EXP = 14400`, `REFRESH_TOKEN_EXP = 604800`
- **Cookie builders** (single source of truth):

  ```rust
  fn set_access_token_cookie(jar, value, domain) -> CookieJar   // Path=/, Max-Age=604800
  fn set_refresh_token_cookie(jar, value, domain) -> CookieJar  // Path=/auth/token, Max-Age=604800
  fn clear_cookies(jar, domain) -> CookieJar                    // Max-Age=0 for both
  ```

- **JWT validation** (pure fn — callable from pingora, no Tower dep):

  ```rust
  fn validate_access_token(cookie_value: &str, secret: &Secret) -> Result<TokenInfo, AuthError>
  ```

- `IdentityHeaders` extractor — axum `FromRequestParts`: reads `x-madome-user-id` + `x-madome-user-role` headers injected by gateway; returns 401/403 as appropriate.

  ```rust
  pub struct IdentityHeaders { pub user_id: Uuid, pub user_role: UserRole }
  // Returns 401 if x-madome-user-id absent or unparseable
  // Role enforcement (403) done by the handler after extraction
  ```

  In integration tests: `MockAuthServer` in `crates/madome-testing` injects these headers directly (no gateway in test path).

#### `crates/madome-domain`

- Newtype IDs: `UserId(Uuid)`, `BookId(u32)`, `TagId(u32)`, `AuthcodeId(Uuid)`
- `BookTagKind`: `Artist | Group | Series | Character | Female | Male | Misc` (kebab-case serde + Display + FromStr)
- `UserRole`: `Normal=0 | Developer=1 | Bot=2` (Compat names; rename post-Stabilize; `from_u8()`, PartialOrd+Ord)
- `BookKind`: `Doujinshi | Manga | GameCg | ArtistCg | ImageSet` (kebab-case serde)
- `TasteKind`: `Book | BookTag` (snake_case serde)
- `HistoryKind`: `Book` / `NotificationKind`: `Book` (snake_case serde)
- `Sort`: `Desc | Asc` — standalone enum; reusable for any sort direction context
- `BookSortBy`: uses `Sort` for direction:
  ```rust
  pub enum BookSortBy {
      Id(Sort),          // "id-desc" / "id-asc"
      PublishedAt(Sort), // "published-at-desc" / "published-at-asc"
      CheckedAt(Sort),   // "checked-at-desc" / "checked-at-asc"
      UpdatedAt(Sort),   // "updated-at-desc" / "updated-at-asc"
      Random,            // "random"
  }
  ```
  Default: `BookSortBy::Id(Sort::Desc)`.
  Wire format: `"id-desc"` / `"id-asc"` etc. — requires custom `Deserialize` (e.g., match on string).
- `SearchBookSortBy`: search-specific sort (Meilisearch relevance not available in general listing):
  ```rust
  pub enum SearchBookSortBy {
      RankDesc,       // "rank-desc" (default) — Meilisearch relevance
      Id(Sort),       // "id-desc" / "id-asc"
  }
  ```
  Default: `SearchBookSortBy::RankDesc`. Also requires custom `Deserialize`.
- `PageRequest { per_page: u32 (1-100, default 25), page: u32 (min 1, default 1) }`

#### `crates/madome-testing`

- `TestApp` — axum app with isolated TestDb + MockCache
- `TestDb` — isolated PostgreSQL schema per test; runs sea-orm migrations; tears down after
- `MockAuthServer` — configurable `TokenInfo` for auth-gated tests
- `MockGrpcServer` — in-process tonic server with configurable responses
- Fixture loader: `Fixture::load("contracts/http/auth/create_token.json")`
- `make_test_session(req: http::Request<Bytes>) -> pingora::Session` — pingora test helper: serializes request to HTTP/1.1 wire format via tokio duplex pair (~50 lines, one-time cost). Used only by gateway tests.

---

### 2.5 Gateway Design (pingora)

**Responsibility: JWT enrichment, not enforcement**

```
inbound request
  1. Strip ALL inbound x-madome-* headers     ← prevents spoofing
     ALLOWED_INBOUND_X_MADOME: &[&str] = &[]  ← extend as needed
  2. x-request-id: absent/invalid → generate UUID v4
  3. Validate madome_access_token cookie (JWT sig + exp)
     Valid → inject x-madome-user-id + x-madome-user-role
     Absent/invalid → forward without identity headers
  4. Forward to upstream (pingora handles connection pooling)
  5. Return upstream response unchanged (Set-Cookie passes through)
```

JWT validation reuses `madome_auth_types::validate_access_token()` — pure function, no Tower dependency.

**Services: how they check identity headers** (replaces per-service JWT validation):

Services use `IdentityHeaders` extractor in handlers. The headers cannot be spoofed externally
because k8s NetworkPolicy blocks direct access to service ports — only gateway can reach them.

**Gateway testing**:
- Unit: `enrich.rs` via `make_test_session()`; assert stripping, identity injection, absent-cookie passthrough
- Integration: spin up pingora server on random loopback port + mock upstream; assert forwarded headers + response passthrough including Set-Cookie

**Internal structure** (stateless — no schema/, migration/):

```
services/gateway/src/
  main.rs      # pingora::proxy::http_proxy_service setup + run
  config.rs    # upstream URL map, JWT secret
  router.rs    # upstream_peer(): (host, path-prefix) → BasicPeer
  filters/enrich.rs  # strip → validate → inject
```

---

### 2.6 Worker Design (Outbox Pattern)

**Why Outbox pattern**:
Dual-write problem — "write to DB + send FCM" = two operations that can fail independently.
Outbox solution: write event to `outbox_events` in the **same DB transaction** as the main write
→ guaranteed delivery. Worker picks up events and processes them asynchronously.

```
Library service:
  BEGIN;
    INSERT INTO books ...;
    INSERT INTO outbox_events (kind='book_added', payload=...) ...;
  COMMIT;

Worker:
  LISTEN book_added (PostgreSQL LISTEN/NOTIFY)
  OR poll outbox_events WHERE processed_at IS NULL
  → process event → mark processed_at = NOW()
```

**Current known jobs:**

| Trigger             | Source outbox              | Worker action                                        |
| ------------------- | -------------------------- | ---------------------------------------------------- |
| Book added/released | library DB `outbox_events` | CreateNotification gRPC → FCM push                   |
| Authcode created    | auth DB `outbox_events`    | SMTP email                                           |
| Book renewal due    | library DB `outbox_events` | user.RenewBook THEN library.RenewBook (ORDER CRITICAL) |

**Renewal call order**: `user.RenewBook` BEFORE `library.RenewBook` — verified from `previous/job/src/renew_books.rs`. Wrong order = Compat violation.

**Outbox schema is entirely new**: The legacy has no `outbox_events` table (verified:
`previous/library-madome-app/schema/src/lib.rs` has only `book`, `book_tag`, `book_tag_ref`,
`renewed_books`). Outbox tables are purely additive — they do not conflict with existing data.

**Note on `renew_book` event**: this is a **new outbox event kind**. Legacy `previous/job/` uses polling
(`GetCurrentlyRenewingBooks` gRPC call), not outbox events. New design: library service emits a
`renew_book` event when `POST /books/{existing_id}/renew/{new_id}` is called; worker picks it up.
No separate CronJob service needed.

**Outbox table schema** (one per service with outbox):

```sql
CREATE TABLE outbox_events (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    kind            TEXT NOT NULL,
    payload         JSONB NOT NULL,
    idempotency_key TEXT NOT NULL UNIQUE,
    attempts        INT NOT NULL DEFAULT 0,
    last_error      TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    next_attempt_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processed_at    TIMESTAMPTZ,
    failed_at       TIMESTAMPTZ
);
CREATE INDEX ON outbox_events (next_attempt_at) WHERE processed_at IS NULL AND failed_at IS NULL;
```

**Reliability**: at-least-once, idempotent jobs, `SELECT ... FOR UPDATE SKIP LOCKED`, bounded exponential backoff, dead-letter after `max_attempts`.

PostgreSQL LISTEN/NOTIFY preferred; polling as fallback.

---

### 2.7 Why This Is Better Than Legacy

| Concern                | Legacy (`previous/sdk/`)                   | New Design                                                               |
| ---------------------- | ------------------------------------------ | ------------------------------------------------------------------------ |
| Cookie attributes      | Per-service copy-paste from SDK            | Centralized in `crates/madome-auth-types` builders                       |
| Auth validation        | Manual cookie read + JWT parse per handler | `IdentityHeaders` extractor (services check headers injected by gateway) |
| Internal service calls | Ad-hoc HTTP `/_/` routes                   | gRPC via tonic; proto defs in `packages/proto/`                          |
| Business logic         | Mixed with axum handler code               | Isolated `usecase/` (no framework imports)                               |
| DB queries             | Mixed with domain logic                    | `domain/` trait + `infra/db.rs` sea-orm impl; entities in `schema/src/`  |
| Test isolation         | Shared DB state                            | `TestDb` with per-test isolated schemas (sea-orm migrations)             |
| Contract validation    | None                                       | `contracts/` + `contract-harness` in CI                                  |
| Config validation      | Runtime panic on missing env var           | Fail-fast at startup via `std::env::var()`                               |
| Observability          | Ad-hoc                                     | `init_tracing()` + structured spans per handler                          |

---

### 2.8 Contract Test Infrastructure

```
contracts/
  cookies/prod.txt, dev.txt      # exact Set-Cookie strings per env
  http/auth/*.json               # check_token, create_token_pair, refresh_token_pair, ...
  http/library/*.json            # list_books, get_book, search_books, ...
  http/users/*.json              # get_me, list_tastes, ...
  spa-routing/routes.txt         # paths → must return 200 from nginx
  openapi/public.yaml            # frozen Compat OpenAPI spec (CI assertion)
```

`tools/contract-harness/` Rust binary:

```bash
cargo run -p contract-harness -- --base-url http://localhost:3000 --env dev
# Exits 0: all pass. Exits 1: diff of failures.
```

---

## Part 3: Implementation Sequence (v2 — master+dev workflow)

Each **Unit** is one master PR. WIP commits flow on `dev`; open master PR when
the unit is fully implemented, tested, and all gates pass.
GitHub Issues are opened at planning time; PRs reference with `closes #N`.

---

### Unit A — Workspace + Shared Crates + CI

**Deliverable**: Cargo workspace with all shared crates implemented and tested + gaji CI workflows.

**Dev commits (guidance):**
1. `Cargo.toml` workspace root + `rust-toolchain.toml`
2. `crates/madome-domain`
3. `crates/madome-auth-types`
4. `crates/madome-core`
5. `packages/proto/` — user.proto, library.proto, notification.proto; tonic-build stubs
6. `crates/madome-testing`
7. `workflows/` TypeScript gaji source → `gaji build` → `.github/workflows/ci.yml`

**Gates (master PR):**
- `cargo fmt --all -- --check` clean
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` clean
- `cargo test --workspace --all-features` — 0 failures, 0 skipped
- CI green on master PR

**Tests to add:**
- `madome-domain`: serde round-trips for all enums; `UserRole` PartialOrd/Ord; `BookSortBy` + `SearchBookSortBy` custom deserialize (`"id-desc"` → `Id(Sort::Desc)`, `"rank-desc"` → `RankDesc` etc.); `PageRequest` defaults (per_page=25, page=1) and boundary validation (per_page 1–100)
- `madome-auth-types`: doc tests for all cookie builders; `validate_access_token()` happy + error; `IdentityHeaders` 401 on missing/invalid headers
- `madome-core`: `AppError` → HTTP status mapping; `init_tracing()` doesn't panic

**Docs to add:**
- `README.md` (monorepo structure, quickstart, dev setup)
- `contracts/README.md` (placeholder; full content in Unit B)

---

### Unit B — Contract Fixtures + Contract Harness

**Deliverable**: `contracts/` with all fixture files (all **fail** — proves gap), plus `tools/contract-harness/` Rust binary.

**Dev commits:**
1. `contracts/cookies/{dev,prod}.txt`
2. `contracts/http/auth/*.json`, `contracts/http/library/*.json`, `contracts/http/users/*.json`
3. `contracts/spa-routing/routes.txt`
4. `contracts/openapi/public.yaml` — captured from legacy `previous/madome-openapi/` and frozen
5. `tools/contract-harness/` — fixture loader, HTTP assertions, diff reporter

**Gates (master PR):**
- `cargo build -p contract-harness` succeeds
- Harness runs against a live service + **all assertions fail** (expected)
- `cargo test --workspace --all-features` 0 failures

**Tests to add:**
- Fixture parsing unit tests; diff output format tests

**Docs to add:**
- `contracts/README.md` (full: fixture format, how to run harness, how to add contracts)

---

### Unit C — Auth Service

**Deliverable**: `services/auth` full hexagonal stack. All auth contract assertions pass.

**Note**: Auth service has **no gRPC server** — gateway validates JWT locally via `validate_access_token()` pure function, no auth gRPC call needed.

**Dev commits:**
1. `services/auth/schema/` — entity models: users, auth_codes, passkeys
2. `services/auth/migration/` — migration binary
3. `services/auth/src/domain/` — types + repository traits
4. `services/auth/src/infra/db.rs`, `cache.rs` (Redis authcodes + passkey sessions)
5. `services/auth/src/usecase/` — authcode CRUD, token issue/refresh/revoke, passkey flows
6. `services/auth/src/handlers/` — all routes from §1.2 auth table
7. `services/auth/src/router.rs`, `config.rs`, `error.rs`, `main.rs`
8. `services/auth/tests/integration/`

**Gates (master PR):**
- Unit + integration tests: 0 failures, 0 skipped
- `cargo run -p contract-harness -- --service auth --base-url http://localhost:3112` exits 0
- Cookie attrs verified attribute-by-attribute vs `contracts/cookies/dev.txt`
- Outbox: `authcode_created` event in same transaction as authcode creation

**Tests to add:**
- Every handler: happy path + all error statuses (401, 403, 404, 409, 429)
- Auth-gated routes: valid token → 200; no token → 401; expired → 401
- Refresh: validates refresh JWT, ignores expired access JWT, issues new pair
- Logout: both cookies cleared with Max-Age=0
- Passkey: registration start/finish, authentication start/finish
- Outbox: `authcode_created` event inserted in same transaction

**Docs to add:** `services/auth/README.md` (env vars: DATABASE_URL, REDIS_URL, JWT_SECRET, WEBAUTHN_RP_ID, ...)

---

### Unit D — Users Service

**Deliverable**: `services/users` full stack + gRPC UserService + NotificationService. All users contracts pass.

**Dev commits:**
1. `services/users/schema/` — users, tastes, histories, notifications, fcm_tokens
2. `services/users/migration/`
3. `services/users/src/domain/`, `infra/db.rs`, `infra/grpc.rs`
4. `services/users/src/usecase/`
5. `services/users/src/handlers/`
6. `services/users/src/grpc_server.rs` — UserService + NotificationService tonic impl
7. Integration tests

**Gates (master PR):**
- Unit + integration tests: 0 failures, 0 skipped
- `cargo run -p contract-harness -- --service users` exits 0
- gRPC: GetUser, GetTastes, RenewBook, CreateNotification all respond correctly

**Tests to add:**
- Taste: add book/book_tag taste; delete; list all; list filtered by `book-ids[]`; get by kind/value; 409 on duplicate
- History: create/update (upsert by book_id); delete; list paginated
- Notifications: list; FCM token upsert
- `@me` path: `@me` not `me` in URL matching
- DELETE JSON body: correct extraction of kind + value from request body
- gRPC: UserService + NotificationService with MockGrpcServer

**Docs to add:** `services/users/README.md`

---

### Unit E — Library Service

**Deliverable**: `services/library` full stack + Meilisearch + gRPC LibraryService. All library contracts pass.

**Dev commits:**
1. `services/library/schema/` — books, book_tags, book_tag_relations, currently_renewing, outbox_events
2. `services/library/migration/`
3. `services/library/src/domain/`, `infra/db.rs`, `infra/search.rs` (Meilisearch)
4. `services/library/src/usecase/` — book/tag CRUD, search, OG HTML
5. `services/library/src/handlers/`
6. `services/library/src/grpc_server.rs` — LibraryService (RenewBook RPC)
7. Integration tests

**Gates (master PR):**
- Unit + integration tests: 0 failures, 0 skipped
- `cargo run -p contract-harness -- --service library` exits 0
- `legacy-only` filter behavior correct
- Hyphen fallback tested (warns + retries with spaces)
- Outbox: `book_added` and `renew_book` events verified in DB

**Tests to add:**
- GET /books: pagination, `kinds[]` filter, `released`, `legacy-only`, `exclude-dislikes`
- GET /books dispatch: `ids[]` → by IDs; `tags[]` → by tags
- Deprecated `kind` param still works
- Search: Meilisearch query; filter params
- Tag search: hyphen fallback path logs warn + retries
- OG routes: no auth required; 200 HTML
- Admin routes: role ≥ 1 and role ≥ 2 gates
- Outbox: `book_added` and `renew_book` events in DB after writes

**Docs to add:** `services/library/README.md`

---

### Unit F — Image + Sync Services

**Deliverable**: `services/image` + `tools/image-gen/` + `services/sync`. Image and sync contracts pass.

**Dev commits:**
1. `services/image/src/` — GET (path completion + Range), GET `/@` (directory listing), PUT (multipart)
2. `tools/image-gen/src/` — deterministic placeholder image generator
3. `services/sync/src/` — HMAC middleware + IP allowlist + SSE stream + users gRPC calls

**Gates (master PR):**
- Image: GET 200/206 (Range), directory listing, PUT 204
- Directory listing: sorted by char count; thumbnail exclusion
- Sync: HMAC validation; nonce replay prevention
- `cargo run -p contract-harness -- --service image` exits 0

**Tests to add:**
- Image GET: exact match, path completion, range request, directory listing sort/exclusion
- Image PUT: role ≥ 2 required
- Sync: HMAC correctness; nonce replay → 429; IP allowlist → 403

**Docs to add:**
- `services/image/README.md`
- `services/sync/README.md` (HMAC signing algorithm, IP allowlist config)

---

### Unit G — Worker Service

**Deliverable**: `services/worker` — outbox processor (FCM, email, renewal). Worker integration tests pass.

**Internal structure** (no HTTP server, no gRPC server — pure background processor):

```
services/worker/
  src/
    main.rs      # tokio::main — LISTEN/NOTIFY listener + fallback poll loop
    config.rs    # DB URLs (auth, library, users), FCM credentials, SMTP config
    jobs/
      fcm.rs     # book_added → CreateNotification gRPC → FCM push
      email.rs   # authcode_created → SMTP send
      renew.rs   # renew_book → user.RenewBook THEN library.RenewBook
    infra/
      grpc.rs    # tonic clients: NotificationService, LibraryService, UserService
      mail.rs    # email client (lettre)
      fcm.rs     # FCM HTTP v1 API client
```

**Dev commits:**
1. `services/worker/src/config.rs`
2. `services/worker/src/infra/grpc.rs` — tonic clients
3. `services/worker/src/infra/mail.rs` — email client (lettre)
4. `services/worker/src/infra/fcm.rs` — FCM HTTP v1 client
5. `services/worker/src/jobs/{fcm,email,renew}.rs`
6. `services/worker/src/main.rs` — LISTEN/NOTIFY + fallback poll loop
7. Integration tests with TestDb

**Gates (master PR):**
- Unit + integration tests: 0 failures, 0 skipped
- Renewal order: user.RenewBook called BEFORE library.RenewBook — enforced by test assertion
- Dead-letter: after max_attempts → `failed_at` set, event not silently dropped
- Idempotency: duplicate event not processed twice

**Tests to add:**
- Each job: mock gRPC + mock mail/FCM; assert exact call args
- Integration: real TestDb outbox; insert event → assert processed_at set + mock client invoked
- Renewal order: assert user.RenewBook precedes library.RenewBook call (verified via mock call sequence)
- Retry: simulate failure → assert next_attempt_at incremented; max_attempts → failed_at
- Idempotency: same idempotency_key not sent twice

**Docs to add:** `services/worker/README.md` (outbox schema, job registry, idempotency guarantees, retry policy)

---

### Unit H — Gateway Service

**Deliverable**: `services/gateway` — pingora proxy with JWT enrichment + header stripping + routing. Gateway integration tests pass.

**Dev commits:**
1. `services/gateway/src/config.rs`
2. `services/gateway/src/router.rs`
3. `services/gateway/src/filters/enrich.rs`
4. `services/gateway/src/main.rs`
5. Unit tests via `make_test_session()`
6. Integration tests: pingora on random loopback port + mock upstream

**Gates (master PR):**
- x-madome-* headers stripped (cannot be spoofed by client)
- Valid cookie: x-madome-user-id + x-madome-user-role injected
- Invalid/absent cookie: forwarded without identity headers
- Set-Cookie from upstream passes through untouched
- All unit + integration tests: 0 failures, 0 skipped

**Tests to add:**
- `enrich.rs` unit: strip spoofed headers; valid JWT → inject; invalid → no inject; absent → no inject
- Integration: mock upstream asserts injected headers; asserts stripped headers
- Integration: upstream Set-Cookie passes through
- Routing: correct upstream by host + path prefix

**Docs to add:** `services/gateway/README.md` (routing table, x-madome-* whitelist, identity header contract)

---

### Unit I — Frontend + GitOps

**Deliverable**: `apps/web` Vue 3 SPA scaffold + `k8s/` base manifests + SOPS secrets. SPA routing contracts pass.

**Dev commits:**
1. `apps/web/` — Vue 3 scaffold (vite, build-time env injection, no SSR)
2. `k8s/base/` — Deployments, Services, ingress, gRPC ports, NetworkPolicy
3. `k8s/overlays/{dev,prod}/` — Kustomize overlays
4. `k8s/sops/` — SOPS+age encrypted secrets

**Gates (master PR):**
- SPA routing: all paths in `contracts/spa-routing/routes.txt` return 200 from nginx
- `kubectl apply --dry-run=client -k k8s/overlays/dev` succeeds
- SOPS decrypt round-trip succeeds

**Tests to add:**
- SPA routing check via contract-harness
- SOPS round-trip (in CI or local script)

**Docs to add:**
- `k8s/README.md` (apply, SOPS decrypt, rollback steps)
- `apps/web/README.md` (build command, env var injection, local dev)

---

### Unit J — E2E Smoke + Rollback Rehearsal

**Deliverable**: `tools/e2e-smoke/` Rust binary + `tools/openapi/` + rollback rehearsal. All smoke tests pass. **Compat exit criteria met.**

**Dev commits:**
1. `tools/e2e-smoke/src/main.rs` — full-stack: auth flow → browse books → read → tastes
2. `tools/openapi/` — merges per-service utoipa specs → `dist/openapi/public.yaml`
3. CI: add `e2e` gaji job to workflows (becomes required check on master PRs)
4. Rollback rehearsal: document + perform steps

**Gates (master PR — Compat exit criteria):**
- `cargo run -p e2e-smoke -- --env dev` exits 0
- Cookie attrs verified end-to-end vs `contracts/cookies/dev.txt`
- `cargo run -p contract-harness -- --base-url ... --env dev` exits 0 for all services
- `dist/openapi/public.yaml` matches `contracts/openapi/public.yaml`
- Rollback rehearsal completed

**Tests to add:**
- `e2e-smoke/`: auth flow; book browsing; taste add/delete; history; notification
- `openapi/`: diff `dist/` vs `contracts/openapi/public.yaml`

**Docs to add:**
- `docs/ops/database.md` — CNPG Cluster, schema migrations, backup/restore, initial migration, secrets, monitoring, outbox health
- Rollback runbook (in `k8s/README.md` or `docs/ops/rollback.md`)

---

## Part 4: Verification Commands

```bash
# Every PR (run locally before pushing):
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features

# After Unit B (all assertions fail = expected):
cargo run -p contract-harness -- --base-url http://localhost:3000

# Per service after Unit C–H:
cargo run -p contract-harness -- --service auth    --base-url http://localhost:3112
cargo run -p contract-harness -- --service users   --base-url http://localhost:3112
cargo run -p contract-harness -- --service library --base-url http://localhost:3112

# After Unit J (full stack):
cargo run -p e2e-smoke -- --env dev
```

---

## Tests / Docs Summary

| Unit | Key tests | Key docs |
|------|-----------|----------|
| A | Domain serde; BookSortBy + SearchBookSortBy custom deserialize; PageRequest validation; cookie builder doc tests; IdentityHeaders 401 | `README.md`, `contracts/README.md` (placeholder) |
| B | Fixture parsing; diff output format | `contracts/README.md` (full) |
| C | All auth routes + status codes; cookie attrs attr-by-attr; passkey flows; outbox | `services/auth/README.md` |
| D | Taste/history CRUD; list by `book-ids[]`; JSON body DELETE; `@me` path; gRPC RPCs | `services/users/README.md` |
| E | Book filters (kinds[], legacy-only, dislikes); search; hyphen fallback; OG no-auth; outbox | `services/library/README.md` |
| F | Path completion; Range 206; directory listing; HMAC; nonce replay | `services/image/README.md`, `services/sync/README.md` |
| G | Job mocks; outbox integration; renewal order; retry/dead-letter; idempotency | `services/worker/README.md` |
| H | Header strip/inject; routing; Set-Cookie passthrough | `services/gateway/README.md` |
| I | SPA route → 200; k8s dry-run; SOPS round-trip | `k8s/README.md`, `apps/web/README.md` |
| J | Full e2e flow; openapi diff | `docs/ops/database.md`, rollback runbook |
