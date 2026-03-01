# Plan: Error Kind Standard — JSON error responses with domain-specific kinds

## Context

Current error responses are plain text `(StatusCode, "lowercase string")`. This makes
client-side error handling fragile — clients must parse human-readable strings.

New standard: JSON body with a machine-readable `kind` (`UPPER_SNAKE_CASE`) and a
human-readable `message` (`lowercase`). Kinds are **domain-specific per service** so
clients can branch on precise error types without string matching.

> **Compat note**: Two deliberate deviations from the legacy format:
> 1. Response body changes from plain text to JSON `{"kind":…,"message":…}`.
> 2. `Content-Type` header changes from `text/plain; charset=utf-8` to `application/json`.
>
> The contract harness gains `expect.body` support. 4 auth fixtures are updated to assert
> on the new JSON body — contracts serve as the complete API spec (status + body).
> Body format is also verified by unit tests in `error.rs`.

---

## Error Response Format

```json
{
  "kind": "USER_NOT_FOUND",
  "message": "user not found"
}
```

- `kind` — `UPPER_SNAKE_CASE` string; machine-readable; stable across releases
- `message` — `lowercase`; human-readable; may change between releases

**Do not expose internal error details** in `message` — especially for `INTERNAL` and
`INVALID_CREDENTIAL`.

---

## Kind Naming Convention (for `code-conventions.md` / `error-kinds.md`)

| Rule | Example |
|------|---------|
| `UPPER_SNAKE_CASE` | `USER_NOT_FOUND` |
| Name the thing that failed, not the HTTP status | `INVALID_TOKEN` not `UNAUTHORIZED` |
| Generic fallback only when no specific kind fits | `INTERNAL`, `NOT_FOUND` |
| Each service defines its own domain kinds in `src/error.rs` | — |
| `INTERNAL` is the only kind shared across all services | — |

---

## Defined Kinds

### `AppError` (generic — `crates/madome-core`)

| Variant | Kind | HTTP |
|---------|------|------|
| `Unauthorized` | `UNAUTHORIZED` | 401 |
| `Forbidden` | `FORBIDDEN` | 403 |
| `NotFound` | `NOT_FOUND` | 404 |
| `Conflict` | `CONFLICT` | 409 |
| `Internal` | `INTERNAL` | 500 |

*AppError is used for middleware/infra-level errors where no domain context is available.*

### `AuthServiceError` (domain-specific — `services/auth`)

| New Variant | Kind | HTTP | Replaces | Used in |
|-------------|------|------|---------|---------|
| `UserNotFound` | `USER_NOT_FOUND` | 404 | `NotFound` (user lookup) at authcode.rs:38, token.rs:108, passkey.rs:183; `Unauthorized` at passkey.rs:80 (401→404 intentional: JWT-authenticated user not found in DB is data inconsistency, semantically 404; no passkey contract fixtures so Compat unaffected) | authcode.rs:38, token.rs:108, passkey.rs:80,183 |
| `CredentialNotFound` | `CREDENTIAL_NOT_FOUND` | 404 | `NotFound` (passkey/credential) | passkey.rs:51,187,236 |
| `InvalidAuthcode` | `INVALID_AUTHCODE` | 401 | `NotFound` (authcode lookup) | token.rs:114 |
| `InvalidToken` | `INVALID_TOKEN` | 401 | `Unauthorized` (JWT access token) | handlers/token.rs:54,57,61 |
| `InvalidRefreshToken` | `INVALID_REFRESH_TOKEN` | 401 | `Unauthorized` (refresh flow) | token.rs:73,157,163 |
| `InvalidSession` | `INVALID_SESSION` | 401 | `Unauthorized` (WebAuthn cache miss) | passkey.rs:133,136,242,245 |
| `InvalidCredential` | `INVALID_CREDENTIAL` | 400 | `BadRequest` (WebAuthn verify fail) | passkey.rs:141,256, handlers/passkeys.rs:60 |
| `TooManyAuthcodes` | `TOO_MANY_AUTHCODES` | 429 | `TooManyRequests` | authcode.rs:43 |
| `Internal(anyhow::Error)` | `INTERNAL` | 500 | `Internal` (unchanged) | everywhere |

*`BadRequest(String)` is removed — the string was internal WebAuthn error detail (do not expose).*
*`Unauthorized` (role insufficient) maps to `InvalidToken` for now (matches legacy 401).*
*`InvalidAuthcode` returns 401 (not 404) — prevents confirming whether a user exists.*

---

## Rust Implementation Pattern

