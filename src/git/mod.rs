/// Commit-record types and conventional-commit/PR detection.
pub mod commit;
/// Diff parsing, rendering, and token-budget truncation.
pub mod diff;
/// Parsing of `git log` and numstat porcelain output.
pub mod porcelain;
/// Helpers for invoking `git` subprocesses.
pub mod proc;
/// Parsing of `git reflog` output.
pub mod reflog;
