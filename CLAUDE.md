# CLAUDE.md

Team Operating Rules for This Repository (Plan‑First, Zero‑Failure Gates)
Applies to: All Claude Code agents working on this monorepo.

---

## 0) Why This Document Exists (read once)

This file defines **non-obvious rules, gates, and failure modes** that prevent contract drift and unsafe merges.
It intentionally avoids repeating things you can rediscover by reading code.

---

## 1) Project Overview

**Madome** is a web-based comic reader focused on browsing, organizing, and reading large collections smoothly.

### What Madome is

- A comic/manga viewer website you access in a browser.
- Built to make finding a title and reading chapters fast and frictionless.

### Key characteristics

- **Library-first browsing:** Navigate titles, chapters, and pages in a structured way.
- **Reader experience:** Designed for comfortable long-form reading (quick page loading, intuitive navigation).
- **Search & discovery:** Helps users locate works efficiently within a large catalog.
- **Account features:** Supports user accounts and authentication for personalized usage.
- **Mirrored content workflow:** Can mirror content from external sources so the library stays up to date.

### Tech Stack

| Layer            | Technology                                            |
| ---------------- | ----------------------------------------------------- |
| Backend services | Rust (tokio + axum)                                   |
| Frontend         | TypeScript SPA (build-time env injection, no SSR)     |
| Database         | PostgreSQL (existing schema preserved through Compat) |
| Auth             | HttpOnly cookie-based (access token + refresh token)  |
| Notifications    | FCM + AWS SES (via outbox + worker)                   |
| Infra            | Kubernetes, FluxCD, SOPS + age secrets                |
| GitOps           | Kustomize overlays; Husako introduced post-Compat     |

### Monorepo Layout

```text
apps/        # deployable user-facing applications (SPA)
services/    # deployable backend services (gateway, auth, users, library, image, sync, worker)
crates/      # shared Rust library crates (Cargo workspace members)
             #   madome-domain     — domain types (UserId, BookId, BookKind, etc.)
             #   madome-auth-types — cookie builders, IdentityHeaders extractor, JWT validation
             #   madome-core       — shared utilities (error types, pagination, etc.)
             #   madome-testing    — TestApp + test fixtures (used only in #[cfg(test)])
packages/    # shared TypeScript packages
packages/proto/ # protobuf / RPC definitions (shared by Rust + TS)
tools/       # codegen, contract harness, CI scripts
k8s/         # GitOps (Flux + Kustomize), SOPS secrets
contracts/   # immutable external contract fixtures + golden outputs
docs/        # created as needed: multi-service topology, operational runbooks
             # code-level docs live in rustdoc (cargo doc --document-private-items)
```

### Current Phase: Compat

The project is in **Phase A (Compat)**: reimplementing the backend in this monorepo while keeping
external behavior **byte-for-byte identical** to the legacy system. The legacy system is the source
of truth for all contracts during this phase.

Exit criteria: contract tests pass in dev + prod; cookie contracts pass; gateway guardrails verified;
rollback plan rehearsed. See `MIGRATION_PLAN.md` for full phase scope.

**Legacy source**: `previous/` — read-only reference for all Compat behavior. Never modify files under `previous/`.

**`previous/` usage rule**: Read `previous/` only to verify external contracts (API response schemas, cookie attributes, status codes, behavioral rules like renewal call order). The new implementation's internal structure follows the architecture in the active plan — do not mirror or copy legacy internal patterns.

| New service / crate | Legacy source                  |
| ------------------- | ------------------------------ |
| `services/auth`     | `previous/auth-madome-app/`    |
| `services/users`    | `previous/user-madome-app/`    |
| `services/library`  | `previous/library-madome-app/` |
| `services/image`    | `previous/image-madome-app/`   |
| `services/sync`     | `previous/sync/`               |
| `crates/madome-*`   | `previous/madome-sdk-rs/`      |
| `apps/` (SPA)       | `previous/madome-vue/`         |

---

## 2) Quick Reference Commands

**Rust services (all backend):**

```bash
# 1. Format
cargo fmt --all

# 2. Lint (zero warnings)
cargo clippy --workspace --all-targets --all-features -- -D warnings

# 3. Tests
cargo test --workspace --all-features
```

**TypeScript SPA:** See `apps/*/README.md` (framework TBD; commands added when scaffolded).

**CI workflows (GitHub Actions):**

