mod common;

use assert_cmd::Command;

#[test]
fn branch_lists_branches() {
  let dir = common::init_repo();
  let actual = Command::cargo_bin("gb")
    .unwrap()
    .args(["branch"])
    .current_dir(dir.path())
    .output()
    .unwrap();
  let stdout = String::from_utf8(actual.stdout).unwrap();
  assert!(stdout.contains("main"));
  assert!(stdout.lines().any(|l| l.starts_with("* main")), "expected current branch marker; got: {stdout}");
}

#[test]
fn branch_better_is_valid_envelope() {
  let dir = common::init_repo();
  let actual = Command::cargo_bin("gb")
    .unwrap()
    .args(["--better", "branch"])
    .current_dir(dir.path())
    .output()
    .unwrap();
  let stdout = String::from_utf8(actual.stdout).unwrap();
  let v: serde_json::Value = serde_json::from_str(&stdout).expect("JSON");
  assert_eq!(v["command"], "branch");
  let current = v["data"]["current"].as_str();
  assert_eq!(current, Some("main"));
  let locals = v["data"]["locals"].as_array().unwrap();
  assert!(locals.iter().any(|r| r["name"] == "main"));
}

#[test]
fn branch_forwards_extra_args() {
  let dir = common::init_repo();
  let actual = Command::cargo_bin("gb")
    .unwrap()
    .args(["branch", "--list"])
    .current_dir(dir.path())
    .output()
    .unwrap();
  let stdout = String::from_utf8(actual.stdout).unwrap();
  assert!(stdout.contains("main"));
}
