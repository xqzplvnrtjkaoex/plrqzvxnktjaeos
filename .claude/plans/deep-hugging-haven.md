# Plan: Define explicit documentation review scope

## Context

CLAUDE.md §4.3 and §15 require checking "docs to add or modify" after every change,
but never define what "documentation" means. This causes incomplete reviews — only
obvious files (CLAUDE.md, README.md) get checked while rustdoc, `.claude/docs/`, and
service READMEs are missed.

The user's intent: identify which docs are **relevant to the change** (added/modified/removed
content), then check only those — not every doc every time. The scope covers `*.md` files
across the project, `#[utoipa::path]` handler annotations (external API reference), and
rustdoc in modified `.rs` files.

The doc inventory is placed in a dedicated `.claude/docs/doc-scope.md` file (following the
same pattern as `testing-philosophy.md`, `pr-guide.md`, etc.) so that new doc categories
can be added without touching CLAUDE.md.

---

## Changes

### 1. New file: `.claude/docs/doc-scope.md`

Create this file with the full scope inventory and scoping rule:

```markdown
# Documentation Scope

When checking "are there docs to update?", identify which of these categories are
**relevant to the change**, then check only those. Do not review every doc on every PR.

| Category | Where to look |
|----------|---------------|
| Root `*.md` | `CLAUDE.md`, `README.md`, `MIGRATION_PLAN.md` |
| Process docs | `.claude/docs/*.md` |
| Plan docs | `.claude/plans/*.md` (keep in sync with implementation) |
| Service/tool READMEs | `services/*/README.md`, `tools/*/README.md`, `contracts/README.md` |
| utoipa annotations | `#[utoipa::path]` attributes above handler functions in modified `.rs` files — merged by `tools/openapi/` into `dist/openapi/public.yaml` (public API reference for external developers) |
| Rustdoc in code | `///` and `//!` comments in modified `.rs` files; doc tests |

**Scoping rule**: A doc is relevant if the change affects what that doc describes — new
or changed behavior, endpoints, config, error messages, ops procedures, or type/function
semantics. Stale content in any relevant doc = PR blocked.

## Keeping this file current

When creating a new document (file, directory, or embedded annotation type) that does not
match any row in the table above, add a new row to the table as part of the same PR that
introduces the new document.
```

### 2. §4.3 Documentation gate — add doc-scope reference + DoD bullet

Insert a Required-read line before the DoD list, and add a docs bullet to the list:

```diff
+**Required before checking docs:** read `.claude/docs/doc-scope.md` — it defines which
+doc categories to check and the scoping rule. Do not skip the doc check without reading it first.
+
 A PR is not "done" until:

 - non-obvious behavior/ops changes are documented
 - runbooks/READMEs updated when relevant
 - rollback steps are recorded when the change affects prod safety
+- relevant docs updated (categories and scoping rule: see `.claude/docs/doc-scope.md`)
```

### 3. §15 Plans — tighten "Docs to add or modify" checklist

```diff
-2. **Docs to add or modify** — user-visible changes (new endpoints, new config options, changed error messages, etc.) require an update to the relevant documentation.
+2. **Docs to add or modify** — identify which docs defined in `.claude/docs/doc-scope.md`
+   are relevant to this change, then check only those. User-visible changes (new endpoints,
+   config options, error messages) require updates; stale rustdoc and utoipa annotations on
+   changed handlers must also be updated.
```

And tighten the simple-task rule:

```diff
-For simple tasks that don't go through planning, ask the user whether tests or documentation need to be updated after the work is done.
+For simple tasks that don't go through planning: after completing the work, identify which
+docs (per `.claude/docs/doc-scope.md`) are relevant to the change, check those for staleness,
+then report what (if anything) needs updating.
```

---

## Files touched

| File | Action |
|------|--------|
| `.claude/docs/doc-scope.md` | Create (new) |
| `CLAUDE.md` | §4.3 (add Required-read + DoD bullet), §15 (checklist item 2 + simple-task rule) |

## Verification

Read the updated §4.3 and §15 and confirm:
- `.claude/docs/doc-scope.md` is referenced by name
- §4.3 has a "Required before checking docs" line + a docs bullet in the DoD list
- §15 item 2 no longer enumerates paths inline — refers to doc-scope.md
- The simple-task rule no longer asks the user — it directs Claude to identify and check first
