# Plan: Document domain trait naming and repository grouping convention

## Context

During Unit D (users service), the taste repository was initially split into two separate
traits (`TasteBookRepository` + `TasteBookTagRepository`), each backed by a single table.
This was wrong — legacy fetches both taste types in a single UNION ALL query, so splitting
the repository doubled query count for the combined list path. The fix was merging into a
single `TasteRepository` that owns both `taste_books` and `taste_book_tags` tables.

This incident exposed two undocumented conventions:

1. **Repository grouping** — when should a repository trait own one table vs. multiple?
2. **Domain trait naming** — the codebase uses three suffixes (`XxxRepository`, `XxxPort`,
   `XxxCache`) but no doc explains the distinction.

Both are implicit in the code today. Document them in `.claude/docs/code-conventions.md`.

---

## Changes

### `.claude/docs/code-conventions.md` — add "Domain trait naming" section

Insert after the "Error Kinds" section (after line 56, before the `---` + "Env var reading"
section). Content:

```markdown
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
```

---

## Files

| File | Action |
|------|--------|
| `.claude/docs/code-conventions.md` | Add "Domain trait naming and repository grouping" section after "Error Kinds" (line 56) |

---

## Verification

1. Read `.claude/docs/code-conventions.md` and confirm the new section is correctly placed.
2. Cross-check every example in the table against the actual code:
   - `services/users/src/domain/repository.rs` — UserRepository, TasteRepository,
     HistoryRepository, NotificationRepository, FcmTokenRepository, RenewBookPort,
     LibraryQueryPort
   - `services/auth/src/domain/repository.rs` — AuthCodeRepository, PasskeyRepository,
     PasskeyCache

---

## Tests / Docs

- No tests (documentation change only)
- `.claude/docs/code-conventions.md` is the target document
