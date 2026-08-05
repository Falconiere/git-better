use serde_json::Value;

mod common;

use common::conventions::{cache_dir, gb, profile_json};

#[test]
fn summary_reports_a_conventional_repo() {
  let repo = common::init_with_pr_trail();
  let cache = cache_dir();

  let out = gb(repo.path(), cache.path())
    .args(["conventions", "--plain"])
    .output()
    .unwrap();
  let text = String::from_utf8(out.stdout).unwrap();

  assert!(out.status.success());
  assert!(
    text.contains("commit:  conventional-commits | scope used | suffix (#N)"),
    "unexpected summary: {text}"
  );
  assert!(text.contains("branch:  type/kebab [feat]"), "{text}");
  assert!(text.contains("prose:   pending []"), "{text}");
  assert_eq!(text.lines().count(), 5, "{text}");
}

#[test]
fn summary_reports_an_unconventional_repo_as_unknown() {
  let repo = common::init_repo();
  common::make_commit(repo.path(), "tweak the parser");
  common::make_commit(repo.path(), "more work");
  let cache = cache_dir();

  let out = gb(repo.path(), cache.path())
    .args(["conventions", "--plain"])
    .output()
    .unwrap();
  let text = String::from_utf8(out.stdout).unwrap();

  assert!(
    text.contains("commit:  unknown | scope none | suffix none"),
    "{text}"
  );
}

#[test]
fn json_profile_carries_the_schema_and_evidence() {
  let repo = common::init_with_pr_trail();
  let cache = cache_dir();

  let profile = profile_json(repo.path(), cache.path());

  assert_eq!(profile["schema_version"], 1);
  assert_eq!(
    profile["commit_format"]["convention"],
    "conventional-commits"
  );
  assert_eq!(profile["commit_format"]["scope"], "used");
  assert_eq!(profile["commit_format"]["pr_suffix"], "(#N)");
  assert_eq!(profile["pr"]["title_format"], "conventional-commits");
  assert_eq!(profile["branch_naming"]["pattern"], "type/kebab");
  assert_eq!(profile["remote_consulted"], false);
  assert_eq!(profile["pr"]["recent_titles"].as_array().unwrap().len(), 0);
  assert_eq!(
    profile["commit_format"]["samples"]
      .as_array()
      .unwrap()
      .len(),
    3
  );
  assert_eq!(profile["repo_root"], repo.path().to_str().unwrap());
}

#[test]
fn pr_template_headings_become_body_sections() {
  let repo = common::init_repo();
  let github = repo.path().join(".github");
  std::fs::create_dir_all(&github).unwrap();
  std::fs::write(
    github.join("PULL_REQUEST_TEMPLATE.md"),
    "## Summary\n\ntext\n\n## Tests\n\n- [ ] ran them\n\n#### Ignored\n",
  )
  .unwrap();
  let cache = cache_dir();

  let profile = profile_json(repo.path(), cache.path());

  assert_eq!(
    profile["pr"]["template_path"],
    ".github/PULL_REQUEST_TEMPLATE.md"
  );
  let sections = profile["pr"]["body_sections"].as_array().unwrap();
  assert_eq!(sections.len(), 2);
  assert_eq!(sections[0], "Summary");
  assert_eq!(sections[1], "Tests");
}

#[test]
fn release_tooling_and_changelog_are_detected() {
  let repo = common::init_repo();
  let workflows = repo.path().join(".github/workflows");
  std::fs::create_dir_all(&workflows).unwrap();
  std::fs::write(workflows.join("release.yml"), "name: release\n").unwrap();
  std::fs::write(repo.path().join("CHANGELOG.md"), "# Changelog\n").unwrap();
  std::fs::write(repo.path().join("release-please-config.json"), "{}\n").unwrap();
  let cache = cache_dir();

  let profile = profile_json(repo.path(), cache.path());

  let tooling = profile["release"]["tooling"].as_array().unwrap();
  assert!(
    tooling.contains(&Value::from("release.yml-workflow")),
    "{tooling:?}"
  );
  assert!(
    tooling.contains(&Value::from("release-please")),
    "{tooling:?}"
  );
  assert_eq!(profile["release"]["changelog"], "CHANGELOG.md");
}

