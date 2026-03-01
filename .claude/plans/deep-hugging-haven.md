# Plan: Fix `am` naming + convert history upsert to ON CONFLICT

## Context

Two issues in `services/users/src/infra/db.rs`:

1. **`am` abbreviation**: 4 occurrences of `let mut am = row.into_active_model();` violate the
   naming convention. Variable should be named after the schema module (singular form).

2. **History upsert is two queries**: `upsert()` does SELECT + match (INSERT or UPDATE) — this is
   a race-condition-prone pattern and unnecessarily verbose. sea-orm supports
   `INSERT ... ON CONFLICT DO UPDATE` via `OnConflict`, collapsing it to one atomic query.

**Not converting taste/fcm upserts**: `upsert_book` and `upsert_book_tag` return `bool`
(whether the value changed) with a `row.is_dislike == taste.is_dislike` guard. `fcm_token::upsert`
has a `user_id` ownership guard. Both have business logic that doesn't map cleanly to ON CONFLICT.

---

## Changes

### 1. Rename `am` → entity-derived name (3 occurrences)

Name after the schema module (singular form), not the generic type:

| Line | Module | `am` → |
|------|--------|--------|
| 294 | `taste_books` | `taste_book` |
| 327 | `taste_book_tags` | `taste_book_tag` |
| 670 | `fcm_tokens` | `fcm_token` |

Line 455 (`history_books`) is removed entirely by step 2.

### 2. Rewrite history `upsert` to single ON CONFLICT query

**Before** (lines 447–474):
```rust
async fn upsert(&self, history: &HistoryBook) -> Result<(), UsersServiceError> {
    let existing = history_books::Entity::find_by_id((history.user_id, history.book_id))
        .one(&self.db)
        .await
        .context("find history book for upsert")?;
    match existing {
        Some(row) => {
            let mut am = row.into_active_model();
            am.page = Set(history.page);
            am.updated_at = Set(Utc::now());
            am.update(&self.db).await.context("update history book")?;
        }
        None => {
            history_books::ActiveModel { ... }
                .insert(&self.db).await.context("insert history book")?;
        }
    }
    Ok(())
}
```

**After:**
```rust
async fn upsert(&self, history: &HistoryBook) -> Result<(), UsersServiceError> {
    let history_book = history_books::ActiveModel {
        user_id: Set(history.user_id),
        book_id: Set(history.book_id),
        page: Set(history.page),
        created_at: Set(history.created_at),
        updated_at: Set(history.updated_at),
    };
    history_books::Entity::insert(history_book)
        .on_conflict(
            OnConflict::columns([
                history_books::Column::UserId,
                history_books::Column::BookId,
            ])
            .update_columns([
                history_books::Column::Page,
                history_books::Column::UpdatedAt,
            ])
            .to_owned(),
        )
        .exec_without_returning(&self.db)
        .await
        .context("upsert history book")?;
    Ok(())
}
```

### 3. Add import

Add `sea_query::OnConflict` to the `sea_orm` import block (line 3).

### 4. `.claude/docs/code-conventions.md` — add sea-orm naming rule

Insert after the "Naming clarity" bullet (after line 20), before the `---`:

```markdown
- **sea-orm entity variables**: Name variables after the schema module in singular
  form, not after the generic type. Write `let mut taste_book = row.into_active_model();`,
  not `let mut active_model = ...` or `let mut am = ...`. The module name
  (`taste_books`, `fcm_tokens`, etc.) gives the entity name; use its singular form.
```

---

## Files

| File | Action |
|------|--------|
| `services/users/src/infra/db.rs` | Rename `am` (3 places) + rewrite history upsert + add import |
| `.claude/docs/code-conventions.md` | Add sea-orm entity variable naming rule |

---

## Verification

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```
