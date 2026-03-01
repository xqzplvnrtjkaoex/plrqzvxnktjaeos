# Plan: Code Review Fixes — Safety, Error Mapping, Performance

## Context

Full codebase review found 18 issues across all crate usage. After verifying each issue
against the actual code, **14 are confirmed actionable** (originally 11; items 11, 12, 18
re-added after discussion). 4 were invalidated or downgraded.

---

## Changes

### 1. `proto_to_auth_user` → `TryFrom` impl (items 3, 4)

**File: `services/auth/src/infra/grpc.rs:57-63`**

Replace standalone function with `TryFrom` trait impl:

```rust
impl TryFrom<madome_proto::user::User> for AuthUser {
    type Error = AuthServiceError;

    fn try_from(user: madome_proto::user::User) -> Result<Self, Self::Error> {
        let id = user
            .id
            .parse()
            .map_err(|_| anyhow::anyhow!("invalid UUID from users service"))?;
        let role = u8::try_from(user.role)
            .map_err(|_| anyhow::anyhow!("role out of u8 range: {}", user.role))?;
        Ok(AuthUser { id, email: user.email, role })
    }
}
```

Update callers (lines 35, 50):
- `proto_to_auth_user(resp.into_inner())` → `resp.into_inner().try_into()?`

### 2. `user_from_model` → `TryFrom` impl (item 6)

**File: `services/users/src/infra/db.rs:92-102`**

Replace standalone function with `TryFrom` trait impl. The conversion can fail (role out of
u8 range), so `TryFrom` is correct — the **caller** decides whether to panic or propagate:

```rust
impl TryFrom<users::Model> for User {
    type Error = anyhow::Error;

    fn try_from(model: users::Model) -> Result<Self, Self::Error> {
        let role = u8::try_from(model.role)
            .map_err(|_| anyhow::anyhow!("role column out of u8 range: {}", model.role))?;
        Ok(User {
            id: model.id,
            name: model.name,
            handle: model.handle,
            email: model.email,
            role,
            created_at: model.created_at,
            updated_at: model.updated_at,
        })
    }
}
```

Update call sites (lines 40, 49) — both are `Ok(model.map(user_from_model))` where
`model: Option<users::Model>`. Use `Option::map + transpose`:

```rust
// model: Option<users::Model> → Result<Option<User>, UsersServiceError>
Ok(model.map(User::try_from).transpose().context("corrupted user row")?)
```

Error propagation (not panic) because a corrupted DB row shouldn't crash the service.

### 3. gRPC error mapping — `From<UsersServiceError> for Status` (item 5)

**File: `services/users/src/error.rs`**

Implement `From` trait instead of a standalone function — more idiomatic.

Add import: `use tonic::Status;`

```rust
impl From<UsersServiceError> for Status {
    fn from(e: UsersServiceError) -> Self {
        match &e {
            UsersServiceError::UserNotFound
            | UsersServiceError::TasteNotFound
            | UsersServiceError::HistoryNotFound
            | UsersServiceError::BookNotFound
            | UsersServiceError::BookTagNotFound => Status::not_found(e.to_string()),
            UsersServiceError::UserAlreadyExists
            | UsersServiceError::TasteAlreadyExists => Status::already_exists(e.to_string()),
            UsersServiceError::InvalidHandle
            | UsersServiceError::MissingData => Status::invalid_argument(e.to_string()),
            UsersServiceError::Forbidden => Status::permission_denied(e.to_string()),
            UsersServiceError::Internal(_) => Status::internal(e.to_string()),
        }
    }
}
```

**File: `services/users/src/grpc_server.rs`**

8 total `.map_err(...)` calls exist. 3 are UUID parsing (lines 33, 87, 151) — keep as-is.
Replace the other 5 (lines 41, 66, 101, 136, 173) with `.map_err(Into::into)`.

