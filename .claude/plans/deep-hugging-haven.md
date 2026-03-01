# Plan: Destructure `PageRequest` after `clamped()` + add convention

## Context

`page.page` and `page.per_page` reads awkwardly. After `let page = page.clamped();`, all
field access becomes `page.page` — the stuttering name makes code harder to scan. Destructuring
immediately (`let PageRequest { per_page, page } = page.clamped();`) yields clean `per_page`
and `page` locals. Add a convention rule and fix all 5 existing occurrences.

---

## Changes

### 1. `.claude/docs/code-conventions.md` — add destructuring rule

Insert after the "Generic bounds" bullet (line 11), before "Naming clarity":

```markdown
- **Struct field access**: When a field name stutters with the variable name
  (e.g. `page.page`), destructure immediately instead of accessing fields.
  Write `let PageRequest { per_page, page } = page.clamped();`, not
  `let page = page.clamped(); ... page.page`.
```

### 2. `services/users/src/infra/db.rs` — destructure in 5 list methods

Each occurrence follows the same mechanical change:

**Before:**
```rust
let page = page.clamped();
// ... later:
.offset(((page.page - 1) * page.per_page) as u64)
.limit(page.per_page as u64)
```

**After:**
```rust
let PageRequest { per_page, page } = page.clamped();
// ... later:
.offset(((page - 1) * per_page) as u64)
.limit(per_page as u64)
```

5 occurrences (line numbers approximate):

| Method | Line | Cast type |
|--------|------|-----------|
| `list_all` (raw SQL) | 111 | `as i64` (separate `offset`/`limit` vars) |
| `list_books` | 190 | `as u64` |
| `list_book_tags` | 218 | `as u64` |
| `list` (history) | 408 | `as u64` |
| `list` (notification) | 511 | `as u64` |

`list_all` is slightly different — it uses intermediate `offset`/`limit` variables:
```rust
// Before:
let page = page.clamped();
let offset = ((page.page - 1) * page.per_page) as i64;
let limit = page.per_page as i64;

// After:
let PageRequest { per_page, page } = page.clamped();
let offset = ((page - 1) * per_page) as i64;
let limit = per_page as i64;
```

---

## Files

| File | Action |
|------|--------|
| `.claude/docs/code-conventions.md` | Add destructuring convention |
| `services/users/src/infra/db.rs` | Destructure `PageRequest` in 5 methods |

---

## Verification

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```
