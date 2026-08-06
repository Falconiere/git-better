# git-better

> Token-lean git companion for humans and LLM agents. **macOS arm64 (Apple
> Silicon), Linux x86_64, Linux arm64.**

`gb` is a drop-in `git` wrapper that:

- **Humans** get prettified, TTY-aware, lockfile-aware read commands out of the box
  (`gb status`, `gb diff --stat`, `gb log`, `gb show`, `gb branch`, `gb reflog`).
- **LLM agents** append `--better` to get a token-budgeted JSON envelope
  enriched with branch intent, related PR, lockfile excludes, and conventional-commit grouping.
- **Both** get `gb conventions`: a cached profile of the repository's commit,
  branch, PR, and release style, so house conventions are read once instead of
  rediscovered on every commit.

It is **not** a git replacement — unknown subcommands (`gb remote -v`,
`gb bundle --version`, `gb rerere`, …) forward verbatim to `git` with
`color.ui=false` and the standard lockfile path-excludes preserved.

## Install

```bash
brew install Falconiere/tap/git-better   # Homebrew tap (macOS + Linuxbrew)
cargo install git-better                 # from crates.io
cargo install --path .                   # from a local clone
```

Or the shell installer, which detects the platform and fetches the prebuilt
binary:

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/Falconiere/git-better/releases/latest/download/git-better-installer.sh | sh
```

Each tagged release ships a `.tar.xz` per platform, each with a `.sha256`
sidecar, on the
[releases page](https://github.com/Falconiere/git-better/releases):

| Platform | Archive |
| --- | --- |
| macOS arm64 | `git-better-aarch64-apple-darwin.tar.xz` |
| Linux x86_64 | `git-better-x86_64-unknown-linux-gnu.tar.xz` |
| Linux arm64 | `git-better-aarch64-unknown-linux-gnu.tar.xz` |

The Linux builds are glibc, not musl; the shell installer checks the local glibc
against the build's floor and refuses cleanly rather than installing a binary
that cannot run. On an older distro, or on musl, or on an Intel Mac, use
`cargo install git-better`. Releases are cut by release-plz + dist — see
[docs/release.md](docs/release.md).

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

gb conventions             # five-line house-style summary (cached)
gb conventions --json      # raw profile JSON
gb conventions --better    # JSON envelope with next-step hints
gb conventions --refresh   # recompute, ignoring cache freshness

gb skill install           # write the protocol into detected agent config paths
gb skill print             # the protocol document on stdout

gb <any-git-subcommand> [args...]   # pass-through to git
```

## `gb conventions`

Infers the repository's conventions from declared files *and* git history —
most repos declare nothing, which is the point:

```
commit:  conventional-commits | scope none | suffix none
branch:  type/kebab [chore, feat]
pr:      template .github/PULL_REQUEST_TEMPLATE.md | sections [Summary, Tests]
release: [release.yml-workflow] | none
prose:   pending [CONTRIBUTING.md]
```

- The commit convention is decided by majority over the last 50 subjects, so the
  profile tracks current practice rather than ancient history.
- Profiles are cached at `$GB_CACHE_DIR`, else `$XDG_CACHE_HOME/git-better`,
  else `~/.cache/git-better` — under `conventions/<digest>.json`. A cached
  profile is reused until a declared convention file changes or it turns 7 days
  old; `--refresh` forces a recompute. A commit-style change alone does not
  invalidate the cache.
- `--with-remote` allows one bounded `gh pr list` lookup for recent PR titles.
  Without it, `gb` makes no network calls at all.
- Files listed under `prose:   pending` should be read once, distilled, and
  persisted so they are never re-read:

  ```bash
  printf '%s' "squash before merge; no merge commits" \
    | gb conventions --save-prose CONTRIBUTING.md
  ```

## `gb skill install`

Writes the embedded protocol document into the agent config paths it finds:

| Target | Path |
| :-- | :-- |
| `claude-user` | `$CLAUDE_CONFIG_DIR`/`~/.claude` → `skills/git-better/SKILL.md` |
| `claude-project` | `.claude/skills/git-better/SKILL.md` |
| `cursor` | `.cursor/rules/git-better.mdc` |
| `windsurf` | `.windsurf/rules/git-better.md` |
| `copilot` | `.github/copilot-instructions.md` (fenced block) |
| `codex` | `$CODEX_HOME`/`~/.codex` → `AGENTS.md` (fenced block) |
| `agents-md` | `AGENTS.md` (fenced block) |

By default only detected targets are written — a marker directory such as
`~/.claude` or `.cursor/`, or an already-present shared instruction file. `--all`
or an explicit `--target <name>` installs regardless and creates parents,
`--dry-run` reports without writing, and `--force` overwrites content that is
not ours. Fenced blocks are replaced in place, so re-installing never
duplicates.

## Conventions

- Colors auto-disable when stdout is **not a TTY** (e.g. piped), or when
  `NO_COLOR=1` is set.
- `GB_ASCII=1` forces ASCII icons (`[M]`, `[?]`, `feat:`) over Unicode
  (`●`, `◐`, `✨`).
- The full set of conventional-commit type tags is `feat / fix / docs /
  refactor / perf / test / chore / build / ci`. Unknown types render as
  `•`. `gb conventions` recognizes two more (`style`, `revert`) when deciding
  whether a repository *uses* conventional commits — that is format detection,
  not tagging.
- Lockfiles excluded from `gb diff` by default: `*.lock`, `*-lock.json`,
  `*.lockb`, `*.sum`, `bun.lock`, `Cargo.lock`, `package-lock.json`,
  `pnpm-lock.yaml`, `yarn.lock`.

## License

MIT — see [LICENSE](LICENSE).