### error.rs (per service)

```rust
#[derive(Debug, thiserror::Error)]
pub enum AuthServiceError {
    #[error("user not found")]
    UserNotFound,
    #[error("credential not found")]
    CredentialNotFound,
    #[error("invalid authcode")]
    InvalidAuthcode,
    #[error("invalid token")]
    InvalidToken,
    #[error("invalid refresh token")]
    InvalidRefreshToken,
    #[error("session expired")]
    InvalidSession,
    #[error("invalid credential")]
    InvalidCredential,
    #[error("too many authcodes")]
    TooManyAuthcodes,
    #[error("internal error")]
    Internal(#[from] anyhow::Error),
}

impl AuthServiceError {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::UserNotFound => "USER_NOT_FOUND",
            Self::CredentialNotFound => "CREDENTIAL_NOT_FOUND",
            Self::InvalidAuthcode => "INVALID_AUTHCODE",
            Self::InvalidToken => "INVALID_TOKEN",
            Self::InvalidRefreshToken => "INVALID_REFRESH_TOKEN",
            Self::InvalidSession => "INVALID_SESSION",
            Self::InvalidCredential => "INVALID_CREDENTIAL",
            Self::TooManyAuthcodes => "TOO_MANY_AUTHCODES",
            Self::Internal(_) => "INTERNAL",
        }
    }
}

impl IntoResponse for AuthServiceError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::UserNotFound | Self::CredentialNotFound => StatusCode::NOT_FOUND,
            Self::InvalidAuthcode
            | Self::InvalidToken
            | Self::InvalidRefreshToken
            | Self::InvalidSession => StatusCode::UNAUTHORIZED,
            Self::InvalidCredential => StatusCode::BAD_REQUEST,
            Self::TooManyAuthcodes => StatusCode::TOO_MANY_REQUESTS,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        // Log 500s only — tower-http TraceLayer already records method/uri/status for all
        // requests. 4xx are expected client errors; logging them here would be noise.
        // Internal errors need the anyhow chain logged so the root cause is traceable.
        if let Self::Internal(ref e) = self {
            tracing::error!(error = %e, kind = "INTERNAL", "internal error");
        }
        let body = serde_json::json!({
            "kind": self.kind(),
            "message": self.to_string(),
        });
        (status, axum::Json(body)).into_response()
    }
}
```

### AppError (same pattern — generic kinds + same logging rule)

`AppError::Internal` also gets `tracing::error!(error = %e, kind = "INTERNAL", "internal error")`.
All other `AppError` variants (4xx) produce no log — same rationale as above.

---

## Files to Create / Modify

### Error implementation

| File | Action |
|------|--------|
| `.claude/docs/error-kinds.md` | **Create** — full standard (format, naming rules, kind table per service) |
| `.claude/docs/code-conventions.md` | Add "Error Kinds" section + reference to `error-kinds.md` |
| `crates/madome-core/src/error.rs` | Update `IntoResponse` → JSON; add `kind()` method |
| `services/auth/src/error.rs` | Refactor variants (domain-specific) + `kind()` + JSON `IntoResponse` |
| `services/auth/src/usecase/authcode.rs` | `NotFound` → `UserNotFound`, `TooManyRequests` → `TooManyAuthcodes` |
| `services/auth/src/usecase/token.rs` | user `NotFound` → `UserNotFound`; authcode `NotFound` → `InvalidAuthcode`; `Unauthorized` → `InvalidRefreshToken` |
| `services/auth/src/usecase/passkey.rs` | `NotFound` → `UserNotFound`/`CredentialNotFound`, `Unauthorized` → `InvalidSession`, `BadRequest` → `InvalidCredential` |
| `services/auth/src/handlers/token.rs` | `Unauthorized` → `InvalidToken` |
| `services/auth/src/handlers/passkeys.rs` | `BadRequest` → `InvalidCredential` |

### Contract harness — add body assertion support

Currently `Expect` only has `status` + `headers`; body is never read. Three files need changes:

| File | Change |
|------|--------|
| `tools/contract-harness/src/fixture.rs` | Add `body: Option<serde_json::Value>` to `Expect` with `#[serde(default)]` (backwards-compatible) |
| `tools/contract-harness/src/runner.rs` | Read response body as text; deserialize as JSON; compare with `expect.body` if Some (exact match). Add `body_mismatch: Option<String>` to `RunResult`. |
| `tools/contract-harness/src/reporter.rs` | Include `body_mismatch` in FAIL output and in `passed()` check. |

