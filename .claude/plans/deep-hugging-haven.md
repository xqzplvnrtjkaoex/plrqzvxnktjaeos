# Plan: Fix less-idiomatic code patterns

## Context

Code review found two less-idiomatic patterns in the existing codebase:

1. Double `unwrap_or` on `Option<Option<T>>` in taste handler — should use `.and_then()`
2. Auth service error tests missing `should_` prefix — violates `testing-philosophy.md`

---

## Changes

### 1. `services/users/src/handlers/taste.rs:106-111` — simplify double unwrap

```rust
// Before (lines 106-111):
let sort_by = query
    .sort_by
    .as_deref()
    .map(TasteSortBy::from_kebab)
    .unwrap_or(Some(TasteSortBy::default()))
    .unwrap_or_default();

// After:
let sort_by = query
    .sort_by
    .as_deref()
    .and_then(TasteSortBy::from_kebab)
    .unwrap_or_default();
```

### 2. `services/auth/src/error.rs` — rename 9 test functions

| Before | After |
|--------|-------|
| `user_not_found_json_body` | `should_return_user_not_found` |
| `credential_not_found_json_body` | `should_return_credential_not_found` |
| `invalid_authcode_json_body` | `should_return_invalid_authcode` |
| `invalid_token_json_body` | `should_return_invalid_token` |
| `invalid_refresh_token_json_body` | `should_return_invalid_refresh_token` |
| `invalid_session_json_body` | `should_return_invalid_session` |
| `invalid_credential_json_body` | `should_return_invalid_credential` |
| `too_many_authcodes_json_body` | `should_return_too_many_authcodes` |
| `internal_json_body` | `should_return_internal` |

---

## Files

| File | Action |
|------|--------|
| `services/users/src/handlers/taste.rs` | `.and_then()` instead of double `unwrap_or` |
| `services/auth/src/error.rs` | Rename 9 test functions to `should_` prefix |

---

## Verification

```bash
cargo test -p madome-users --all-features
cargo test -p madome-auth --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