#[test]
fn better_envelope_carries_hints_and_cache_state() {
  let repo = common::init_repo();
  std::fs::write(repo.path().join("CONTRIBUTING.md"), "# Contributing\n").unwrap();
  let cache = cache_dir();

  let out = gb(repo.path(), cache.path())
    .args(["conventions", "--better"])
    .output()
    .unwrap();
  let env: Value = serde_json::from_slice(&out.stdout).unwrap();

  assert_eq!(env["ok"], true);
  assert_eq!(env["command"], "conventions");
  assert_eq!(env["meta"]["cache"], "miss");
  assert!(env["meta"]["bytes"].as_u64().unwrap() > 0);
  assert_eq!(env["data"]["prose_pending"][0], "CONTRIBUTING.md");

  let hints = env["hints"].as_array().unwrap();
  assert!(
    hints.iter().any(|h| h
      .as_str()
      .is_some_and(|s| s.contains("--save-prose CONTRIBUTING.md"))),
    "{hints:?}"
  );
  assert!(
    hints
      .iter()
      .any(|h| h.as_str().is_some_and(|s| s.contains("--with-remote"))),
    "{hints:?}"
  );
}

#[test]
fn json_and_better_together_are_rejected() {
  let repo = common::init_repo();
  let cache = cache_dir();

  let out = gb(repo.path(), cache.path())
    .args(["conventions", "--json", "--better"])
    .output()
    .unwrap();

  assert!(!out.status.success());
  let err = String::from_utf8(out.stderr).unwrap();
  assert!(err.contains("mutually exclusive"), "{err}");
}

#[test]
fn save_prose_persists_rules_and_clears_pending() {
  let repo = common::init_repo();
  std::fs::write(
    repo.path().join("CONTRIBUTING.md"),
    "# Contributing\n\nSquash before merge.\n",
  )
  .unwrap();
  let cache = cache_dir();

  let pending = profile_json(repo.path(), cache.path());
  assert_eq!(pending["prose_pending"][0], "CONTRIBUTING.md");

  let saved = gb(repo.path(), cache.path())
    .args(["conventions", "--save-prose", "CONTRIBUTING.md", "--json"])
    .write_stdin("squash before merge; no merge commits")
    .output()
    .unwrap();
  assert!(
    saved.status.success(),
    "{:?}",
    String::from_utf8_lossy(&saved.stderr)
  );

  let profile: Value = serde_json::from_slice(&saved.stdout).unwrap();
  assert_eq!(profile["prose_pending"].as_array().unwrap().len(), 0);
  assert_eq!(
    profile["prose_distilled"]["CONTRIBUTING.md"]["rules"],
    "squash before merge; no merge commits"
  );

  let later = profile_json(repo.path(), cache.path());
  assert_eq!(later["prose_pending"].as_array().unwrap().len(), 0);
  assert_eq!(
    later["prose_distilled"]["CONTRIBUTING.md"]["rules"],
    "squash before merge; no merge commits"
  );
}

#[test]
fn distilled_prose_is_truncated_to_the_cap() {
  let repo = common::init_repo();
  std::fs::write(repo.path().join("CONTRIBUTING.md"), "# Contributing\n").unwrap();
  let cache = cache_dir();

  let out = gb(repo.path(), cache.path())
    .args(["conventions", "--save-prose", "CONTRIBUTING.md", "--json"])
    .write_stdin("x".repeat(9000))
    .output()
    .unwrap();
  let profile: Value = serde_json::from_slice(&out.stdout).unwrap();

  let rules = profile["prose_distilled"]["CONTRIBUTING.md"]["rules"]
    .as_str()
    .unwrap();
  assert_eq!(rules.len(), 8 * 1024);
}

