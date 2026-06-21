use crate::error::GbError;
use std::process::{Command, Stdio};

/// Git arguments that disable colored output.
pub const NOCOLOR: &[&str] = &["-c", "color.ui=false"];

/// Pathspec excludes that hide lockfiles from diff and status output.
pub const LOCKFILE_EXCLUDES: &[&str] = &[
  ":(exclude)*.lock",
  ":(exclude)*-lock.json",
  ":(exclude)*.lockb",
  ":(exclude)*.sum",
  ":(exclude)Cargo.lock",
  ":(exclude)package-lock.json",
  ":(exclude)bun.lock",
  ":(exclude)pnpm-lock.yaml",
  ":(exclude)yarn.lock",
];

/// Runs `git` with the given arguments and returns captured stdout.
pub fn run_git(args: &[String]) -> Result<String, GbError> {
  let output = Command::new("git")
    .args(NOCOLOR)
    .args(args)
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .output()?;
  if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if stderr.contains("not a git repository") {
      return Err(GbError::NotARepository);
    }
    let code = output.status.code().unwrap_or(-1);
    return Err(GbError::GitFailed { code, stderr });
  }
  let stderr = String::from_utf8_lossy(&output.stderr);
  if !stderr.trim().is_empty() {
    eprint!("{stderr}");
  }
  Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Runs `git` with the given arguments plus lockfile pathspec excludes appended.
pub fn run_git_with_excludes(args: &[String]) -> Result<String, GbError> {
  let mut full: Vec<String> = Vec::with_capacity(args.len() + LOCKFILE_EXCLUDES.len() + 1);
  full.extend(args.iter().cloned());
  if !full.iter().any(|a| a == "--") {
    full.push("--".to_string());
  }
  full.extend(LOCKFILE_EXCLUDES.iter().map(|s| s.to_string()));
  run_git(&full)
}

/// Writes the given text to stdout, ignoring write errors.
pub fn write_to_stdout(text: &str) {
  use std::io::Write;
  let stdout = std::io::stdout();
  let mut handle = stdout.lock();
  let _ = handle.write_all(text.as_bytes());
}
