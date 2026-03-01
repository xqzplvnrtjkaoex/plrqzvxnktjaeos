# Documentation Standards Reference

> Reference doc for CLAUDE.md §12. Read before writing public API docs, rustdoc comments,
> or any project documentation.

---

## Language and style rules

- Always respond in English and write documents in English.
- Before writing docs, see <https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing> and avoid these patterns.

---

## Documentation Standards

Ambiguous cases not covered here → discuss before deciding. Document less, but document accurately.

### External API docs (public-facing)

**Goal**: readable by an API consumer with zero internal context.

- Endpoint tables: method, path, auth requirement, status codes, one-line description.
- Request/response schemas: field names, types, constraints (e.g. `u32 ≥ 1`).
- Minimal prose — tables and code blocks only. No paragraphs.
- Never expose internal names: domain struct names, DB column names, gRPC service names.
- Location: `services/{name}/openapi/public.yaml`.

### Internal code docs — `cargo doc` is the primary tool

**Command**: `cargo doc --document-private-items --open`

| Target                     | Rule                                                                                 |
| -------------------------- | ------------------------------------------------------------------------------------ |
| `lib.rs` / `mod.rs` `//!`  | Required for every non-trivial module. Cover: what it owns, key invariants, gotchas. |
| Public items in `domain/`  | `///` required on all public types and trait methods.                                |
| Public items in `usecase/` | `///` required on `execute()`. I/O struct fields only if non-obvious.                |
| `infra/db.rs`              | `///` on repo impl methods if behavior differs from trait doc.                       |
| `handlers/`                | Comment only non-obvious extraction logic.                                           |

**Doc tests required for:**

- All cookie builder functions in `crates/madome-auth-types`
- `validate_access_token()` and `IdentityHeaders` extractor in `crates/madome-auth-types`
- Any function in `crates/madome-core` that has non-obvious behavior

**What NOT to document**: obvious getters/setters, one-liners whose name is self-explaining, re-exports.

### `docs/` — only for what rustdoc cannot express

Create a file under `docs/` only when the topic genuinely spans multiple services or is
operational (not code). Topics driven by need, not a fixed list.

Typical candidates:

- Multi-service topology + ingress routing + gRPC interface map (Mermaid `graph LR`)
- Operational runbooks: k8s apply, SOPS decrypt, migration steps, rollback

**Use Mermaid** for all diagrams (renders in GitHub natively).
**Do not create files preemptively** — write when the implementation is complete.

### Timing rules

| Doc type                    | When to write                                      |
| --------------------------- | -------------------------------------------------- |
| `//!` module docs           | Same PR as the module                              |
| `///` item docs             | Same PR as the item                                |
| Doc tests                   | Same PR as the function                            |
| `openapi/public.yaml`       | Same PR as the service implementation              |
| `docs/` files               | After the implementation they describe is complete |
| `services/{name}/README.md` | Same PR as the service implementation              |

**Update rule**: when behavior changes, update the doc in the same PR. Stale docs = failing review.
