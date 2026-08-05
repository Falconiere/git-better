use std::path::Path;

use assert_cmd::Command;
use serde_json::Value;
use tempfile::TempDir;

mod common;

fn skill_text() -> String {
  std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("SKILL.md")).unwrap()
}

fn gb(repo: &Path, home: &Path) -> Command {
  let mut cmd = Command::cargo_bin("gb").unwrap();
  cmd
    .current_dir(repo)
    .env("HOME", home)
    .env_remove("CLAUDE_CONFIG_DIR")
    .env_remove("CODEX_HOME");
  cmd
}

fn install(repo: &Path, home: &Path, extra: &[&str]) -> String {
  let out = gb(repo, home)
    .args(["skill", "install"])
    .args(extra)
    .output()
    .unwrap();
  assert!(
    out.status.success(),
    "install failed: {}",
    String::from_utf8_lossy(&out.stderr)
  );
  String::from_utf8(out.stdout).unwrap()
}

fn fixture() -> (TempDir, TempDir) {
  (common::init_repo(), tempfile::tempdir().unwrap())
}

#[test]
fn print_emits_the_embedded_protocol() {
  let (repo, home) = fixture();

  let out = gb(repo.path(), home.path())
    .args(["skill", "print"])
    .output()
    .unwrap();

  assert!(out.status.success());
  assert_eq!(String::from_utf8(out.stdout).unwrap(), skill_text());
}

#[test]
fn the_protocol_never_contains_the_fence_markers() {
  let text = skill_text();

  assert!(
    !text.contains("<!-- git-better:begin -->"),
    "SKILL.md must not contain the fence markers — a fenced install would nest \
     the block and stop being idempotent"
  );
  assert!(!text.contains("<!-- git-better:end -->"));
}

#[test]
fn path_lists_every_target() {
  let (repo, home) = fixture();

  let out = gb(repo.path(), home.path())
    .args(["skill", "path"])
    .output()
    .unwrap();
  let text = String::from_utf8(out.stdout).unwrap();

  assert_eq!(text.lines().count(), 7, "{text}");
  assert!(
    text.contains(&format!(
      "claude-user\t{}",
      home
        .path()
        .join(".claude/skills/git-better/SKILL.md")
        .display()
    )),
    "{text}"
  );
  assert!(
    text.contains(&format!(
      "agents-md\t{}",
      repo.path().join("AGENTS.md").display()
    )),
    "{text}"
  );
}

#[test]
fn dry_run_writes_nothing() {
  let (repo, home) = fixture();

  let report = install(repo.path(), home.path(), &["--all", "--dry-run"]);

  assert!(report.contains("wrote"), "{report}");
  assert!(!repo.path().join("AGENTS.md").exists());
  assert!(!repo.path().join(".cursor/rules/git-better.mdc").exists());
  assert!(
    !home
      .path()
      .join(".claude/skills/git-better/SKILL.md")
      .exists()
  );
}

#[test]
fn install_all_then_reinstall_reports_unchanged() {
  let (repo, home) = fixture();

  let first = install(repo.path(), home.path(), &["--all"]);
  assert_eq!(first.matches("wrote").count(), 7, "{first}");

  let claude = home.path().join(".claude/skills/git-better/SKILL.md");
  assert_eq!(std::fs::read_to_string(&claude).unwrap(), skill_text());

  let second = install(repo.path(), home.path(), &["--all"]);
  assert_eq!(second.matches("unchanged").count(), 7, "{second}");
}

#[test]
fn cursor_rule_carries_frontmatter_instead_of_the_skill_header() {
  let (repo, home) = fixture();

  install(repo.path(), home.path(), &["--target", "cursor"]);
  let text = std::fs::read_to_string(repo.path().join(".cursor/rules/git-better.mdc")).unwrap();

  assert!(text.starts_with("---\ndescription:"), "{text}");
  assert!(text.contains("alwaysApply: true"), "{text}");
  assert!(
    text.contains("# git-better — Token-Lean Git Protocol"),
    "{text}"
  );
  assert!(!text.contains("name: git-better"), "{text}");
}

