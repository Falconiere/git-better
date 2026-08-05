use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use assert_cmd::Command;
use serde_json::Value;
use tempfile::TempDir;

/// A `gb` command rooted in `repo` with its convention cache redirected to `cache`.
pub fn gb(repo: &Path, cache: &Path) -> Command {
  let mut cmd = Command::cargo_bin("gb").expect("the gb binary is not built");
  cmd.current_dir(repo).env("GB_CACHE_DIR", cache);
  cmd
}

/// A throwaway cache directory, so a test never touches the real one.
pub fn cache_dir() -> TempDir {
  tempfile::tempdir().expect("cannot create a temporary cache directory")
}

/// Runs `gb conventions --json` and returns the parsed profile.
pub fn profile_json(repo: &Path, cache: &Path) -> Value {
  let out = gb(repo, cache)
    .args(["conventions", "--json"])
    .output()
    .unwrap();
  assert!(out.status.success(), "gb conventions --json failed");
  serde_json::from_slice(&out.stdout).unwrap()
}

/// Runs `gb conventions --better` with extra arguments and returns the envelope.
pub fn better_envelope(repo: &Path, cache: &Path, extra: &[&str]) -> Value {
  let mut cmd = gb(repo, cache);
  cmd.args(["conventions", "--better"]).args(extra);
  let out = cmd.output().unwrap();
  assert!(out.status.success(), "gb conventions --better failed");
  serde_json::from_slice(&out.stdout).unwrap()
}

/// Returns the single cached profile under `cache`, asserting there is exactly one.
pub fn sole_cache_file(cache: &Path) -> PathBuf {
  let mut entries: Vec<PathBuf> = std::fs::read_dir(cache.join("conventions"))
    .unwrap()
    .filter_map(Result::ok)
    .map(|entry| entry.path())
    .collect();
  assert_eq!(entries.len(), 1, "expected one cached profile: {entries:?}");
  entries.pop().unwrap()
}

/// Returns a file's modification time.
pub fn mtime(path: &Path) -> SystemTime {
  std::fs::metadata(path).unwrap().modified().unwrap()
}

/// Backdates a file's modification time by `by`.
pub fn age_file(path: &Path, by: Duration) {
  let file = std::fs::File::options().write(true).open(path).unwrap();
  file.set_modified(SystemTime::now() - by).unwrap();
}

/// A PATH directory holding only `git`, so `gh` is guaranteed absent.
pub fn bin_dir_with_git_only() -> TempDir {
  let dir = tempfile::tempdir().expect("cannot create a temporary bin directory");
  let git = std::env::split_paths(&std::env::var_os("PATH").expect("PATH is unset"))
    .map(|entry| entry.join("git"))
    .find(|candidate| candidate.is_file())
    .expect("git is not on PATH");
  std::os::unix::fs::symlink(git, dir.path().join("git")).expect("cannot symlink git");
  dir
}
