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