Note: this also **fixes incorrect error mapping** — lines 41 and 66 currently map ALL
errors (including `Internal`) to `NOT_FOUND`. The `From` impl maps each variant to its
correct status code.

### 4. Magic numbers in `create_user` handler (item 2)

**File: `services/users/src/handlers/user.rs:27-33`**

Replace raw numeric comparisons with `UserRole` constants:

```rust
use madome_domain::user::UserRole;

// Only Bot can create users
if identity.user_role < UserRole::Bot.as_u8() {
    return Err(UsersServiceError::Forbidden);
}
// Can only create Normal or Developer users
let role = body.role.unwrap_or(UserRole::Normal.as_u8());
if role > UserRole::Developer.as_u8() {
    return Err(UsersServiceError::Forbidden);
}
```

### 5. Taste upsert — single-query ON CONFLICT (item 7)

**File: `services/users/src/infra/db.rs:295-362`**

Replace SELECT-then-INSERT/UPDATE with a single `INSERT ... ON CONFLICT DO UPDATE ... WHERE`.
The `action_and_where` method (sea-query) adds a WHERE clause to DO UPDATE, so the update
only fires when `is_dislike` actually differs. `exec_without_returning` returns `u64` (rows
affected): 0 means the row already existed with the same value.

```rust
async fn upsert_book(&self, taste: &TasteBook) -> Result<bool, UsersServiceError> {
    let rows_affected = taste_books::Entity::insert(taste_books::ActiveModel {
        user_id: Set(taste.user_id),
        book_id: Set(taste.book_id),
        is_dislike: Set(taste.is_dislike),
        created_at: Set(taste.created_at),
    })
    .on_conflict(
        OnConflict::columns([taste_books::Column::UserId, taste_books::Column::BookId])
            .update_column(taste_books::Column::IsDislike)
            .action_and_where(
                Expr::col((taste_books::Entity, taste_books::Column::IsDislike))
                    .ne(taste.is_dislike),
            )
            .to_owned(),
    )
    .exec_without_returning(&self.db)
    .await
    .context("upsert taste book")?;
    Ok(rows_affected > 0)
}
```

Same pattern for `upsert_book_tag` (3 conflict columns: user_id, tag_kind, tag_name).

Imports needed: `OnConflict` already imported at file top (line 6). Add `Expr` to the same
import line: `use sea_orm::sea_query::{Expr, OnConflict};` (currently only `OnConflict`).

### 6. N+1 notification list — batch tag loading (item 8)

**File: `services/users/src/infra/db.rs:539-557`**

Replace per-notification tag query loop (lines 539-557) with a single batch query.

**Before (N+1):** for-loop on line 540 queries `notification_book_tags` once per notification.

**After (batch):** replace lines 539-558 with:

```rust
// Batch-load all tags for the fetched notifications
let notification_ids: Vec<Uuid> = models.iter().map(|m| m.id).collect();
let all_tags = notification_book_tags::Entity::find()
    .filter(notification_book_tags::Column::NotificationBookId.is_in(notification_ids))
    .all(&self.db)
    .await
    .context("list notification book tags")?;

// Group by notification_book_id
let mut tags_map: std::collections::HashMap<Uuid, Vec<(String, String)>> =
    std::collections::HashMap::new();
for tag in all_tags {
    tags_map
        .entry(tag.notification_book_id)
        .or_default()
        .push((tag.tag_kind, tag.tag_name));
}

let results = models
    .into_iter()
    .map(|model| {
        let book_tags = tags_map.remove(&model.id).unwrap_or_default();
        NotificationBook {
            id: model.id,
            user_id: model.user_id,
            book_id: model.book_id,
            book_tags,
            created_at: model.created_at,
        }
    })
    .collect();
Ok(results)
```

Reduces 1 + N queries to 2 queries (one for notifications, one for all their tags).

### 7. Missing indexes — new migration (items 9, 14)

