# Plan: Add peer review guideline to CLAUDE.md §18

## Context

§18 currently covers only implementation-focused team workflows. Teammates can also be
spawned for **non-implementation work** — plan review, code review, and security audits —
where a second pair of eyes catches bugs, logic errors, and plan-vs-code discrepancies.
No guideline exists for this. Add §18.10.

---

## Changes

### `CLAUDE.md` — update §18.1 Team structure

Change the team member model line to reflect the model-per-role rule:

```diff
-- **Team member model**: `claude-sonnet-4-6` (Sonnet 4.6).
+- **Team member model**: `claude-sonnet-4-6` (Sonnet 4.6) for implementation;
+  `claude-opus-4-6` (Opus 4.6) for non-implementation work (review, audit). See §18.10.
```

### `CLAUDE.md` — add §18.10 Peer review with teammates

Insert after §18.9 (Teammate report format), before the `---` that separates §18 from §19:

```markdown
### 18.10 Peer review with teammates

Beyond implementation, spawn teammates for peer review in these situations:

| Situation | Purpose |
|-----------|---------|
| Plan review | Catch design flaws, missed edge cases, plan-vs-code discrepancies |
| Code review | Find logic bugs, security vulnerabilities, convention violations |
| Security audit | OWASP checks, secret exposure, injection vectors |

Review teammates should **discuss with each other** via `SendMessage` — not just report
independently to the Leader. The back-and-forth debate surfaces issues that a single
reviewer would miss.

**Not mandatory** — skip for simple changes. Use when the change is complex or high-risk.
When a full team is overkill, a single `Agent` tool reviewer subagent may suffice.
```

---

## Files touched

| File | Action |
|------|--------|
| `CLAUDE.md` | Update §18.1 (model-per-role rule) + add §18.10 after §18.9 |

---

## Verification

Read `CLAUDE.md` and confirm §18.10 is correctly placed between §18.9 and the `---` separator.

---

## Tests / Docs

- No tests (documentation change only)
- `CLAUDE.md` is the target document
