# PR Guide Reference

> Reference doc for CLAUDE.md §5. Read before writing a plan or opening a PR.

---

## The only allowed workflow (§5)

> In the agent team model (§13), only the Leader follows steps 1–5.
> Teammates receive a ready-made task description and execute steps 6–9 only.

1. **Read**: inspect existing code, contract fixtures, and prior decisions.
2. **Plan**: write a small, testable plan (template below).
3. **Annotate**: list unknowns + how you will remove them (code reading, logs, fixtures).
4. **Revise**: incorporate discoveries; tighten gates.
5. **Leader approval**: mandatory checkpoint.
6. **Implement**: smallest diff that satisfies the plan.
7. **Test**: run unit + integration + e2e-smoke.
8. **Document**: update the relevant docs.
9. **PR**: open PR with checklist complete; no missing gates.

If you cannot complete steps 6–8 in a single PR, the plan is too large → split.

### Plan template (copy/paste)

Use this exact structure in the Issue/PR description:

- **Goal (single sentence):**
- **Non‑negotiables impacted:** (e.g., cookie contract, gateway pass-through, k8s/SOPS)
- **Files to read first:** (paths)
- **Contracts/tests to add first:** (unit/integration/e2e + contract fixtures)
- **Implementation steps (smallest possible):**
  1.
  2.
  3.
- **Verification gates:** (explicit commands / CI checks)
- **Rollback plan (if prod-affecting):**
- **Open questions / risks:** (and how to resolve)

---

## Work Size and Decomposition Rules

### "Small PR" rule

A PR must have:

- one primary outcome
- one owner
- one set of gates that clearly proves it is safe

Anti-patterns (must be split):

- gateway + multiple services + infra in one PR
- changing contract fixtures while also changing implementation without proving legacy mismatch
- "refactor + behavior change" combined

### How to split work (approved pattern)

Split by **gate**:

1. PR A: add failing tests/fixtures (proves mismatch)
2. PR B: implement minimal logic to pass
3. PR C: cleanup/refactor/docs

---

## Interface Files (Extra Review Required)

The following are "interfaces" and require explicit Leader review (and often CODEOWNERS):

- `contracts/**` — immutable golden fixtures:
  - `contracts/http/**` — request/response golden assertions (headers + bodies)
  - `contracts/cookies/**` — exact `Set-Cookie` strings per env (dev/prod)
  - `contracts/spa-routing/**` — SPA route parity assertions
  - `contracts/openapi/**` — frozen public OpenAPI spec (Compat)
- gateway routing config: `services/gateway/config/**`
- public OpenAPI specs: `services/*/openapi/public.yaml`
- internal RPC definitions: `packages/proto/**` (shared by Rust + TS)
- GitOps: `k8s/**` (especially `k8s/sops/**`)

Rule:

- If you touch an interface file, your Plan must call it out explicitly and include the additional tests that prove safety.

---

## Branch Protection Recommendations

The default branch should have (recommended):

- PRs required (no direct push)
- required status checks: unit + integration + e2e-smoke + lint/format
- required reviews (at least 1; more for interface files)
- force push blocked
- merge queue enabled (prevents "green on branch, red at merge")

If these settings are missing, file a permissions/settings request (see `.claude/docs/permissions.md`).

---

## PR Template (Required Checklist)

Every PR description must include the following sections:

### Summary

- **What changed:**
- **Why:**
- **Risk level:** (low/medium/high)
- **Rollback:** (none / steps / link to runbook)

### Plan and gates

- [ ] Plan link (Issue/Doc):
- [ ] Leader approval recorded (link/comment):

### Tests (must be explicit)

- [ ] Unit: <command or CI check name>
- [ ] Integration: <command or CI check name>
- [ ] E2E-smoke: <command or CI check name>
- [ ] Result: **0 failures, 0 skips, 0 flakes**

### Contracts and interfaces

- [ ] External contract unchanged (paths/schemas/status/SPA routes)
- [ ] Cookie behavior unchanged (if auth/cookies touched)
- [ ] Gateway invariants preserved (if gateway touched)
- [ ] Interface files touched? If yes, list them + extra review links

### Docs (DoD)

- [ ] README/runbook updated (link to file/section)
- [ ] Migration/ops docs updated (if applicable)

### Permissions

- [ ] Permission changes needed? If yes, link to request/issue