**New file: `services/users/migration/src/m20250301_000008_add_missing_indexes.rs`**

```rust
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .table(FcmTokens::Table)
                    .col(FcmTokens::UserId)
                    .name("idx_fcm_tokens_user_id")
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .table(NotificationBookTags::Table)
                    .col(NotificationBookTags::NotificationBookId)
                    .name("idx_notification_book_tags_notification_book_id")
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_index(Index::drop().name("idx_notification_book_tags_notification_book_id").to_owned()).await?;
        manager.drop_index(Index::drop().name("idx_fcm_tokens_user_id").to_owned()).await
    }
}
```

**Update `services/users/migration/src/lib.rs`:** Add module + `Box::new(...)` entry.

Reference existing Iden enums from migration files 6 and 7 (or define locally).

### 8. Auth startup — use `connect_lazy()` (item 13)

**File: `services/auth/src/main.rs:35-39`**

```diff
 let users_channel = tonic::transport::Channel::from_shared(config.users_grpc_url.clone())
     .expect("invalid USERS_GRPC_URL")
-    .connect()
-    .await
-    .expect("failed to connect to users gRPC");
+    .connect_lazy();
```

This prevents auth from failing to start when the users service is temporarily unavailable.
The channel reconnects lazily on first use, which is how the contract-harness already does it.

### 9. PasskeyResponse — add ms-precision timestamp serializer (item 15)

**File: `services/auth/src/handlers/passkeys.rs:27-31`**

```diff
 #[derive(Serialize)]
 pub struct PasskeyResponse {
     pub credential_id: String,
+    #[serde(serialize_with = "madome_core::serde::to_rfc3339_ms")]
     pub created_at: DateTime<Utc>,
 }
```

Consistent with all other response types in the codebase (user, taste, history, notification
handlers all use `to_rfc3339_ms`). Passkeys are new (not in legacy), so no Compat risk.

---

### 10. Unify JWT: claims struct + validation functions + feature gate (items 12, 18)

Three duplications eliminated:
- `AccessClaims` / `TokenClaims` → unified `JwtClaims`
- Two identical `Validation::new()` configs → shared `decode_jwt()`
- Feature renamed: `USE_ONLY_IN_AUTH_SERVICE` — gates `Serialize` + `validate_token()`

**File: `crates/madome-auth-types/Cargo.toml`**

```toml
[features]
USE_ONLY_IN_AUTH_SERVICE = []
```

**File: `crates/madome-auth-types/src/token.rs`**

Full rewrite of validation module:

```rust
/// JWT claims payload shared by token creation (auth service) and validation (gateway).
///
/// # Fields
///
/// | Field | JWT claim | Rust type | Meaning |
/// |-------|-----------|-----------|---------|
/// | `sub` | `sub` | UUID string | user ID |
/// | `role` | custom | `u8` wire value | see [`madome_domain::user::UserRole`] |
/// | `exp` | `exp` | seconds since epoch | token expiration |
///
/// # Feature gate
///
/// [`Deserialize`] is always available — all consumers validate tokens.
/// [`Serialize`] requires the **`USE_ONLY_IN_AUTH_SERVICE`** cargo feature. Only the auth
/// service enables it because it is the sole token issuer.
#[derive(Debug, Deserialize)]
#[cfg_attr(any(feature = "USE_ONLY_IN_AUTH_SERVICE", test), derive(Serialize))]
pub struct JwtClaims {
    /// User ID (UUID string).
    pub sub: String,
    /// User role as `u8` wire value.
    pub role: u8,
    /// Expiration timestamp (seconds since UNIX epoch).
    pub exp: u64,
}

// ── Core decode (private) ────────────────────────────────────────────────

/// Decode and validate a JWT, returning raw claims.
///
/// Validation: HS256, exp checked, required claims: `exp` + `sub`.
/// Default leeway = 60s — tolerates clock skew between services.
/// Same library + version as legacy; matches legacy behavior.
fn decode_jwt(token: &str, secret: &str) -> Result<JwtClaims, AuthError> {
    let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.validate_exp = true;
    validation.required_spec_claims.clear();
    validation.set_required_spec_claims(&["exp", "sub"]);

    let data = decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::Expired,
        jsonwebtoken::errors::ErrorKind::InvalidSignature
        | jsonwebtoken::errors::ErrorKind::InvalidEcdsaKey
        | jsonwebtoken::errors::ErrorKind::InvalidRsaKey(_) => AuthError::InvalidSignature,
        _ => AuthError::Malformed,
    })?;

    Ok(data.claims)
}

// ── Public: all consumers ────────────────────────────────────────────────

/// Validate an access-token cookie value, returning parsed identity.
///
/// This is the primary public API for token validation. Gateway calls this
/// on every request to extract user identity from the JWT cookie.
pub fn validate_access_token(cookie_value: &str, secret: &str) -> Result<TokenInfo, AuthError> {
    let claims = decode_jwt(cookie_value, secret)?;
    let user_id = claims.sub.parse::<Uuid>().map_err(|_| AuthError::Malformed)?;
    Ok(TokenInfo {
        user_id,
        user_role: claims.role,
        access_token_exp: claims.exp,
    })
}

// ── Feature-gated: auth service only ─────────────────────────────────────

/// Validate a token and return raw JWT claims.
///
/// Used by the auth service's refresh flow — validates the refresh token,
/// then looks up the user from the `sub` claim to issue new tokens.
///
/// Requires the `USE_ONLY_IN_AUTH_SERVICE` feature. Only the auth service
/// should call this directly; all other consumers use [`validate_access_token`].
#[cfg(any(feature = "USE_ONLY_IN_AUTH_SERVICE", test))]
pub fn validate_token(token: &str, secret: &str) -> Result<JwtClaims, AuthError> {
    decode_jwt(token, secret)
}
```

The `Serialize` import must be feature-gated to avoid unused-import warnings:

```rust
use serde::Deserialize;
#[cfg(any(feature = "USE_ONLY_IN_AUTH_SERVICE", test))]
use serde::Serialize;
```

Test helper `make_token` uses `JwtClaims` (has `Serialize` via `cfg(test)`).

**File: `services/auth/Cargo.toml`**

```diff
-madome-auth-types = { path = "../../crates/madome-auth-types" }
+madome-auth-types = { path = "../../crates/madome-auth-types", features = ["USE_ONLY_IN_AUTH_SERVICE"] }
```

**File: `services/auth/src/usecase/token.rs`**

Remove: `TokenClaims` struct + `validate_token` function + unused imports:
- jsonwebtoken: remove `Validation`, `decode`, `DecodingKey` (keep `encode`, `EncodingKey`, `Header`)
- serde: remove `use serde::{Deserialize, Serialize}` entirely (no local types derive them)
Import from shared crate:

```rust
use madome_auth_types::token::{JwtClaims, validate_token};
```

Replace all `TokenClaims` → `JwtClaims`.
Replace `validate_token(...)` call in `RefreshTokenUseCase::execute` (line 166) with:

```rust
// validate_token returns detailed AuthError (Expired, InvalidSignature, Malformed),
// but for the refresh flow any failure = invalid refresh token from the client's
// perspective. The detailed error is intentionally discarded here.
let claims = validate_token(refresh_token_value, &self.jwt_secret)
    .map_err(|_| AuthServiceError::InvalidRefreshToken)?;
```

**No changes** to `services/users/Cargo.toml` or `crates/madome-testing/Cargo.toml` —
they don't enable `USE_ONLY_IN_AUTH_SERVICE`.

---

### 11b. IdentityHeaders rejection: 401 → 500 + rustdoc (item 11)

**File: `crates/madome-auth-types/src/identity.rs`**

