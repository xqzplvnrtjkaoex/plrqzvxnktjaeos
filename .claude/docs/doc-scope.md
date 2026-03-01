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
