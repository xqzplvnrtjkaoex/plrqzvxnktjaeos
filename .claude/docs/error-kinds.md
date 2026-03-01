# Error Kinds Standard

Machine-readable error kinds for JSON error responses.

## Response Format

```json
{
  "kind": "USER_NOT_FOUND",
  "message": "user not found"
}
```

- `kind` — `UPPER_SNAKE_CASE`; machine-readable; stable across releases
- `message` — `lowercase`; human-readable; may change between releases

**Do not expose internal error details** in `message` — especially for `INTERNAL` and
`INVALID_CREDENTIAL`.

## Naming Rules

| Rule | Example |
|------|---------|
| `UPPER_SNAKE_CASE` | `USER_NOT_FOUND` |
| Name the thing that failed, not the HTTP status | `INVALID_TOKEN` not `UNAUTHORIZED` |
| Generic fallback only when no specific kind fits | `INTERNAL`, `NOT_FOUND` |
| Each service defines its own domain kinds in `src/error.rs` | — |
| `INTERNAL` is the only kind shared across all services | — |

## AppError (`crates/madome-core`)

Used for middleware/infra-level errors where no domain context is available.

| Variant | Kind | HTTP |
|---------|------|------|
| `Unauthorized` | `UNAUTHORIZED` | 401 |
| `Forbidden` | `FORBIDDEN` | 403 |
| `NotFound` | `NOT_FOUND` | 404 |
| `Conflict` | `CONFLICT` | 409 |
| `Internal` | `INTERNAL` | 500 |

## AuthServiceError (`services/auth`)

| Variant | Kind | HTTP |
|---------|------|------|
| `UserNotFound` | `USER_NOT_FOUND` | 404 |
| `CredentialNotFound` | `CREDENTIAL_NOT_FOUND` | 404 |
| `InvalidAuthcode` | `INVALID_AUTHCODE` | 401 |
| `InvalidToken` | `INVALID_TOKEN` | 401 |
| `InvalidRefreshToken` | `INVALID_REFRESH_TOKEN` | 401 |
| `InvalidSession` | `INVALID_SESSION` | 401 |
| `InvalidCredential` | `INVALID_CREDENTIAL` | 400 |
| `TooManyAuthcodes` | `TOO_MANY_AUTHCODES` | 429 |
| `Internal` | `INTERNAL` | 500 |

Notes:
- `InvalidAuthcode` returns 401 (not 404) — prevents confirming whether a user exists.
- `InvalidCredential` (400) — do not expose WebAuthn internal error detail in `message`.

## Adding Kinds for a New Service

1. Create `services/{service}/src/error.rs` with a domain-specific error enum.
2. Implement `kind() -> &'static str` and `IntoResponse` (JSON body + status).
3. Log `tracing::error!` only for `Internal(_)` — 4xx are expected client errors.
4. Add the service's kind table to this file.
5. Add a section to `.claude/docs/code-conventions.md` if the service introduces new naming patterns.