These headers are gateway-internal. If missing, it's infrastructure failure, not client
auth failure. Change rejection from 401 to 500 and add tracing.

Lines 43-46 — change rejection:

```rust
async move {
    let user_id = user_id.ok_or_else(|| {
        tracing::error!("x-madome-user-id header missing or invalid — gateway misconfigured");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let user_role = user_role.ok_or_else(|| {
        tracing::error!("x-madome-user-role header missing or invalid — gateway misconfigured");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(Self { user_id, user_role })
}
```

Struct-level rustdoc (line 8) — update to reflect 500 + cross-reference:

```rust
/// User identity injected by the gateway via `x-madome-user-id` and `x-madome-user-role` headers.
///
/// Rejects with `500 Internal Server Error` if headers are missing or invalid.
/// See [`IdentityHeaders::from_request_parts`] for details.
```

Method-level rustdoc — add detailed explanation:

```rust
/// Extract identity from gateway-injected headers.
///
/// # Rejection
///
/// Returns `500 Internal Server Error` (not 401) when headers are missing or
/// malformed. These headers are injected by the gateway after JWT validation —
/// their absence means the gateway is misconfigured or the request bypassed it
/// entirely. This is a server-side infrastructure failure, not a client
/// authentication problem. The client cannot fix this by re-authenticating.
///
/// An `error!` log is emitted so operators can diagnose the issue.
```

**Update tests** (lines 84, 94, 101, 112): `StatusCode::UNAUTHORIZED` → `StatusCode::INTERNAL_SERVER_ERROR`.

**Add dependency**: `tracing` to `madome-auth-types/Cargo.toml` (workspace dep).

---

### 12. Document type conversion conventions

**File: `.claude/docs/code-conventions.md`**

Add a new section after "gRPC error mapping" (line 183).
Also update lines 175-183 to reference the `From` trait pattern:

```markdown
## gRPC error mapping

Domain errors convert to `tonic::Status` via `From` trait impl. Use `.map_err(Into::into)`
at call sites.

| Domain error | tonic Status |
|---|---|
| `*NotFound` | `NOT_FOUND` |
| `*AlreadyExists` | `ALREADY_EXISTS` |
| `InvalidHandle`, `MissingData` | `INVALID_ARGUMENT` |
| `Forbidden` | `PERMISSION_DENIED` |
| `Internal` | `INTERNAL` |
```

Then add the gRPC client channels section and the type conversion section:

```markdown
---

## Serde format convention

| Wire location | `rename_all` | Rationale |
|---------------|--------------|-----------|
| **URL** (query params, path segments) | `kebab-case` | URL convention — hyphens are word separators |
| **JSON body** (request / response) | `snake_case` | JSON convention — underscores are standard |

Examples:
- URL: `BookKind` (`artist-cg`), `BookTagKind` (`female`), `Sort` (`id-desc`), `*ListQuery` structs
- Body: `TasteResponse` (`book_tag`), `CreateTasteRequest`, `HistoryResponse`, `UserRole` (`normal`)

Domain types follow the convention of their primary serialization context.

---

## gRPC client channels

Use `connect_lazy()` for inter-service gRPC channels in long-running services. Eager
`connect()` creates startup ordering dependencies — if the target service restarts or
isn't ready yet, the caller fails to start. Lazy connection defers until first RPC call
and auto-reconnects.

Use eager `connect()` only when immediate failure is desired: CLI tools, one-shot scripts,
or health-check probes that need to verify connectivity upfront.

---

## Type conversion conventions

Implement conversions using std traits (`From`, `TryFrom`, `FromStr`). Never write
standalone conversion functions when a trait impl works.

**`From` vs `TryFrom`:** If the conversion can fail at all, use `TryFrom`. The caller
decides whether to panic (`.expect()`) or propagate (`.try_into()?`). Don't bake panic
into the conversion itself — that removes the caller's choice.

Call-site patterns — prefer `Into::into` / `TryInto::try_into` over naming the concrete type:

| Context | Pattern | Example |
|---------|---------|---------|
| Infallible A→B | `.into()` | `let status: Status = error.into();` |
| Fallible A→B (propagate) | `.try_into()?` | `let user: User = model.try_into()?;` |
| Fallible A→B (panic) | `.try_into().expect("reason")` | `model.try_into().expect("corrupted row")` |
| `Result::map_err` | `.map_err(Into::into)` | `.await.map_err(Into::into)?` |
| `Option`/`Result`/`Either` map | `.map(Into::into)` | `.ok().map(Into::into)` |
| Iterator (infallible) | `.map_into()` (itertools) | `vec.into_iter().map_into::<B>()` |

**When NOT to use `From`/`TryFrom`:**

Standalone functions or methods are correct when any of these apply:

| Reason | Category | Example |
|--------|----------|---------|
| Same source type, different parsing | Mechanical | `from_kebab_case(&str)` vs `from_snake_case(&str)` — Rust allows only one `From<&str>` per type |
| Extra parameters needed | Mechanical | `From`/`TryFrom` take only the source value — can't pass secret, config, context. Still a pure conversion, just can't fit the signature |
| Side-effectful operation | Semantic | DB lookups, network calls — the result depends on external state, not just the input. `from` implies same input → same output; side effects break that contract |

The first two are **mechanical** — the operation is a pure conversion but the trait can't
express it. The last is **semantic** — the operation isn't a conversion at all, it's a lookup
that produces a different type.

**When type inference fails:**

- Single value: `let b: BType = a.into();`
- Iterator: `.map_into::<BType>()`
- `map_err`: `.map_err(BType::from)` (exception — `Into::into` can't carry a turbofish)

---

## JWT claims

`JwtClaims` lives in `madome-auth-types` as the single source of truth for the JWT
payload. `Serialize` is gated behind the `USE_ONLY_IN_AUTH_SERVICE` cargo feature — only the auth service
enables it.

| Consumer | Feature | Available derives |
|----------|---------|-------------------|
| auth service | `USE_ONLY_IN_AUTH_SERVICE` | `Serialize` + `Deserialize` |
| gateway, users, testing | (none) | `Deserialize` only |

---

## Handler dispatching pattern

When a single legacy endpoint serves multiple query-param modes (e.g. `kind=book`,
`kind=book_tag`, `book-ids[]`), the handler dispatches to different use cases based on
the query params. This is **handler responsibility, not business logic** — each use case
remains a focused unit and the handler just picks the right one.

Do not refactor this into a single "dispatcher" use case — it moves the same branching
deeper without reducing complexity, and couples unrelated use cases.

Example: `services/users/src/handlers/taste.rs` — `get_tastes()` dispatches to 4 use cases
based on `kind` and `book-ids[]` query params.
```

---

## Dropped findings (not bugs)