#[test]
fn fenced_install_preserves_existing_content_and_never_duplicates() {
  let (repo, home) = fixture();
  let agents = repo.path().join("AGENTS.md");
  std::fs::write(&agents, "# House rules\n\nAlways rebase.\n").unwrap();

  install(repo.path(), home.path(), &["--target", "agents-md"]);
  let once = std::fs::read_to_string(&agents).unwrap();
  assert!(once.starts_with("# House rules"), "{once}");
  assert_eq!(once.matches("<!-- git-better:begin -->").count(), 1);
  assert_eq!(once.matches("<!-- git-better:end -->").count(), 1);

  install(repo.path(), home.path(), &["--target", "agents-md"]);
  let twice = std::fs::read_to_string(&agents).unwrap();
  assert_eq!(twice, once, "re-install must be a no-op");
  assert_eq!(twice.matches("<!-- git-better:begin -->").count(), 1);
}

#[test]
fn fenced_block_is_replaced_in_place_when_content_drifts() {
  let (repo, home) = fixture();
  let agents = repo.path().join("AGENTS.md");
  std::fs::write(
    &agents,
    "intro\n\n<!-- git-better:begin -->\nstale text\n<!-- git-better:end -->\n\noutro\n",
  )
  .unwrap();

  install(repo.path(), home.path(), &["--target", "agents-md"]);
  let text = std::fs::read_to_string(&agents).unwrap();

  assert!(text.starts_with("intro"), "{text}");
  assert!(text.contains("outro"), "{text}");
  assert!(!text.contains("stale text"), "{text}");
  assert_eq!(text.matches("<!-- git-better:begin -->").count(), 1);
}

#[test]
fn foreign_whole_file_is_skipped_until_forced() {
  let (repo, home) = fixture();
  let rule = repo.path().join(".windsurf/rules/git-better.md");
  std::fs::create_dir_all(rule.parent().unwrap()).unwrap();
  std::fs::write(&rule, "someone else's rule\n").unwrap();

  let skipped = install(repo.path(), home.path(), &["--target", "windsurf"]);
  assert!(skipped.contains("skipped (foreign content"), "{skipped}");
  assert_eq!(
    std::fs::read_to_string(&rule).unwrap(),
    "someone else's rule\n"
  );

  let forced = install(
    repo.path(),
    home.path(),
    &["--target", "windsurf", "--force"],
  );
  assert!(forced.contains("wrote"), "{forced}");
  assert_eq!(std::fs::read_to_string(&rule).unwrap(), skill_text());
}

#[test]
fn default_install_only_touches_detected_targets() {
  let (repo, home) = fixture();
  std::fs::create_dir_all(repo.path().join(".cursor")).unwrap();

  let report = install(repo.path(), home.path(), &[]);

  assert!(repo.path().join(".cursor/rules/git-better.mdc").exists());
  assert!(!repo.path().join("AGENTS.md").exists(), "{report}");
  assert!(
    !home
      .path()
      .join(".claude/skills/git-better/SKILL.md")
      .exists()
  );
  assert!(report.contains("not detected"), "{report}");
}

#[test]
fn unknown_target_lists_the_valid_names() {
  let (repo, home) = fixture();

  let out = gb(repo.path(), home.path())
    .args(["skill", "install", "--target", "emacs"])
    .output()
    .unwrap();

  assert!(!out.status.success());
  let err = String::from_utf8(out.stderr).unwrap();
  assert!(err.contains("unknown target 'emacs'"), "{err}");
  assert!(err.contains("agents-md"), "{err}");
}

#[test]
fn better_report_groups_targets_by_outcome() {
  let (repo, home) = fixture();

  let out = gb(repo.path(), home.path())
    .args(["skill", "install", "--all", "--better", "--dry-run"])
    .output()
    .unwrap();
  let env: Value = serde_json::from_slice(&out.stdout).unwrap();

  assert_eq!(env["ok"], true);
  assert_eq!(env["command"], "skill install");
  assert_eq!(env["meta"]["targets"], 7);
  assert_eq!(env["data"]["installed"].as_array().unwrap().len(), 7);
  assert_eq!(env["data"]["unchanged"].as_array().unwrap().len(), 0);
  assert_eq!(env["data"]["skipped"].as_array().unwrap().len(), 0);
}

#[test]
fn outside_a_repository_project_targets_are_skipped() {
  let plain = tempfile::tempdir().unwrap();
  let home = tempfile::tempdir().unwrap();

  let report = install(plain.path(), home.path(), &["--all"]);

  assert_eq!(
    report.matches("skipped (no repository)").count(),
    5,
    "{report}"
  );
  assert!(
    home
      .path()
      .join(".claude/skills/git-better/SKILL.md")
      .exists()
  );
  assert!(home.path().join(".codex/AGENTS.md").exists());
}
