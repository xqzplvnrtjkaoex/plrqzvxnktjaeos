# Plan: Add `OrderByRandom` + enforce `where` clause convention

## Context

Two issues:

1. **`Random` sort no-op**: `TasteSortBy::Random` and `HistorySortBy::Random` exist but 3
   sea-orm query builder branches silently skip ordering (`=> query`). The raw SQL UNION ALL
   path works correctly. Need an `OrderByRandom` trait (reference: `syrflover/sea-extra`).

2. **Inline generic bounds**: All use case structs and impls use `impl<R: Trait>` instead of
   `where` clauses. Add convention and refactor all existing occurrences.

---

## Changes

### 1. `.claude/docs/code-conventions.md` — add `where` clause rule

Insert after the "Async" bullet (line 8), before "Naming clarity":

```markdown
- **Generic bounds**: Use `where` clauses, not inline bounds.
  Write `impl<R> Foo<R> where R: Trait`, not `impl<R: Trait> Foo<R>`.
  Applies to both `struct` definitions and `impl` blocks.
```

### 2. `crates/madome-core` — add `OrderByRandom` trait

**`Cargo.toml`** — add:
```toml
sea-orm = { workspace = true }
```

**`src/sea_ext.rs`** — new file:
```rust
use sea_orm::{
    sea_query::{Func, SimpleExpr},
    EntityTrait, Order, QueryOrder, Select,
};

pub trait OrderByRandom {
    fn order_by_random(self) -> Self;
}

impl<E> OrderByRandom for Select<E>
where
    E: EntityTrait,
{
    fn order_by_random(mut self) -> Self {
        QueryOrder::query(&mut self)
            .order_by_expr(SimpleExpr::FunctionCall(Func::random()), Order::Desc);
        self
    }
}
```

**`src/lib.rs`** — add `pub mod sea_ext;`

### 3. `services/users/src/infra/db.rs` — fix 3 no-op `Random` arms

Add import: `use madome_core::sea_ext::OrderByRandom;`

| Line | Before | After |
|------|--------|-------|
| 199 | `TasteSortBy::Random => query,` | `TasteSortBy::Random => query.order_by_random(),` |
| 230 | `TasteSortBy::Random => query,` | `TasteSortBy::Random => query.order_by_random(),` |
| 423 | `HistorySortBy::Random => query,` | `HistorySortBy::Random => query.order_by_random(),` |

### 4. Refactor inline bounds → `where` clauses

All `pub struct Xxx<R: Trait>` and `impl<R: Trait> Xxx<R>` become
`pub struct Xxx<R> where R: Trait` and `impl<R> Xxx<R> where R: Trait`.

**`services/users/src/usecase/taste.rs`** — 12 structs + 12 impls (24 changes)
**`services/users/src/usecase/history.rs`** — 4 structs + 4 impls
**`services/users/src/usecase/user.rs`** — 3 structs + 3 impls
**`services/users/src/usecase/notification.rs`** — 2 structs + 2 impls
**`services/users/src/usecase/fcm_token.rs`** — 1 struct + 1 impl
**`services/users/src/usecase/renew_book.rs`** — 1 struct + 1 impl
**`services/auth/src/usecase/authcode.rs`** — 1 struct + 1 impl
**`services/auth/src/usecase/token.rs`** — 2 structs + 2 impls
**`services/auth/src/usecase/passkey.rs`** — 6 structs + 6 impls

---

## Files

| File | Action |
|------|--------|
| `.claude/docs/code-conventions.md` | Add `where` clause rule |
| `crates/madome-core/Cargo.toml` | Add `sea-orm` dep |
| `crates/madome-core/src/sea_ext.rs` | New — `OrderByRandom` trait |
| `crates/madome-core/src/lib.rs` | Add `pub mod sea_ext;` |
| `services/users/src/infra/db.rs` | Import + fix 3 `Random` arms |
| `services/users/src/usecase/*.rs` | Inline bounds → `where` (23 structs + 23 impls) |
| `services/auth/src/usecase/*.rs` | Inline bounds → `where` (9 structs + 9 impls) |

---

## Verification

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```
