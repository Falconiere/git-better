mod common;

use assert_cmd::Command;

#[test]
fn log_better_groups_commits_by_pr() {
  let dir = common::init_with_pr_trail();
  let actual = Command::cargo_bin("gb")
    .unwrap()
    .args(["--better", "log"])
    .current_dir(dir.path())
    .output()
    .unwrap();
  let stdout = String::from_utf8(actual.stdout).unwrap();
  let v: serde_json::Value = serde_json::from_str(&stdout).expect("JSON");
  let groups = v["data"]["groups"].as_array().expect("groups array");
  let pr142_group = groups.iter().find(|g| g["pr"] == 142);
  assert!(
    pr142_group.is_some(),
    "expected a group with pr=142; got: {}",
    serde_json::to_string_pretty(&groups).unwrap()
  );
  let g = pr142_group.unwrap();
  let commits = g["commits"].as_array().unwrap();
  assert_eq!(
    commits.len(),
    3,
    "PR #142 should bundle the 3 conventional commits"
  );
}

#[test]
fn log_better_counts_conventional_types() {
  let dir = common::init_with_pr_trail();
  let actual = Command::cargo_bin("gb")
    .unwrap()
    .args(["--better", "log"])
    .current_dir(dir.path())
    .output()
    .unwrap();
  let stdout = String::from_utf8(actual.stdout).unwrap();
  let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
  let by_type = v["data"]["by_type"].as_object().unwrap();
  assert_eq!(by_type.get("feat").and_then(|v| v.as_u64()), Some(1));
  assert_eq!(by_type.get("fix").and_then(|v| v.as_u64()), Some(1));
  assert_eq!(by_type.get("docs").and_then(|v| v.as_u64()), Some(1));
  assert_eq!(by_type.get("chore").and_then(|v| v.as_u64()), Some(1));
}

#[test]
fn log_better_includes_author_breakdown() {
  let dir = common::init_with_pr_trail();
  let actual = Command::cargo_bin("gb")
    .unwrap()
    .args(["--better", "log"])
    .current_dir(dir.path())
    .output()
    .unwrap();
  let stdout = String::from_utf8(actual.stdout).unwrap();
  let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
  let by_author = v["data"]["by_author"].as_object().unwrap();
  assert!(by_author.contains_key("Test"));
}

#[test]
fn log_story_prints_one_line_branch_summary() {
  let dir = common::init_with_pr_trail();
  let actual = Command::cargo_bin("gb")
    .unwrap()
    .args(["log", "--story"])
    .current_dir(dir.path())
    .output()
    .unwrap();
  assert!(actual.status.success());
  let stdout = String::from_utf8(actual.stdout).unwrap();
  assert!(
    stdout.contains("commit"),
    "story should mention commit count"
  );
  assert!(
    stdout.contains("feat/oauth") || stdout.contains("⎇"),
    "story should mention the branch"
  );
}

#[test]
fn log_story_better_emits_envelope() {
  let dir = common::init_with_pr_trail();
  let actual = Command::cargo_bin("gb")
    .unwrap()
    .args(["--better", "log", "--story"])
    .current_dir(dir.path())
    .output()
    .unwrap();
  let stdout = String::from_utf8(actual.stdout).unwrap();
  let v: serde_json::Value = serde_json::from_str(&stdout).expect("JSON");
  assert_eq!(v["command"], "log");
  assert!(v["data"]["story"].is_string());
  assert_eq!(v["data"]["pr"], 142);
}

#[test]
fn log_pretty_prints_recent_commits_with_type_tags() {
  let dir = common::init_with_pr_trail();
  let actual = Command::cargo_bin("gb")
    .unwrap()
    .args(["log"])
    .current_dir(dir.path())
    .output()
    .unwrap();
  let stdout = String::from_utf8(actual.stdout).unwrap();
  assert!(stdout.contains("recent commits"));
}

#[test]
fn log_better_budget_truncates_commits() {
  let dir = common::init_with_pr_trail();
  let full = Command::cargo_bin("gb")
    .unwrap()
    .args(["--better", "log"])
    .current_dir(dir.path())
    .output()
    .unwrap();
  let full_stdout = String::from_utf8(full.stdout).unwrap();
  let full_v: serde_json::Value = serde_json::from_str(&full_stdout).unwrap();
  let full_count = full_v["data"]["commits"].as_array().unwrap().len();

  let actual = Command::cargo_bin("gb")
    .unwrap()
    .args(["--better", "log", "--budget", "50"])
    .current_dir(dir.path())
    .output()
    .unwrap();
  let stdout = String::from_utf8(actual.stdout).unwrap();
  let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
  assert_eq!(v["data"]["truncated"], true);
  let count = v["data"]["commits"].as_array().unwrap().len();
  assert!(
    count < full_count,
    "budget should drop commits ({count} < {full_count})"
  );
}

#[test]
fn log_forwards_extra_args() {
  let dir = common::init_with_pr_trail();
  let actual = Command::cargo_bin("gb")
    .unwrap()
    .args(["log", "--author", "Test"])
    .current_dir(dir.path())
    .output()
    .unwrap();
  let stdout = String::from_utf8(actual.stdout).unwrap();
  assert!(
    stdout.contains("Test"),
    "passthrough --author should filter by author; got: {stdout}"
  );
}