| # | Finding | Why dropped |
|---|---------|-------------|
| 1 | Taste handler "business logic" | Handler dispatches to different use cases based on query params — this IS handler responsibility |
| 10 | Duplicate `.route()` calls | Idiomatic axum — separate `.route()` per HTTP method is the documented pattern |
| 11 | ~~IdentityHeaders bare StatusCode~~ | Resolved — 401→500 + rustdoc explaining gateway-internal headers (change #11) |
| 12 | ~~JWT clock skew leeway~~ | Resolved — leeway comment in `decode_jwt` rustdoc (change #10) |
| 16 | Inconsistent `kind` format | Intentional: URL types use `kebab-case`, body types use `snake_case` (see code-conventions) |
| 17 | Redis pool size | Operational tuning, not a code defect |
| 18 | ~~Duplicate JWT claims structs~~ | Resolved — unified into `JwtClaims` with feature-gated `Serialize` (change #10) |

---

## Files

| File | Action |
|------|--------|
| `services/auth/src/infra/grpc.rs` | `TryFrom<proto::User> for AuthUser`, remove standalone fn |
| `services/auth/src/main.rs` | `.connect()` → `.connect_lazy()` |
| `services/auth/src/handlers/passkeys.rs` | Add `to_rfc3339_ms` serializer to `created_at` |
| `crates/madome-auth-types/Cargo.toml` | Add `[features] USE_ONLY_IN_AUTH_SERVICE = []` + `tracing` dep |
| `crates/madome-auth-types/src/token.rs` | Rewrite: `JwtClaims` + `decode_jwt` + `validate_token` (gated) + rustdoc |
| `services/auth/Cargo.toml` | Enable `USE_ONLY_IN_AUTH_SERVICE` feature for `madome-auth-types` |
| `services/auth/src/usecase/token.rs` | Remove `TokenClaims` + `validate_token`, import from shared crate |
| `services/auth/tests/integration/token_test.rs` | Update `validate_token` import path + 2 error assertions |
| `crates/madome-auth-types/src/identity.rs` | 401→500, add tracing + rustdoc |
| `services/users/src/error.rs` | Add `From<UsersServiceError> for Status` |
| `services/users/src/infra/db.rs` | `TryFrom<users::Model> for User`, taste upsert ON CONFLICT, N+1 fix |
| `services/users/src/grpc_server.rs` | Replace all `.map_err(...)` with `.map_err(Into::into)` |
| `services/users/src/handlers/user.rs` | Replace magic numbers with `UserRole` constants |
| `services/users/migration/src/m20250301_000008_add_missing_indexes.rs` | **New** — indexes on fcm_tokens.user_id + notification_book_tags.notification_book_id |
| `services/users/migration/src/lib.rs` | Register new migration |
| `.claude/docs/code-conventions.md` | Add type conversion conventions + gRPC channel + JWT claims sections |

---

## Tests to add or modify

- **Users service has no integration tests** — only unit tests via mocks in usecase modules.
  The mock `upsert_book`/`upsert_book_tag` in `usecase/taste.rs:363-366` returns hardcoded
  values and won't exercise the real ON CONFLICT query. This is acceptable — the ON CONFLICT
  pattern is already proven by the history upsert (line 472) which has the same structure.
- `services/users/src/usecase/taste.rs` mock: trait signatures unchanged → no test changes
- `services/users/src/error.rs` tests: unchanged (error mapping tests cover HTTP `IntoResponse`, not gRPC `From`)
- `services/auth/tests/integration/` — do NOT reference `proto_to_auth_user` (use `MockUserPort`) → no changes
- `crates/madome-auth-types/src/token.rs` tests: `AccessClaims` → `JwtClaims` rename only, logic unchanged
- `crates/madome-auth-types/src/identity.rs` tests: `UNAUTHORIZED` → `INTERNAL_SERVER_ERROR` (4 assertions)
- `services/auth/tests/integration/token_test.rs`: **import + error assertion changes required**
  - Line 4: remove `validate_token` from `madome_auth::usecase::token` import
  - Add: `use madome_auth_types::token::{validate_token, AuthError};`
  - Line 32: `Err(AuthServiceError::InvalidRefreshToken)` → `Err(AuthError::InvalidSignature)`
  - Line 41: `Err(AuthServiceError::InvalidRefreshToken)` → `Err(AuthError::Malformed)`
  - Lines 19, 53, 86, 89, 191: `validate_token(...).unwrap()` — import path changes only, no assertion changes
- No new tests needed — changes are internal safety improvements, not new features

## Docs to add or modify

- `.claude/docs/code-conventions.md` — add serde format convention, gRPC channels, type conversion conventions, JWT claims, handler dispatching sections (change #12)
- `crates/madome-auth-types/src/token.rs` — rustdoc on `JwtClaims` explaining feature gate (change #10)

---

## Verification

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```
