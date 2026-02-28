# GitHub Workflow Reference

> Reference doc for CLAUDE.md §5. Read before creating branches, opening PRs, or
> modifying CI workflows.

---

## Branching strategy

Two persistent branches:

| Branch | Purpose | Direct push |
|--------|---------|-------------|
| `master` | Stable — PR-merged units only | ❌ PR only |
| `dev` | Active development, WIP commits | ✅ allowed |

Short-lived feature branches (`feat/...`, `fix/...`) branch off `dev` for isolated work
and merge back into `dev` when ready.

```
dev  ──●──●──●──●──●──────────────▶  (direct commits + feature branch merges)

master ───────────────────────────PR merge commit──▶  (complete units only, PR required)
```

**When to open a PR to `master`:**

Open a PR when a planned unit is complete:

- All behaviors for the unit are implemented
- `cargo fmt`, `cargo clippy`, `cargo test` all clean
- Contract tests pass (for services)

What counts as a "unit" is defined by the plan for each task — the plan will explicitly
mark which steps produce a master PR. Rule of thumb: a unit is complete when it can be
independently tested and rolled back (e.g., all shared crates together, one full service).

---

## Branch naming (feature branches off dev)

```
<type>/<short-description>
```

Types: `feat`, `fix`, `chore`, `docs`, `refactor`
Description: kebab-case, ≤ 40 chars

Examples: `feat/madome-domain`, `fix/auth-cookie-path`, `chore/ci-setup`

---

## Commit messages

- One-line imperative subject, ≤ 72 chars
- Body optional; use for "why", not "what"
- Issue/ticket references (`closes #42`, `PROJ-123`, etc.) go in the **PR body**, not the
  commit subject

Examples: `add madome-domain crate`, `fix refresh token path attribute`

---

## GitHub Issues

Three types of issues:

| Type | When | Who opens |
|------|------|-----------|
| **Task** | Planning phase — one per PR unit | Claude (during planning) |
| **Escalation** | §13 condition: unexpected finding or decision needed | Claude (mid-implementation) |
| **Bug/blocker** | Unexpected failure or external blocker | Claude or user |

**Task issues:** When writing a plan that involves multiple PR units, Claude opens a GitHub
Issue per unit before implementation begins. Each issue title matches the planned PR scope.
PRs reference the issue with `closes #N` in the PR body.

**Escalation issues:** When implementation reveals something the plan didn't anticipate
(see §13 in CLAUDE.md), Claude opens an issue describing what was found, pauses work, and
waits for a response before continuing.

```bash
# Open a task issue
gh issue create \
  --title "shared crates: madome-domain, auth-types, core" \
  --body "Implement shared crates as per plan ..."

# Reference in PR body:
# closes #3
```

---

## Opening a PR to master

```bash
# Confirm dev is ready
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features

# Push dev (if not already up to date)
git push origin dev

# Open PR
gh pr create \
  --base master \
  --head dev \
  --title "auth service: Compat implementation" \
  --body "..."
```

PR body must follow the checklist structure in `pr-guide.md`.

---

## Draft PRs

Open a draft PR to track in-progress work without triggering merge:

```bash
gh pr create --draft --title "WIP: auth service" --head dev --base master --body "..."
```

Convert to ready when implementation is complete and CI is clean:

```bash
gh pr ready
```

---

## CI checks

Required before `master` merge (blocking):

| Job | Command | Phase |
|-----|---------|-------|
| `fmt` | `cargo fmt --all --check` | Now |
| `clippy` | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Now |
| `test` | `cargo test --workspace --all-features` | Now |
| `e2e` | `cargo run --bin e2e-smoke` (or equivalent) | Added when `tools/e2e-smoke` is implemented |

E2E tests will be added to the CI workflow (as a new gaji job) once `tools/e2e-smoke`
exists. When added, they become a required status check on `master` PRs.

On `dev` pushes: same checks run, but informational only — direct pushes are not blocked
by CI status. CI failure on `dev` should still be fixed before opening a master PR.

**Workflow source**: `workflows/` (TypeScript) → `gaji build` → `.github/workflows/` (YAML).
Do not hand-edit the generated YAML.

---

## Review process

- **Solo work**: CI gates replace peer review. Self-review the diff before merging.
- **Multi-contributor**: at least 1 reviewer approval required.
- Interface files (`contracts/`, `packages/proto/`, `k8s/`, gateway config): require
  Leader review regardless.

Address all review comments before merging. No unresolved threads.
If a reviewer requests changes: push a follow-up commit on the same branch — do not open a new PR.

---

## Merge strategy

- `dev → master`: **merge commit** (preserves dev commit history)
- `feature → dev`: squash or merge — dev history is flexible
- Force push blocked on `master`
- Delete short-lived feature branches after merge into `dev`

---

## When CI is red on a master PR

1. Fix on `dev` (or a fix branch off `dev`).
2. Re-run lint + tests locally.
3. Push to `dev` → CI re-runs automatically on the open PR.
