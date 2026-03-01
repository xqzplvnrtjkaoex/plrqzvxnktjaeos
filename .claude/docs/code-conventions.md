# Code Conventions

- **Modules**: `snake_case`
- **Types/Structs**: `PascalCase`
- **Functions/Methods**: `snake_case`
- **Constants**: `UPPER_SNAKE_CASE`
- **Tests**: Inline `#[cfg(test)]` blocks in each module
- **Async**: Use async I/O wherever the calling context is already async.
- **Generic bounds**: Use `where` clauses, not inline bounds.
  Write `impl<R> Foo<R> where R: Trait`, not `impl<R: Trait> Foo<R>`.
  Applies to both `struct` definitions and `impl` blocks.
- **Struct field access**: When a field name stutters with the variable name
  (e.g. `page.page`), destructure immediately instead of accessing fields.
  Write `let PageRequest { per_page, page } = page.clamped();`, not
  `let page = page.clamped(); ... page.page`.
- **Naming clarity**: Use full, descriptive names. Do not abbreviate unless the
  abbreviation is universally understood in Rust (`db`, `tx`, `err`, `ctx`, `req`,
  `resp`, `idx`). Single-letter variables are only acceptable as `_` (unused) or
  `i`/`j` in numeric loops. Closure params in `.map()` / `.filter()` chains must
  be readable: `|taste|` not `|t|`, `|model|` not `|m|`.
- **sea-orm entity variables**: Name variables after the schema module in singular
  form, not after the generic type. Write `let mut taste_book = row.into_active_model();`,
  not `let mut active_model = ...` or `let mut am = ...`. The module name
  (`taste_books`, `fcm_tokens`, etc.) gives the entity name; use its singular form.

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

## Domain trait naming and repository grouping

### Trait suffix rules

All traits in `domain/repository.rs` follow one of three naming suffixes:

| Suffix | Backend | When to use |
|--------|---------|-------------|
| `XxxRepository` | Database (sea-orm) | Standard CRUD for a single domain aggregate |
| `XxxPort` | Cross-aggregate DB transaction or external service (gRPC) | Operation that spans multiple aggregates or calls another service |
| `XxxCache` | Redis / in-memory | Ephemeral state with TTL (e.g., ceremony states) |

### Repository grouping — one trait per aggregate, not per table

The number of tables a repository owns depends on how the domain queries them:

| Pattern | When to use | Example |
|---------|-------------|---------|
| 1 trait : 1 table | Independent CRUD, no cross-table queries | `UserRepository` → `users` |
| 1 trait : N tables (union) | Combined list from sibling tables via UNION ALL | `TasteRepository` → `taste_books` + `taste_book_tags` |
| 1 trait : N tables (parent-child) | Parent + children always read/written together | `NotificationRepository` → `notification_books` + `notification_book_tags` |
| 1 trait : N tables (transaction) | Multi-table writes must be atomic | `AuthCodeRepository` → `auth_codes` + `outbox_events` |

**Decision rule:** If two tables appear in the same SQL query (JOIN, UNION ALL, or
transaction), they belong in one trait. If they are always queried independently, keep
them in separate traits.

### Port examples

| Port | Why it's a Port, not a Repository |
|------|-----------------------------------|
| `RenewBookPort` | Atomic operation spanning 3 aggregates (taste + history + notification tables) |
| `LibraryQueryPort` | Outbound gRPC call to another service — no local DB |

### Cache examples

| Cache | Backend | Key pattern |
|-------|---------|-------------|
| `PasskeyCache` | Redis | `passkey_reg:{user_id}:{reg_id}`, `passkey_auth:{email}:{auth_id}` — two ceremony kinds grouped in one trait |

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

Domain errors convert to `tonic::Status` via `From` trait impl. Use `.map_err(Status::from)`
at call sites.

| Domain error | tonic Status |
|---|---|
| `*NotFound` | `NOT_FOUND` |
| `*AlreadyExists` | `ALREADY_EXISTS` |
| `InvalidHandle`, `MissingData` | `INVALID_ARGUMENT` |
| `Forbidden` | `PERMISSION_DENIED` |
| `Internal` | `INTERNAL` |

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
