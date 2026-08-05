# AGENTS.md — instructions for AI coding agents working on this repo

> If you are an AI agent (Claude Code, OpenCode, Cursor, Codex, Copilot, Windsurf, …)
> reading this file, follow these instructions before doing anything else in this repo.

## What this is

`git-better` (binary `gb`) is a Rust CLI that wraps `git` for token-lean
human + LLM-agent workflows. It is **not** a git replacement — it shells out
to the system `git` and only intercepts the documented subcommands for
prettification and `--better` JSON output.

## Tech stack

- **Language**: Rust 2024 edition, MSRV 1.85 (for `std::sync::LazyLock`).
  `rust-toolchain.toml` pins a newer compiler for local dev/CI; `Cargo.toml`
  `rust-version` is the MSRV gate.
- **Platform**: **macOS arm64 (Apple Silicon) only** — this is a
  single-machine personal tool for the user. Release artifacts target
  `aarch64-apple-darwin`. CI runs on `ubuntu-latest` (cheaper; the
  binary is a static Rust artifact, no platform-specific syscalls).
  Do not add x86_64 macOS, Linux, Windows (`*-pc-windows-*`), or BSD
  targets without asking.
- **Git backend**: shell out to the `git` binary. **Never** add `libgit2` /
  `gitoxide` / `nodegit` — the user explicitly chose shell-out.
- **CLI parsing**: `clap` with the `derive` feature.
- **Timestamps**: `time` with the `formatting` feature, for the RFC 3339
  `generated_at` stamp on a convention profile. It is already in the graph via
  `syntect → plist`. There is **no** async runtime — `tokio` was declared in M0,
  never used, and removed in v1. Do not add it back.
- **Hashing**: the FNV-1a helper in `src/conventions/hash.rs`. Cache keys need
  determinism, not cryptography; do not add a hashing crate.
- **Syntax highlighting**: `syntect` with `default-fancy` features. Use the
  shared `SyntaxSet` / `ThemeSet` from `src/output/highlight.rs` — never
  re-construct them per call.
- **Colors**: `owo-colors` with NO_COLOR / TTY awareness. Centralized in
  `src/output/theme.rs`.
- **Icons**: `src/output/icons.rs`. Unicode + ASCII fallback, switch on
  `GB_ASCII=1`.

## Repo layout

```
src/
  main.rs                  clap entrypoint, dispatches to cli::run
  lib.rs                   pub mod re-exports
  error.rs                 GbError enum (thiserror)
  git/
    mod.rs                 pub mod proc
    proc.rs                shell-out wrapper, lockfile excludes, color override
    porcelain.rs           z-parser for `status -z`, `log -z`, `diff -z` (M1)
    commit.rs              CommitRecord, CommitGroup types (M1)
    diff.rs                FileStat, Hunk, BudgetDiff types (M1)
    reflog.rs              ReflogEntry (M1)
  conventions/
    mod.rs                 Profile + nested types, SCHEMA_VERSION (v1)
    detect.rs              git + filesystem convention detection (v1)
    cache.rs               freshness, atomic write, prose merge (v1)
    hash.rs                FNV-1a 64 digest (v1)
  output/
    mod.rs                 OutputMode enum
    theme.rs               palette + color enable detection
    layout.rs              column padding, box drawing, terminal width
    icons.rs               ●/◐/⇡/⇣/✨/🐛/📝 + ASCII fallback
    human.rs               pretty printer
    better.rs              JSON envelope serializer
    conventions_view.rs    five-line convention summary (v1)
    highlight.rs           syntect wrapper (lazy-init static)
  cli/
    mod.rs                 clap Cli enum + dispatch
    status.rs              (M0) byte-for-byte match with `git -c color.ui=false status -sb`
    diff.rs                (M1)
    log.rs                 (M1)
    show.rs                (M1)
    branch.rs              (M1)
    reflog.rs              (M1)
    conventions.rs         (v1) `gb conventions`
    skill.rs               (v1) `gb skill print|path|install`
release-plz.toml            release-plz bot config: version + CHANGELOG + tag (v1)
docs/
  release.md               release runbook + one-time setup (v1)
docs/toolu/
  specs/v1.md              accepted v1 design spec
  plans/2026-08-05-v1.md   executed v1 plan + ledger contract
tests/
  cli_status.rs            (M0)
  output_pretty.rs         (M0)
  output_no_color.rs       (M0)
  output_ascii.rs          (M0)
  cli_diff_*.rs            (M1)
  cli_log_*.rs             (M1)
  cli_show.rs              (M1)
  cli_branch.rs            (M1)
  cli_reflog.rs            (M1)
  cli_passthrough.rs       (M1)
  cli_plain_pipe.rs        (M1)
  cli_conventions.rs       (v1)
  conventions_detect_unit.rs (v1)
  cli_skill.rs             (v1)
  fixtures/                tiny repos for integration tests (gitignored)
```

