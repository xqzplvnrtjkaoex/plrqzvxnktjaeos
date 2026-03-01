# Plan: Disable squash merge — preserve commit history on all PRs

## Context

Squash merge was used for `feat/error-kind-standard → dev` (PR #17), collapsing all feature
branch commits into one. This loses individual commit history. The policy should be:
no squash merge allowed on any PR.

Current repo settings: `allow_squash_merge: true`, `allow_merge_commit: true`,
`allow_rebase_merge: true`.

---

## Changes

### 1. GitHub repo settings — disable squash merge

```bash
gh api --method PATCH repos/xqzplvnrtjkaoex/plrqzvxnktjaeos \
  --field allow_squash_merge=false
```

### 2. `.claude/docs/github-workflow.md` — update Merge strategy section

```diff
 ## Merge strategy

-- `dev → master`: **merge commit** (preserves dev commit history)
-- `feature → dev`: squash or merge — dev history is flexible
+- `dev → master`: **merge commit** — preserves full dev commit history in master
+- `feature → dev`: **merge commit** or **rebase** — squash is disabled
+- Squash merge is disabled on this repo; never use `gh pr merge --squash`
+  Use `gh pr merge --merge` (or `--rebase`) instead
 - Force push blocked on `master`
 - Delete short-lived feature branches after merge into `dev`
```

---

## Files touched

| File | Action |
|------|--------|
| GitHub repo settings | `allow_squash_merge` → `false` via `gh api PATCH` |
| `.claude/docs/github-workflow.md` | Update Merge strategy section |

---

## Verification

```bash
# Confirm setting took effect
gh api repos/xqzplvnrtjkaoex/plrqzvxnktjaeos --jq '.allow_squash_merge'
# Expected: false
```

GitHub PR 머지 UI에서 "Squash and merge" 버튼이 사라지면 완료.

---

## Tests / Docs

- 테스트 없음 (repo config 변경)
- `github-workflow.md` 만 업데이트