Workflows are authored in TypeScript and compiled to YAML using
[gaji](https://github.com/dodok8/gaji). Do **not** hand-edit files under
`.github/workflows/` — edit the TypeScript source in `workflows/` and run `gaji build`.

```bash
gaji build        # compile workflows/ → .github/workflows/
gaji dev --watch  # watch mode during development
```

Commit both the `.ts` source and the generated YAML together.

**Verification rule:** Whenever claiming that implementation is complete or tests pass, always run both lint and tests and confirm both are clean. Do not skip lint during verification.

---

## 3) Code Conventions

- **Rust edition**: 2024 (`rust-version = "1.85"`)

Before writing code, read `.claude/docs/code-conventions.md` —
it covers all naming rules, test placement, and async patterns used in this codebase.

### Service Architecture (Clean Architecture)

Every backend service follows `handlers → usecase → domain → infra`:

- `handlers/`: HTTP only (extract params → call usecase → return response). No business logic.
- `usecase/`: Business logic. No axum, sea-orm, reqwest imports.
- `domain/`: Pure types + repository traits. No framework dependencies.
- `infra/db.rs`: sea-orm impl. Converts entity `Model` → domain type before returning.
- `infra/grpc.rs`: tonic gRPC clients for calling other services.
- `grpc_server.rs`: tonic service impl exposing this service's gRPC interface (omit if none).
- `schema/`: sea-orm entity definitions (DeriveEntityModel), shared by service + migration.
- `migration/`: sea-orm-migration binary. Depends on `schema/`.

Internal communication: **gRPC (tonic)** only — no inter-service HTTP calls.
Proto definitions: `packages/proto/` — shared source for all services.

#### UseCase struct pattern

UseCase is always a struct wrapping its dependencies as fields.
Shared as `Arc<UseCase>` in axum State; swapped for mocks in tests.
Use generics (`<R: Trait>`) for static dispatch — not `Box<dyn Trait>` unless runtime selection is needed.

Non-trivial use cases define `XxxInput` / `XxxOutput` types in the same file.
Handler only sees I/O types — never domain internals it doesn't need.

#### Dependency rule (hard enforcement)

| Layer       | May import from                         | Must NOT import               |
| ----------- | --------------------------------------- | ----------------------------- |
| `domain/`   | `crates/madome-domain`, std             | Everything else               |
| `usecase/`  | `domain/`, `crates/madome-*`            | axum, sea-orm, reqwest, tonic |
| `handlers/` | `usecase/` I/O types, axum extractors   | sea-orm, domain internals     |
| `infra/`    | `domain/` traits, sea-orm, tonic, redis | axum, `usecase/`              |

Violation = PR blocked. Ambiguous case → discuss before implementing.

---

## 4) Hard Gates (No Exceptions)

### 4.1 Plan gate

- Every task begins with a **Plan**.
- **Leader review + approval is required before implementation.**
- If the plan changes materially, re-submit the revised plan for approval.

**Definition of "implementation"** (not allowed pre-approval):

- writing/modifying production code
- changing k8s manifests
- changing contract fixtures
- changing CI config
- changing gateway routing

Allowed pre-approval:

- reading code
- collecting evidence
- drafting the plan
- proposing tests (but do not merge)

### 4.2 Test gate: 0 failures (including flakes and skips)

For every PR:

- Unit tests: required
- Integration tests: required
- E2E-smoke tests: required
- **0 failing tests**
- **0 skipped tests**
- **0 flaky tests** (flake = failure)

If any test is flaky:

- stop merging work that relies on it
- reproduce deterministically
- fix root cause
- remove quarantine immediately

E2E and smoke tests must be written as **Rust binaries** (e.g. `tools/e2e-smoke/`, `tools/contract-harness/`).
Shell scripts are not permitted for test harnesses — error messages are unreliable and debugging is painful.

Before writing any test, read `.claude/docs/testing-philosophy.md` for assertion patterns,
side-effect verification, auth test requirements, and test naming examples.

### 4.3 Documentation gate (DoD)

A PR is not "done" until:

- non-obvious behavior/ops changes are documented
- runbooks/READMEs updated when relevant
- rollback steps are recorded when the change affects prod safety

---

## 5) The Team Workflow (Strict Sequence)

Before writing a plan or opening a PR, read `.claude/docs/pr-guide.md` for the full
workflow sequence, plan template, PR checklist, work size rules, and interface file list.

For GitHub-specific mechanics (branching strategy, `gh` CLI commands, gaji CI setup,
review process, and merge strategy), read `.claude/docs/github-workflow.md`.

---

## 6) Work Size and Decomposition Rules

> See `.claude/docs/pr-guide.md`.

---

## 7) Interface Files (Extra Review Required)

> See `.claude/docs/pr-guide.md`.

---

## 8) Authorization and Permissions (No Workarounds)

Do not work around missing permissions. If GitHub permissions block your work,
read `.claude/docs/permissions.md` for the minimum-privilege rule and request template.

---

## 9) Repo Boundary Rules (Safety)

- Do not access or modify files outside the repository working directory.
- Do not introduce tooling that requires privileged execution unless approved in the plan.
- Do not commit secrets or log sensitive data (tokens, cookies, credentials).
- Do not paste production data into fixtures.
- **No actual hostnames in committed files**: Use `{API_HOST}` / `{FILE_HOST}` as placeholders in all docs and code comments. Actual values live only in k8s SOPS secrets and runtime env vars. In doc-test examples use `"example.com"` as a dummy domain.
- **Secrets must be SOPS+age encrypted** (`k8s/sops/`). Plaintext secrets must never appear in the repo, CI logs, or rendered manifests.
- **Flux MCP** is permitted for read status / debug / trigger reconcile only. All actual changes must go through Git PRs — no imperative `kubectl apply` or Flux overrides.

---

## 10) Branch Protection Recommendations (Project Defaults)

> See `.claude/docs/pr-guide.md`.

---

## 11) PR Template (Required Checklist)

> See `.claude/docs/pr-guide.md`.

---

## 12) Common Traps (Read Before Touching Sensitive Areas)

These are the failure modes most likely to silently break Compat.

**Auth contract immutability (Compat):** Cookie attributes (`Domain`, `Path`, `SameSite`, `Secure`,
`HttpOnly`, expiry semantics) must match legacy **exactly**. The legacy implementation is the source
of truth — read it before implementing. Do not infer from docs.

- **Cookie attributes drift** (Domain/Path/SameSite/Secure/HttpOnly ordering or defaults)
- **Gateway "helpfulness"** that mutates headers/cookies or rewrites paths
- **Assuming dev and prod behave the same** (domains differ; cookie scope differs)
- **Leaking secrets into logs** (cookies/tokens especially)
- **Testing shortcuts** (skips/quarantine becoming permanent)
- **Over-scoped PRs** that combine unrelated concerns

If you suspect one of these traps, stop and revise the plan.

---

## 13) Escalation Rules

Stop and ask (the user directly, or the Leader via `SendMessage` in team mode) if **any** of the
following occur — even mid-implementation:

- Implementation reveals something the plan didn't anticipate: unexpected code structure, missing
  types, conflicting constraints, or a design assumption that turns out to be wrong.
- Completing the task as specified would require an unstated behavioral assumption.
- The code path you're implementing contradicts what the plan described.
- Something feels off: unexpected complexity, missing context, or a decision that seems wrong.
- Contract tests disagree with your expectations.
- Any required behavior is ambiguous in legacy.
- A requirement conflicts (e.g., schema freeze vs required feature).
- You need additional permissions.

**Do not push through surprises with a best guess.** Stop, describe what you found, and ask.
The cost of pausing is seconds. The cost of a silent wrong assumption is a PR rollback.

No silent "best guesses" on contract-sensitive or plan-sensitive behavior.

---

## 14) Writing Docs

> See `.claude/docs/doc-standards.md` for language and style rules.

---

## 15) Plans

**Active Compat plan**: `.claude/plans/bubbly-puzzling-dewdrop.md` — this plan takes priority over `MIGRATION_PLAN.md` for all implementation decisions during the Compat phase.

When implementing non-trivial features, write a plan document first in `.claude/plans/`.

Every plan **must** include a section that explicitly identifies:

1. **Tests to add or modify** — new tests for the feature, and existing tests that may need updating (e.g. assertions that reference changed output, struct fields, or error messages).
2. **Docs to add or modify** — user-visible changes (new endpoints, new config options, changed error messages, etc.) require an update to the relevant documentation.

**In plan mode, always confirm both before calling ExitPlanMode:**

- "Are there tests to add or modify?" — check every time, without exception.
- "Are there docs to add or modify?" — check every time, without exception.

Do not treat these as optional — always verify both before finalising a plan.

For simple tasks that don't go through planning, ask the user whether tests or documentation need to be updated after the work is done.

`.claude/plans/*.md` files take precedence over `MIGRATION_PLAN.md`. If a plan file exists
for the current task, follow it. `MIGRATION_PLAN.md` is the long-horizon roadmap;
`.claude/plans/` files are the authoritative per-task specifications.

---

## 16) External Tool Usage Policy

Before adding or upgrading any crate, API, or CLI dependency,
read `.claude/docs/external-tools.md`.

---

## 17) Documentation Standards

Before writing public API docs or rustdoc comments,
read `.claude/docs/doc-standards.md`.

---

## 18) Team Agent Workflow

Form a **team of Claude Code agents** only when multiple independent tasks can run in parallel. Do not form a team by default.

### 18.1 Team structure

- **Team size**: maximum 4 members (excluding the Leader).
- **Leader**: the main Claude Code session — owns all planning, approves work, and is responsible for the final outcome.
- **Team member model**: `claude-sonnet-4-6` (Sonnet 4.6).

### 18.2 Division of responsibility

| Role     | Plans               | Implements            | Verifies                              |
| -------- | ------------------- | --------------------- | ------------------------------------- |
| Leader   | ✅ Writes all plans | ❌ Does not implement | ✅ Reviews all results                |
| Teammate | ❌ Does not plan    | ✅ Implements only    | ✅ Runs lint + tests before reporting |

**The Leader owns all planning. Teammates own all implementation.**
A teammate must never write or change a plan — if something in the plan is unclear or impossible,
the teammate reports back to the Leader, who revises the plan.

### 18.3 Workflow sequence

```
Leader                          Teammate(s)
  |                                  |
  |-- 1. Write plan ---------------→ |  (task description sent via TaskCreate + TaskUpdate)
  |                                  |
  |                                  |-- 2. Read task
  |                                  |-- 3. Implement (code only, no planning)
  |                                  |-- 4. Run: cargo fmt, clippy, test
  |                                  |-- 5. Report result via SendMessage
  |                                  |
  |←- 6. Receive result ------------ |
  |-- 7. Review + verify ----------→ |
  |-- 8. Accept or request changes   |
```

Steps 1–5 may run in parallel across multiple teammates for independent tasks.

### 18.4 Leader responsibilities

1. Explore the codebase and design the plan before spawning teammates.
2. Decompose the plan into concrete, implementation-ready tasks.
3. Spawn teammates via `TeamCreate` + `Agent` tools.
4. Assign tasks via `TaskUpdate` (set `owner` to teammate name).
5. **Verify every result** a teammate reports before accepting it as done.
6. Revise plans when teammates surface blockers or ambiguities.
7. Shut down the team gracefully with `SendMessage` (`shutdown_request`) when all work is done.
8. **Report the final outcome to the user** once all tasks are complete — summarize what was done, what files changed, test results, and any remaining caveats. The user is the final authority; the Leader's last act is always a clear, concise report to the user.

### 18.5 Teammate responsibilities

1. Read the assigned task description fully before writing any code.
2. Implement exactly what the task specifies — no scope expansion.
3. Run `cargo fmt --all`, `cargo clippy … -D warnings`, and `cargo test` before reporting.
4. Report results (what was done, what passed, any caveats) to the Leader via `SendMessage`.
5. If the task is unclear or a requirement conflicts with existing code, **stop and ask the Leader** — do not guess or self-plan.
6. After completing a task, check `TaskList` for additional unassigned, unblocked work and self-claim it with `TaskUpdate` (set `owner` to your name) — do not wait for the Leader to assign if work is available.

### 18.6 When to form a team

**Form a team when:**
- The task maps to 2+ independent subtasks that can proceed in parallel without file conflicts.
- A task requires different skill sets (e.g., Rust service + proto + migration running in parallel).
- The estimated diff is large enough that sequential solo work would be significantly slower.

**Do NOT form a team when:**
- The task is sequential — each step depends on the previous result.
- The task touches one file or one concern — solo work is faster.
- A subagent (`Agent` tool) can handle the delegation without peer coordination.

Team size limit: maximum 4 teammates.

**Subagent vs. team:**

| | `Agent` tool (subagent) | `TeamCreate` (team) |
|---|---|---|
| Peer messaging | No | Yes (`SendMessage`) |
| Shared task list | No | Yes (`TaskCreate` / `TaskList`) |
| Best for | Focused single task, research | 2+ parallel independent tasks |

### 18.7 No nested teams

Teammates must not spawn their own teams. If a sub-task requires further delegation, the teammate reports it as a blocker to the Leader, who handles the decomposition.

### 18.8 Task description format (Leader → Teammate)

Every task assigned to a teammate must include:

```
Goal:         <one sentence — what the task produces>
Files:        <explicit list of files to create or modify — no overlaps with other tasks>
Reference:    <existing files or patterns to follow>
Done when:    <concrete, verifiable completion criteria>
Constraints:  <layer/dependency rules to respect, contract invariants, etc.>
```

The Leader must ensure no two parallel tasks share the same file.

### 18.9 Teammate report format (Teammate → Leader)

When a teammate reports completion via `SendMessage`, the message must include:

```
Status:   done | blocked
Files:    <list of files created/modified>
Tests:    cargo fmt ✅ | cargo clippy ✅ | cargo test ✅  (or ❌ with error summary)
Notes:    <anything the Leader should know: edge cases, deferred items, surprises>
```

If `Status: blocked`, the teammate must describe exactly what is unclear or impossible,
and stop all implementation until the Leader responds.

---

## 19) Performance Tips

Before optimizing hot paths, read `.claude/docs/performance-tips.md` —
it collects patterns with measured impact specific to this codebase.