## Conventions

- **No inline `//` commentary in Rust source unless asked.** Config files
  (`.toml`) may include brief comments. The user is explicit about this.
- **One `///` doc line per public item is required**, not optional: the
  PostToolUse quality gate rejects any edit that leaves a `pub` item
  undocumented, so touching an older file means documenting the public items it
  already had. Keep them to one concise line; the ban above is about explanatory
  `//` comments inside function bodies, not doc lines.
- **No emojis in source files or commits unless asked.** They appear in
  *output* (via `icons.rs`), not in source.
- **No CLAUDE.md / additional agent files** — `AGENTS.md` is the only one.
- **Public API stays minimal.** `lib.rs` only re-exports modules; each
  module owns its own visibility.
- **Errors**: prefer `anyhow::Result<T>` in `main` and CLI dispatch;
  use `thiserror`-derived enums in internal modules. Never `unwrap()` in
  non-test code.
- **Tests**: integration tests use `assert_cmd` + `tempfile`. Unit tests
  for parsers go in `#[cfg(test)] mod tests` blocks at the bottom of
  the file they test. **Never** test private functions across module
  boundaries — exercise through the public CLI surface.
- **Lockfile excludes** (used in `git diff` / `git log --stat` defaults):
  `:(exclude)*.lock`, `:(exclude)*-lock.json`, `:(exclude)*.lockb`,
  `:(exclude)*.sum`, `:(exclude)Cargo.lock`, `:(exclude)package-lock.json`,
  `:(exclude)bun.lock`, `:(exclude)pnpm-lock.yaml`, `:(exclude)yarn.lock`.
  Add new ones in `src/git/proc.rs` (single source of truth).
- **Convention cache**: `$GB_CACHE_DIR`, else `$XDG_CACHE_HOME/git-better`, else
  `~/.cache/git-better`, under `conventions/<fnv1a(repo_root)>.json`. Any test
  that runs `gb conventions` **must** set `GB_CACHE_DIR` to a temp dir. Cache
  reads and writes are best-effort: a corrupt, foreign, or unwritable cache
  degrades to compute-and-print and never fails the command.
- **Convention detection** recognizes 11 conventional-commit types
  (the 9 tagged ones plus `style` and `revert`) when deciding whether a
  repository *uses* conventional commits. That is format detection; the tag
  table in `output/icons.rs` stays at 9.
- **Versioning**: the CLI surface and the `--better` envelope (with
  `schema_version` on the convention profile) are the stable `1.x` contract. The
  Rust library API is internal and may change in any release.
- **Releases** follow the same pattern as `Falconiere/comemory` and push to the
  same tap, `Falconiere/homebrew-tap`: release-plz owns version + CHANGELOG +
  tag, `dist` owns build + GitHub Release + formula, and two hand-maintained
  workflows follow (`release-finalize.yml` smoke test, `crates-io.yml` publish).
  **Never hand-edit the version in `Cargo.toml`** — the bot does that.
  `.github/workflows/release.yml` is dist-generated except for two blocks marked
  `NOTE:` (the quality gate in `plan`, the App-token step in
  `publish-homebrew-formula`); re-apply both after any `dist init`/`dist generate`,
  and keep `allow-dirty = ["ci"]` so dist tolerates them. Runbook:
  `docs/release.md`.

## Verifying changes

```bash
cargo build --release
cargo nextest run                       # `cargo test` also works
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

All four must pass before opening a PR. The release binary is targeted to
be **< 25 MB**; if a change pushes it over, flag it in the PR description.

## Out of scope (do not add without asking)

- `libgit2` / `gitoxide` / `nodegit`
- A TUI mode (full-screen curses — that's `lazygit` / `gitui` territory)
- Network calls inside `gb` itself (no LLM calls, no telemetry). The single
  exception is the opt-in `gb conventions --with-remote`, which shells out to
  `gh pr list` with a 5-second timeout. It is off by default and must stay so.
- MCP server (explicitly cut from v0.1)
- Write ops (`commit` / `push` / `rebase` / …) with `--better`. They pass through
  to `git` today; a JSON envelope on a write returns almost no tokens. Deferred
  past v1.
- Conventional commit *tags* beyond the well-known 9 types (see the detection
  note above for the deliberate 11-type exception)
- x86_64 macOS, Linux, Windows, BSD targets (Apple Silicon only)
