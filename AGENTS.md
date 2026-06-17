# AGENTS.md — instructions for AI coding agents working on this repo

> If you are an AI agent (Claude Code, OpenCode, Cursor, Codex, Copilot, Windsurf, …)
> reading this file, follow these instructions before doing anything else in this repo.

## What this is

`git-better` (binary `gb`) is a Rust CLI that wraps `git` for token-lean
human + LLM-agent workflows. It is **not** a git replacement — it shells out
to the system `git` and only intercepts the documented subcommands for
prettification and `--better` JSON output.

## Tech stack

- **Language**: Rust 2021 edition, MSRV 1.80 (for `std::sync::LazyLock`).
- **Platform**: **macOS arm64 (Apple Silicon) only** — this is a
  single-machine personal tool for the user. Release artifacts target
  `aarch64-apple-darwin`. CI runs on `ubuntu-latest` (cheaper; the
  binary is a static Rust artifact, no platform-specific syscalls).
  Do not add x86_64 macOS, Linux, Windows (`*-pc-windows-*`), or BSD
  targets without asking.
- **Git backend**: shell out to the `git` binary. **Never** add `libgit2` /
  `gitoxide` / `nodegit` — the user explicitly chose shell-out.
- **CLI parsing**: `clap` with the `derive` feature.
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
  output/
    mod.rs                 OutputMode enum
    theme.rs               palette + color enable detection
    layout.rs              column padding, box drawing, terminal width
    icons.rs               ●/◐/⇡/⇣/✨/🐛/📝 + ASCII fallback
    human.rs               pretty printer
    better.rs              JSON envelope serializer
    highlight.rs           syntect wrapper (lazy-init static)
  cli/
    mod.rs                 clap Cli enum + dispatch
    status.rs              (M0) byte-for-byte match with `git -c color.ui=false status -sb`
    diff.rs                (M1)
    log.rs                 (M1)
    show.rs                (M1)
    branch.rs              (M1)
    reflog.rs              (M1)
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
  fixtures/                tiny repos for integration tests (gitignored)
```

## Conventions

- **No comments in code unless asked.** The user is explicit about this.
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

## Verifying changes

```bash
cargo build --release
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

All four must pass before opening a PR. The release binary is targeted to
be **< 25 MB**; if a change pushes it over, flag it in the PR description.

## Out of scope (do not add without asking)

- `libgit2` / `gitoxide` / `nodegit`
- A TUI mode (full-screen curses — that's `lazygit` / `gitui` territory)
- Network calls inside `gb` itself (no LLM calls, no telemetry)
- MCP server (explicitly cut from v0.1)
- Conventional commit detection beyond the well-known 9 types
- x86_64 macOS, Linux, Windows, BSD targets (Apple Silicon only)
