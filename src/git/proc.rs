use crate::error::GbError;
use std::process::{Command, Stdio};

pub const NOCOLOR: &[&str] = &["-c", "color.ui=false"];

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
  Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

pub fn run_git_with_excludes(args: &[String]) -> Result<String, GbError> {
  let mut full: Vec<String> = Vec::with_capacity(args.len() + LOCKFILE_EXCLUDES.len() + 1);
  full.extend(args.iter().cloned());
  if !full.iter().any(|a| a == "--") {
    full.push("--".to_string());
  }
  full.extend(LOCKFILE_EXCLUDES.iter().map(|s| s.to_string()));
  run_git(&full)
}

pub fn write_to_stdout(text: &str) {
  use std::io::Write;
  let stdout = std::io::stdout();
  let mut handle = stdout.lock();
  let _ = handle.write_all(text.as_bytes());
}
