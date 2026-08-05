use std::time::Duration;

use assert_cmd::Command;
use serde_json::Value;

mod common;

use common::conventions::{
  age_file, better_envelope, cache_dir, gb, mtime, profile_json, sole_cache_file,
};

#[test]
fn second_call_is_served_from_cache() {
  let repo = common::init_with_pr_trail();
  let cache = cache_dir();

  let first = gb(repo.path(), cache.path())
    .args(["conventions", "--plain"])
    .output()
    .unwrap();
  let cached_file = sole_cache_file(cache.path());
  let first_mtime = mtime(&cached_file);

  let second = gb(repo.path(), cache.path())
    .args(["conventions", "--plain"])
    .output()
    .unwrap();

  assert_eq!(first.stdout, second.stdout);
  assert_eq!(
    first_mtime,
    mtime(&cached_file),
    "cache hit must not rewrite the profile"
  );
  assert_eq!(
    better_envelope(repo.path(), cache.path(), &[])["meta"]["cache"],
    "hit"
  );
}

#[test]
fn refresh_recomputes_and_rewrites_the_cache() {
  let repo = common::init_with_pr_trail();
  let cache = cache_dir();

  gb(repo.path(), cache.path())
    .args(["conventions", "--plain"])
    .output()
    .unwrap();
  let cached_file = sole_cache_file(cache.path());
  let before = mtime(&cached_file);

  std::thread::sleep(Duration::from_millis(20));
  let refreshed = better_envelope(repo.path(), cache.path(), &["--refresh"]);

  assert_eq!(refreshed["meta"]["cache"], "miss");
  assert_ne!(
    before,
    mtime(&cached_file),
    "--refresh must rewrite the profile"
  );
}

#[test]
fn changing_a_declared_file_invalidates_the_cache() {
  let repo = common::init_repo();
  std::fs::write(repo.path().join("CONTRIBUTING.md"), "# Contributing\n").unwrap();
  let cache = cache_dir();

  let before = profile_json(repo.path(), cache.path());
  std::fs::write(
    repo.path().join("CONTRIBUTING.md"),
    "# Contributing\n\nBe kind.\n",
  )
  .unwrap();
  let after = profile_json(repo.path(), cache.path());

  assert_ne!(before["source_hash"], after["source_hash"]);
}

#[test]
fn cache_falls_back_to_xdg_cache_home() {
  let repo = common::init_repo();
  let xdg = tempfile::tempdir().unwrap();

  let out = Command::cargo_bin("gb")
    .unwrap()
    .current_dir(repo.path())
    .env_remove("GB_CACHE_DIR")
    .env("XDG_CACHE_HOME", xdg.path())
    .args(["conventions", "--plain"])
    .output()
    .unwrap();

  assert!(out.status.success());
  let written = std::fs::read_dir(xdg.path().join("git-better").join("conventions"))
    .unwrap()
    .filter_map(Result::ok)
    .count();
  assert_eq!(written, 1);
}

#[test]
fn a_cache_entry_older_than_seven_days_is_recomputed() {
  let repo = common::init_repo();
  let cache = cache_dir();

  gb(repo.path(), cache.path())
    .args(["conventions", "--plain"])
    .output()
    .unwrap();
  let cached_file = sole_cache_file(cache.path());
  age_file(&cached_file, Duration::from_secs(8 * 24 * 60 * 60));

  assert_eq!(
    better_envelope(repo.path(), cache.path(), &[])["meta"]["cache"],
    "miss"
  );
}

#[test]
fn a_cache_entry_from_another_schema_is_recomputed() {
  let repo = common::init_repo();
  let cache = cache_dir();

  gb(repo.path(), cache.path())
    .args(["conventions", "--plain"])
    .output()
    .unwrap();
  let cached_file = sole_cache_file(cache.path());
  let mut stored: Value =
    serde_json::from_str(&std::fs::read_to_string(&cached_file).unwrap()).unwrap();
  stored["schema_version"] = Value::from(99);
  std::fs::write(&cached_file, stored.to_string()).unwrap();

  assert_eq!(
    better_envelope(repo.path(), cache.path(), &[])["meta"]["cache"],
    "miss"
  );
  let rewritten: Value =
    serde_json::from_str(&std::fs::read_to_string(&cached_file).unwrap()).unwrap();
  assert_eq!(rewritten["schema_version"], 1);
}

#[test]
fn a_cache_entry_for_another_repository_is_recomputed() {
  let repo = common::init_repo();
  let cache = cache_dir();

  gb(repo.path(), cache.path())
    .args(["conventions", "--plain"])
    .output()
    .unwrap();
  let cached_file = sole_cache_file(cache.path());
  let mut stored: Value =
    serde_json::from_str(&std::fs::read_to_string(&cached_file).unwrap()).unwrap();
  stored["repo_root"] = Value::from("/somewhere/else");
  std::fs::write(&cached_file, stored.to_string()).unwrap();

  assert_eq!(
    better_envelope(repo.path(), cache.path(), &[])["meta"]["cache"],
    "miss"
  );
}

#[test]
fn a_corrupt_cache_entry_is_recomputed_instead_of_failing() {
  let repo = common::init_repo();
  let cache = cache_dir();

  gb(repo.path(), cache.path())
    .args(["conventions", "--plain"])
    .output()
    .unwrap();
  let cached_file = sole_cache_file(cache.path());
  std::fs::write(&cached_file, "{ not json").unwrap();

  let env = better_envelope(repo.path(), cache.path(), &[]);
  assert_eq!(env["meta"]["cache"], "miss");
  assert_eq!(env["data"]["schema_version"], 1);
}
