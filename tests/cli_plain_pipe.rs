mod common;

use assert_cmd::Command;

#[test]
fn piped_status_is_plain() {
  let dir = common::init_repo();
  let actual = Command::cargo_bin("gb")
    .unwrap()
    .args(["status"])
    .current_dir(dir.path())
    .output()
    .unwrap();
  let stdout = String::from_utf8(actual.stdout).unwrap();
  assert!(!stdout.contains("\u{1b}["));
}

#[test]
fn piped_log_is_plain() {
  let dir = common::init_with_pr_trail();
  let actual = Command::cargo_bin("gb")
    .unwrap()
    .args(["log"])
    .current_dir(dir.path())
    .output()
    .unwrap();
  let stdout = String::from_utf8(actual.stdout).unwrap();
  assert!(!stdout.contains("\u{1b}["));
}

#[test]
fn piped_diff_pretty_stat_is_plain() {
  let dir = common::init_syntax_heavy();
  let actual = Command::cargo_bin("gb")
    .unwrap()
    .args(["diff"])
    .current_dir(dir.path())
    .output()
    .unwrap();
  let stdout = String::from_utf8(actual.stdout).unwrap();
  assert!(!stdout.contains("\u{1b}["));
}

#[test]
fn plain_flag_strips_color() {
  let dir = common::init_with_pr_trail();
  let actual = Command::cargo_bin("gb")
    .unwrap()
    .args(["--plain", "log"])
    .current_dir(dir.path())
    .output()
    .unwrap();
  let stdout = String::from_utf8(actual.stdout).unwrap();
  assert!(!stdout.contains("\u{1b}["));
}
