mod common;

use assert_cmd::Command;

#[test]
fn show_prints_stat_view_by_default() {
  let dir = common::init_syntax_heavy();
  let actual = Command::cargo_bin("gb")
    .unwrap()
    .args(["show", "HEAD"])
    .current_dir(dir.path())
    .output()
    .unwrap();
  assert!(actual.status.success());
  let stdout = String::from_utf8(actual.stdout).unwrap();
  assert!(
    stdout.contains("Author"),
    "expected Author line in stat view; got: {stdout}"
  );
  assert!(
    stdout.contains("app.rs"),
    "expected the file in the stat view; got: {stdout}"
  );
}

#[test]
fn show_full_prints_unified_diff() {
  let dir = common::init_syntax_heavy();
  let actual = Command::cargo_bin("gb")
    .unwrap()
    .args(["show", "HEAD", "--full"])
    .current_dir(dir.path())
    .output()
    .unwrap();
  assert!(actual.status.success());
  let stdout = String::from_utf8(actual.stdout).unwrap();
  assert!(
    stdout.contains("@@"),
    "expected hunk header in --full; got: {stdout}"
  );
  assert!(stdout.contains("+"));
}

#[test]
fn show_better_is_json() {
  let dir = common::init_syntax_heavy();
  let actual = Command::cargo_bin("gb")
    .unwrap()
    .args(["--better", "show", "HEAD"])
    .current_dir(dir.path())
    .output()
    .unwrap();
  let stdout = String::from_utf8(actual.stdout).unwrap();
  let v: serde_json::Value = serde_json::from_str(&stdout).expect("JSON");
  assert_eq!(v["command"], "show");
  assert!(v["data"]["commit"]["sha"].is_string());
  assert!(v["data"]["files"].is_array());
  assert!(v["data"]["patch"].is_string());
}

#[test]
fn show_better_detects_conventional_type() {
  let dir = common::init_syntax_heavy();
  let actual = Command::cargo_bin("gb")
    .unwrap()
    .args(["--better", "show", "HEAD"])
    .current_dir(dir.path())
    .output()
    .unwrap();
  let stdout = String::from_utf8(actual.stdout).unwrap();
  let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
  let commit = &v["data"]["commit"];
  assert_eq!(commit["conventional_type"], "feat");
}
