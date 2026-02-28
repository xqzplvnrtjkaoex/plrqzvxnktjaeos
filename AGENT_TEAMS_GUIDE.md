# Agent Teams Operations Guide

> Setup and usage reference for the Claude Code experimental agent teams feature.
> This doc is for the repository owner, not for agents.

---

## Setup

Agent teams are an **experimental feature** and must be explicitly enabled.

Add to `.claude/settings.json`:

```json
{
  "env": {
    "CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS": "1"
  }
}
```

---

## Token cost

Each teammate runs as a full independent Claude session. A 4-teammate team multiplies token
usage significantly. Only form a team when parallel work genuinely speeds things up.

---

## Known limitations

- **Session resumption**: `/resume` and `/rewind` do not restore in-process teammates.
  Complete all teammate tasks before restarting a session.
- **No nested teams**: teammates cannot spawn their own sub-teams (enforced by §18.7 in CLAUDE.md).
- **Maximum 4 teammates** (§18.1). For most tasks, 1–2 is sufficient.
