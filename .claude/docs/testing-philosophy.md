# Testing Philosophy Reference

> Reference doc for CLAUDE.md §4.2. Read before writing any test.

---

## The standard

If all tests pass, no manual check is needed. A test suite that requires a human to additionally
"just verify it works" has failed its purpose.

---

## Assert on values, not just absence of errors

```rust
// WRONG — only proves "didn't crash"
let resp = app.post("/auth/token").json(&body).await;
assert_eq!(resp.status(), 201);

// RIGHT — proves the actual contract
assert_eq!(resp.status(), 201);
let cookies = resp.headers().get_all("set-cookie");
assert_cookie_attr(cookies, "madome_access_token", "Path=/");
assert_cookie_attr(cookies, "madome_access_token", "Max-Age=604800");
assert_cookie_attr(cookies, "madome_access_token", "HttpOnly");
assert_cookie_attr(cookies, "madome_access_token", "SameSite=Lax");
assert!(resp.headers().contains_key("x-madome-access-token-expires"));
```

---

## Cover all observable behaviors

For every function/handler:

- Happy path: correct input → correct output (assert the full output, not a subset)
- Each error path: one test per distinct error case (404, 401, 403, 409, etc.)
- Boundary conditions: min/max values, empty lists, optional fields absent

---

## Verify side effects, not just response codes

State-changing operations must assert that the side effect actually occurred:

```rust
// WRONG — only proves response was 201, not that the outbox event was inserted
let resp = app.post("/books").json(&book_data).await;
assert_eq!(resp.status(), 201);

// RIGHT — also proves the outbox event was created for downstream processing
assert_eq!(resp.status(), 201);
let event = db.query_one("SELECT * FROM outbox_events WHERE kind = 'book_added'").await?;
assert_eq!(event.kind, "book_added");
assert!(event.processed_at.is_none()); // pending, not yet processed by worker
```

Concrete side effects to assert per layer:

- **Handlers**: response headers, cookies set/cleared attribute-by-attribute
- **Write use cases**: DB row inserted/updated — query it back and assert fields
- **Outbox events**: row in `outbox_events` with correct `kind`, `payload`, `idempotency_key`; `processed_at IS NULL`
- **Worker jobs**: mock external client (FCM, SMTP) received correct request with correct arguments
- **Cache writes**: key exists in Redis with correct value and TTL
- **gRPC calls**: mock gRPC server recorded the correct request message

Rule: if a use case has a side effect, there is a test that asserts it happened.

---

## Auth and role tests are not optional

Every auth-gated route must have:

- Test with valid token → succeeds
- Test with no token → 401
- Test with insufficient role → 403
- Test with expired/invalid token → 401

---

## Integration tests use real HTTP (TestApp), not mocked handlers

Unit test the use case logic; integration test the full HTTP path.
Do not mock the handler itself — that tests nothing real.

---

## Test naming: `should_` prefix, describe the contract

```rust
// WRONG
fn test_get_book() { }

// RIGHT — reads as a specification; failure message is self-explaining
fn should_return_404_when_book_does_not_exist() { }
fn should_return_401_when_token_missing() { }
fn should_set_both_cookies_with_max_age_604800_on_login() { }
```

Rule: all `#[test]` functions must start with `should_`. No exceptions.

**If a behavior has no test, it does not exist.**
