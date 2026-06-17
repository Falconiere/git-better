mod common;

use assert_cmd::Command;

#[test]
fn reflog_prints_entries() {
    let dir = common::init_repo();
    let actual = Command::cargo_bin("gb")
        .unwrap()
        .args(["reflog"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8(actual.stdout).unwrap();
    assert!(stdout.contains("reflog"));
    assert!(stdout.contains("commit"));
}

#[test]
fn reflog_better_is_valid_envelope() {
    let dir = common::init_repo();
    let actual = Command::cargo_bin("gb")
        .unwrap()
        .args(["--better", "reflog"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8(actual.stdout).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("JSON");
    assert_eq!(v["command"], "reflog");
    let entries = v["data"]["entries"].as_array().unwrap();
    assert!(!entries.is_empty());
    let first = &entries[0];
    assert!(first["sha"].is_string());
    assert!(first["action"].is_string());
}

#[test]
fn reflog_respects_n_flag() {
    let dir = common::init_repo();
    let actual = Command::cargo_bin("gb")
        .unwrap()
        .args(["reflog", "-n", "1"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8(actual.stdout).unwrap();
    let action_lines: Vec<&str> = stdout
        .lines()
        .filter(|l| l.contains("commit") || l.contains("checkout") || l.contains("reset"))
        .collect();
    assert_eq!(
        action_lines.len(),
        1,
        "expected exactly 1 reflog entry with -n 1; got: {action_lines:?}"
    );
}
