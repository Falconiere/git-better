---
name: git-better
description: "Token-lean git. Use `gb` (status/diff/log/show/branch/reflog) instead of raw git for reads. Append `--better` for LLM-context JSON. Lean defaults; pass any git flag to override."
---

# git-better — Token-Lean Git Protocol

Raw git burns context two ways: bloated read output (`git diff` dumps
every hunk incl. lockfiles + color codes) and re-discovering repo
conventions on every commit. `gb` fixes both. **Always active.**

## Pillar 1 — lean reads

| Instead of | Use | Why |
| :-- | :-- | :-- |
| `git status` | `gb status` | `status -sb`, no color, pretty by default in a TTY |
| `git diff` | `gb diff` | `--stat` first, lockfiles excluded, pretty bar chart; `gb diff <path>` to drill in |
| `git diff --cached` | `gb diff --cached` | any arg forwards verbatim |
| `git log` | `gb log` | `--oneline -n 20` with conventional-commit type tags |
| `git show` | `gb show` | `show --stat HEAD` by default; pass a ref to forward |
| `git branch` | `gb branch` | current branch + ahead/behind + stale |
| `git reflog` | `gb reflog` | last 50 entries, aligned columns |

**Rule:** a *bare* `gb diff` / `gb show` applies the lean default; the
moment you pass a path or any git flag it forwards verbatim (color off).
`gb diff --full` / `gb show --full` force full hunks.

## Pillar 2 — LLM context with `--better`

Append `--better` to any read command to get a structured JSON envelope
suitable for direct LLM consumption:

```
gb status --better
gb diff  --better [--budget 1500]
gb log   --better
gb show HEAD --better
gb branch --better
gb reflog --better
```

The JSON envelope is:

```json
{ "ok": true, "command": "...", "data": {...}, "hints": [...], "meta": {...} }
```

- `data` is strictly typed per command (e.g. `log.data.groups[]`).
- `hints` are next-step suggestions the agent can act on without re-discovery.
- `meta` carries `duration_ms`, `bytes`, and an optional `budget`.
- `--budget N` truncates the patch payload to ~`N` approximate tokens
  (chars/4) and sets `data.truncated: true` plus `data.truncated_files[]`.

## Pass-through (do not break muscle memory)

Any subcommand that is **not** one of the documented first-class
read commands (`status` / `diff` / `log` / `show` / `branch` / `reflog`)
is forwarded to `git` verbatim with `color.ui=false` and the standard
lockfile excludes preserved. So `gb remote -v`, `gb bundle --version`,
`gb rerere`, `gb worktree list`, `gb tag --list`, `gb stash list`,
`gb merge --abort`, `gb rebase --continue`, `gb cherry-pick --abort`,
`gb clean -ndx` — all work.

## Output flags

- `--plain` — strip all colors / icons / box drawing; flat ASCII.
- `NO_COLOR=1` — same effect (de-facto standard).
- `GB_ASCII=1` — force ASCII icons (`[M]`, `[?]`, `feat:`) over Unicode
  (`●`, `◐`, `✨`).
- `--better` — switch to JSON envelope mode.

## Zero-setup raw-git fallbacks

If `gb` is not on `$PATH`, these are the exact `git` invocations
the binary runs under the hood:

```bash
git -c color.ui=false status -sb
git -c color.ui=false diff --stat -- . \
  ':(exclude)*.lock' ':(exclude)*-lock.json' ':(exclude)*.lockb' \
  ':(exclude)*.sum' ':(exclude)Cargo.lock' ':(exclude)package-lock.json' \
  ':(exclude)bun.lock' ':(exclude)pnpm-lock.yaml' ':(exclude)yarn.lock'
git -c color.ui=false log --oneline -n 20
git -c color.ui=false show --stat
```

## Lockfile excludes (single source of truth)

`*.lock`, `*-lock.json`, `*.lockb`, `*.sum`, `Cargo.lock`,
`package-lock.json`, `bun.lock`, `pnpm-lock.yaml`, `yarn.lock`.

## Standing instructions for the agent

- Prefer `gb` for read operations; only fall back to raw `git` if `gb`
  fails or if a subcommand is undocumented.
- When you need a token-budgeted view, use `--better --budget N`.
- Do not re-run `gb status` and `gb diff` and `gb log` back-to-back when
  one call (`gb status --better`) already covers all three for triage.
- If `gb` is not available, use the zero-setup raw-git fallbacks above;
  do not invent your own flags.