### Contract fixtures — add body assertions (auth service only)

"Do not modify existing fixtures" means don't change **existing assertions** (status codes stay
the same). Adding a new `body` field is an additive augmentation — no existing assertion changes.

4 of 6 auth fixtures go through `AuthServiceError::into_response` → get body assertions.
The other 2 (`delete_token_pair_no_auth`, `get_passkeys_no_auth`) use `IdentityHeaders`
rejection (`type Rejection = StatusCode`) → axum returns empty body → no body assertion added.

**Fixture design rule**: assert everything that's part of the API contract; omit what's irrelevant.

| Fixture | Error path | Headers | Body |
|---------|-----------|---------|------|
| `create_authcode_unknown_email.json` | `UserNotFound` | `content-type: application/json` | `{"kind":"USER_NOT_FOUND","message":"user not found"}` |
| `check_access_token_no_auth.json` | `InvalidToken` (token.rs:54) | `content-type: application/json` | `{"kind":"INVALID_TOKEN","message":"invalid token"}` |
| `create_token_pair_unknown_email.json` | `UserNotFound` | `content-type: application/json` | `{"kind":"USER_NOT_FOUND","message":"user not found"}` |
| `refresh_token_pair_no_auth.json` | `InvalidRefreshToken` (token.rs:123) | `content-type: application/json` | `{"kind":"INVALID_REFRESH_TOKEN","message":"invalid refresh token"}` |
| `delete_token_pair_no_auth.json` | `IdentityHeaders` (`StatusCode`) | — (empty body, no Content-Type) | — |
| `get_passkeys_no_auth.json` | `IdentityHeaders` (`StatusCode`) | — | — |

- `axum::Json(body)` sets `Content-Type: application/json` — part of the contract for JSON bodies.
- `StatusCode` rejection returns empty body, no Content-Type — nothing to assert.
- Library/users fixtures unchanged — those services don't have error kinds yet.
- Future success fixtures: 204 No Content → status only; 200/201 with body → add body + content-type; cookies → add Set-Cookie header assertions.

---

## Tests to Add / Modify

### New (inline `#[cfg(test)]`):
- `crates/madome-core/src/error.rs` — add async tests verifying JSON body + `kind` for each `AppError` variant.
  Both crates already have `tokio = { features = ["macros"] }` in `[dev-dependencies]` → `#[tokio::test]` works.
  Pattern: call `into_response()`, read body with `axum::body::to_bytes`, parse as `serde_json::Value`.
- `services/auth/src/error.rs` — same pattern for each `AuthServiceError` variant.

### Existing tests:
- `crates/madome-core/src/error.rs` lines 37-66: five status-code-only tests. **Keep as-is** — status codes
  are unchanged, tests still pass. Append new body tests after them.
- No existing test asserts on error response body format — verified by grep. No other changes needed.

---

## Docs to Update

- `.claude/docs/error-kinds.md` — created as part of this change (see above)
- `.claude/docs/code-conventions.md` — add "Error Kinds" section + reference to `error-kinds.md`
- `contracts/README.md` — in the Fixture Format section, add:
  - `expect.body` field description (JSON exact match, optional)
  - Fixture design guidance: assert everything relevant to the contract (status, content-type,
    body for JSON responses, Set-Cookie for cookie endpoints); omit irrelevant fields
  - Keep all existing "Do not modify" rules unchanged
- `.claude/docs/testing-philosophy.md` — in the "Cover all observable behaviors" section, add:
  - "One test per distinct error case" means one test per distinct **kind** (not just per HTTP status)
    — `UserNotFound` and `CredentialNotFound` are both 404 but are separate error cases requiring separate tests
  - Error path tests must assert the JSON body (`kind` + `message`) in addition to status code
  - Reference `error-kinds.md` for the list of kind values
  - Example: `assert_eq!(json["kind"], "USER_NOT_FOUND")` using `axum::body::to_bytes` in async test
- `tools/contract-harness/src/fixture.rs` — module-level `//!` comment currently says "the request to send and the expected response status"; update to "the request to send and the expected response (status, headers, and optionally body)". Also add `/// Expected response body (exact JSON match, optional).` doc comment to the new `body` field on `Expect`.
- No rustdoc on current error types — nothing to update there

---

## Verification

```bash
# 1. Format + lint + unit tests (body format assertions in error.rs)
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features

# 2. Contract tests — status + body assertions (requires Docker)
cargo run -p contract-harness --features auth
# Expected: all 6 auth fixtures pass; 4 of them also verify JSON body
```
