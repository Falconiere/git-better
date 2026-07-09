mod common;

use assert_cmd::Command;

#[test]
fn passthrough_remote_v() {
  let dir = common::init_repo();
  let actual = Command::cargo_bin("gb")
    .unwrap()
    .args(["remote", "-v"])
    .current_dir(dir.path())
    .output()
    .unwrap();
  assert!(actual.status.success());
  let stdout = String::from_utf8(actual.stdout).unwrap();
  assert_eq!(stdout.trim(), "", "no remotes set");
}

#[test]
fn passthrough_rev_parse() {
  let dir = common::init_repo();
  let actual = Command::cargo_bin("gb")
    .unwrap()
    .args(["rev-parse", "HEAD"])
    .current_dir(dir.path())
    .output()
    .unwrap();
  assert!(actual.status.success());
  let stdout = String::from_utf8(actual.stdout).unwrap();
  assert!(
    stdout.trim().chars().all(|c| c.is_ascii_hexdigit()) && stdout.trim().len() == 40,
    "expected a 40-char hex SHA, got: {stdout:?}"
  );
}

#[test]
fn passthrough_tag_list() {
  let dir = common::init_repo();
  let actual = Command::cargo_bin("gb")
    .unwrap()
    .args(["tag", "--list"])
    .current_dir(dir.path())
    .output()
    .unwrap();
  assert!(actual.status.success());
  let stdout = String::from_utf8(actual.stdout).unwrap();
  assert_eq!(stdout.trim(), "");
}

#[test]
fn passthrough_config_get() {
  let dir = common::init_repo();
  let actual = Command::cargo_bin("gb")
    .unwrap()
    .args(["config", "--get", "user.email"])
    .current_dir(dir.path())
    .output()
    .unwrap();
  assert!(actual.status.success());
  let stdout = String::from_utf8(actual.stdout).unwrap();
  assert_eq!(stdout.trim(), "test@example.com");
}