#[test]
fn save_prose_rejects_empty_stdin_and_unknown_files() {
  let repo = common::init_repo();
  std::fs::write(repo.path().join("CONTRIBUTING.md"), "# Contributing\n").unwrap();
  let cache = cache_dir();

  let empty = gb(repo.path(), cache.path())
    .args(["conventions", "--save-prose", "CONTRIBUTING.md"])
    .write_stdin("   \n")
    .output()
    .unwrap();
  assert!(!empty.status.success());
  assert!(
    String::from_utf8_lossy(&empty.stderr).contains("non-empty text"),
    "{}",
    String::from_utf8_lossy(&empty.stderr)
  );

  let missing = gb(repo.path(), cache.path())
    .args(["conventions", "--save-prose", "NOPE.md"])
    .write_stdin("rules")
    .output()
    .unwrap();
  assert!(!missing.status.success());
  assert!(
    String::from_utf8_lossy(&missing.stderr).contains("no such file"),
    "{}",
    String::from_utf8_lossy(&missing.stderr)
  );

  let escaping = gb(repo.path(), cache.path())
    .args(["conventions", "--save-prose", "../outside.md"])
    .write_stdin("rules")
    .output()
    .unwrap();
  assert!(!escaping.status.success());
}

#[test]
fn save_prose_rejects_a_symlink_that_escapes_the_repository() {
  let repo = common::init_repo();
  let outside = tempfile::tempdir().unwrap();
  let secret = outside.path().join("outside.md");
  std::fs::write(&secret, "not part of the repository\n").unwrap();
  std::os::unix::fs::symlink(&secret, repo.path().join("linked.md")).unwrap();
  let cache = cache_dir();

  let out = gb(repo.path(), cache.path())
    .args(["conventions", "--save-prose", "linked.md"])
    .write_stdin("rules")
    .output()
    .unwrap();

  assert!(
    !out.status.success(),
    "a symlink out of the repo must be refused"
  );
  assert!(
    String::from_utf8_lossy(&out.stderr).contains("inside the repository"),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );
}

#[test]
fn save_prose_accepts_a_symlink_that_stays_inside_the_repository() {
  let repo = common::init_repo();
  std::fs::write(repo.path().join("CONTRIBUTING.md"), "# Contributing\n").unwrap();
  std::os::unix::fs::symlink(
    repo.path().join("CONTRIBUTING.md"),
    repo.path().join("rules.md"),
  )
  .unwrap();
  let cache = cache_dir();

  let out = gb(repo.path(), cache.path())
    .args(["conventions", "--save-prose", "rules.md", "--json"])
    .write_stdin("squash before merge")
    .output()
    .unwrap();
  assert!(
    out.status.success(),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );

  let profile: Value = serde_json::from_slice(&out.stdout).unwrap();
  assert_eq!(
    profile["prose_distilled"]["CONTRIBUTING.md"]["rules"], "squash before merge",
    "a symlink inside the repo resolves to its target path"
  );
}

#[test]
fn with_remote_without_gh_on_path_yields_no_titles() {
  let repo = common::init_repo();
  let cache = cache_dir();
  let bin = common::conventions::bin_dir_with_git_only();

  let out = gb(repo.path(), cache.path())
    .args(["conventions", "--json", "--with-remote"])
    .env("PATH", bin.path())
    .output()
    .unwrap();
  assert!(
    out.status.success(),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );
  let profile: Value = serde_json::from_slice(&out.stdout).unwrap();

  assert_eq!(profile["gh_available"], false);
  assert_eq!(profile["remote_consulted"], true);
  assert_eq!(profile["pr"]["recent_titles"].as_array().unwrap().len(), 0);
}

#[test]
fn outside_a_repository_the_command_fails() {
  let plain = tempfile::tempdir().unwrap();
  let cache = cache_dir();

  let out = gb(plain.path(), cache.path())
    .args(["conventions", "--plain"])
    .env("GIT_CEILING_DIRECTORIES", plain.path())
    .output()
    .unwrap();

  assert!(!out.status.success());
}
