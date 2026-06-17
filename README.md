# git-better

> Token-lean git companion for humans and LLM agents. **macOS only.**

`gb` is a drop-in `git` wrapper that:

- **Humans** get prettified, TTY-aware, lockfile-aware read commands out of the box
  (`gb status`, `gb diff --stat`, `gb log`, `gb show`, `gb branch`, `gb reflog`).
- **LLM agents** append `--better` to get a token-budgeted JSON envelope
  enriched with branch intent, related PR, lockfile excludes, and conventional-commit grouping.

It is **not** a git replacement — unknown subcommands (`gb remote -v`,
`gb bundle --version`, `gb rerere`, …) forward verbatim to `git` with
`color.ui=false` and the standard lockfile path-excludes preserved.

## Install

```bash
cargo install git-better        # from crates.io (later)
# or
cargo install --path .           # from a local clone
```

## Usage

```bash
gb status          # pretty, no color codes when piped
gb status --plain  # flat ASCII (same as `git status -sb` w/o color)
gb status --better # JSON for LLM agents

gb diff            # --stat with lockfiles excluded; pretty bar chart
gb diff --full     # full hunks, syntax-highlighted
gb diff --better --budget 1500  # JSON, truncated to ~1500 tokens

gb log             # pretty oneline with conventional-commit type tags
gb log --story     # one-line "branch story"
gb log --better    # JSON grouped by PR trailer / conventional type

gb show HEAD       # syntax-highlighted diff for a commit
gb branch          # current branch + ahead/behind
gb reflog          # last 50 reflog entries

gb <any-git-subcommand> [args...]   # pass-through to git
```

## Conventions

- Colors auto-disable when stdout is **not a TTY** (e.g. piped), or when
  `NO_COLOR=1` is set.
- `GB_ASCII=1` forces ASCII icons (`[M]`, `[?]`, `feat:`) over Unicode
  (`●`, `◐`, `✨`).
- The full set of conventional-commit type tags is `feat / fix / docs /
  refactor / perf / test / chore / build / ci`. Unknown types render as
  `•`.
- Lockfiles excluded from `gb diff` by default: `*.lock`, `*-lock.json`,
  `*.lockb`, `*.sum`, `bun.lock`, `Cargo.lock`, `package-lock.json`,
  `pnpm-lock.yaml`, `yarn.lock`.

## License

MIT — see [LICENSE](LICENSE).
