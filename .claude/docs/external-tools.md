# External Tool Usage Policy Reference

> Reference doc for CLAUDE.md §11. Read before adding or upgrading any crate, API, or CLI dependency.

---

## Cache paths

| Tool type    | Cache path                            |
| ------------ | ------------------------------------- |
| Rust crate   | `.claude/docs/crates/{crate-name}.md` |
| CLI tool     | `.claude/docs/tools/{tool-name}.md`   |
| External API | `.claude/docs/apis/{service-name}.md` |

---

## Version format in Cargo.toml

Always specify dependencies as `X.Y` (e.g., `tokio = "1.35"`), never just `X`.

- `X.Y` → Cargo.toml is the source of truth for the intended version.
- `cargo update` only bumps Cargo.lock — Cargo.toml stays unchanged → no cache invalidation.
- **"Update dependency X"** means: (1) edit version in `Cargo.toml`, then (2) run `cargo update`.
- Cargo.toml version change → cache mismatch detected → check changelog.

---

## Existing tool (already in Cargo.toml)

1. Read current version from `Cargo.toml` (in `X.Y` form).
2. Open cache doc. Compare the version recorded at the top.
   - **Match** → read doc, proceed. Done.
   - **Mismatch** (you upgraded it) → go to step 3.
3. Check the **changelog** for the range `cache_version → Cargo.toml_version`. Only the diff.
4. Update the cache doc: relevant new features/changes + bump the version number.

---

## New tool (not yet in Cargo.toml)

1. Check the latest stable version on crates.io / docs.rs.
2. Check the cache. (Usually a miss for new tools.)
3. Read official docs or use Context7.
4. Write the cache doc (see format below).
5. Add to `Cargo.toml`.

---

## Before implementing any new functionality

Check whether the **current Cargo.toml version** already provides it natively before
writing custom code. Use the cache doc or current version's API reference.

---

## Cache doc format

````markdown
# {crate-name} — v{version}

Changelog: {url}

## What we use (project-specific)

...

## Gotchas

...

## Example

\```rust
...
\```
````

Short enough to re-read in 30 seconds. Complete enough to proceed without opening official docs.
