# Code Conventions

- **Modules**: `snake_case`
- **Types/Structs**: `PascalCase`
- **Functions/Methods**: `snake_case`
- **Constants**: `UPPER_SNAKE_CASE`
- **Tests**: Inline `#[cfg(test)]` blocks in each module
- **Async**: Use async I/O wherever the calling context is already async.

---

## Env var naming — service prefix required

Service-specific env vars must carry a service prefix so that a shared `.env` at the
workspace root can set all services without collisions:

| Service | Correct      | Wrong  |
|---------|--------------|--------|
| auth    | `AUTH_PORT`  | `PORT` |
| users   | `USERS_PORT` | `PORT` |

Shared infrastructure vars (`DATABASE_URL`, `REDIS_URL`) need no prefix — each service uses
its own value anyway.

In the Rust config struct, the field name matches the env var in lowercase:

```rust
pub auth_port: u16,  // reads AUTH_PORT
```

---

## Error Kinds

Every service error type must expose a machine-readable `kind` string and return a JSON
body. See `.claude/docs/error-kinds.md` for the full standard, naming rules, and kind
tables per service.

Quick rules:
- `kind` is `UPPER_SNAKE_CASE`; name the thing that failed, not the HTTP status
- `message` is `lowercase` from the `#[error("...")]` attribute; human-readable only
- Log `tracing::error!` only for `Internal(_)` — 4xx are expected client errors

```rust
impl IntoResponse for MyServiceError {
    fn into_response(self) -> Response {
        let status = /* match status */;
        if let Self::Internal(ref e) = self {
            tracing::error!(error = %e, kind = "INTERNAL", "internal error");
        }
        let body = serde_json::json!({ "kind": self.kind(), "message": self.to_string() });
        (status, axum::Json(body)).into_response()
    }
}
```

---

## Env var reading — use std::env::var directly

Do not use the `envy` crate for loading config in new code — it is unmaintained.
Read env vars directly with `std::env::var()`:

```rust
let port: u16 = std::env::var("AUTH_PORT")
    .unwrap_or_else(|_| "3112".to_owned())
    .parse()
    .expect("AUTH_PORT must be a valid port number");
```

---

## DateTime serialization

All `DateTime<Utc>` in external JSON must use `#[serde(serialize_with = "madome_core::serde::to_rfc3339_ms")]`.
Output format: `"2023-02-11T11:09:00.000Z"` (RFC 3339, 3-digit millis, `Z` suffix).

---

## Query params: kebab-case

All query param names use kebab-case: `per-page`, `sort-by`, `is-dislike`, `book-ids[]`.
Use `serde(rename_all = "kebab-case")` on query structs.

---

## Pagination

Range and default for every pagination parameter (`per-page`, `page`, and nested variants like
`{noun}-per-page` / `{noun}-page`) must be explicitly decided per endpoint. Compat endpoints use
legacy values as-is. New endpoints require explicit discussion before implementation.

---

## Sort param format

`sort-by=created-at-desc` — direction embedded in kebab-case value. Parsed to
`SortBy::CreatedAt(Sort::Desc)`. All sort enums follow this pattern.

---

## Health endpoints

Every HTTP service must register `/healthz` and `/readyz` using
`madome_core::health::{healthz, readyz}`.

---

## Tagged enum serde

Polymorphic response/request types use `#[serde(tag = "kind", rename_all = "snake_case")]`.
The `kind` field discriminates variants. Applies to: Taste, History, Notification response models;
delete/create request bodies.

---

## gRPC error mapping

Domain errors map to tonic Status:
- `NotFound → NOT_FOUND`
- `AlreadyExists → ALREADY_EXISTS`
- `InvalidArgument → INVALID_ARGUMENT`
- `Internal → INTERNAL`

Use `tonic::Status::not_found(err.to_string())` etc.

---

## DELETE with JSON body

Polymorphic DELETE endpoints (taste, history) accept a JSON body with tagged enum
(`{"kind":"book","book_id":1}`), not path params. Simple single-resource DELETEs (passkeys) use
path params. This is legacy Compat.

---

## HTTP status codes (Compat)

Legacy-verified, all services consistent:
- `POST → 201`
- `GET → 200`
- `PATCH → 200` (with body) or `204` (no body)
- `DELETE → 204`

Revisit after Stabilize when API is redesigned.
