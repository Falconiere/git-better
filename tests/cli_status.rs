mod common;

use assert_cmd::Command;

#[test]
fn gb_status_matches_git_status_sb() {
    let dir = common::init_repo();
    let expected = common::git_stdout(&["status", "-sb"], dir.path());
    let actual = Command::cargo_bin("gb")
        .unwrap()
        .args(["status"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(actual.status.success(), "gb status exited non-zero");
    assert_eq!(
        String::from_utf8(actual.stdout).unwrap(),
        expected,
        "stdout must be byte-for-byte identical to `git -c color.ui=false status -sb`"
    );
}

#[test]
fn gb_status_plain_matches_git_status_sb() {
    let dir = common::init_repo();
    let expected = common::git_stdout(&["status", "-sb"], dir.path());
    let actual = Command::cargo_bin("gb")
        .unwrap()
        .args(["--plain", "status"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(actual.status.success());
    assert_eq!(String::from_utf8(actual.stdout).unwrap(), expected);
}

#[test]
fn gb_status_better_is_valid_json_envelope() {
    let dir = common::init_repo();
    let actual = Command::cargo_bin("gb")
        .unwrap()
        .args(["--better", "status"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(actual.status.success());
    let stdout = String::from_utf8(actual.stdout).unwrap();
    let v: serde_json::Value =
        serde_json::from_str(&stdout).expect("gb --better must emit valid JSON");
    assert_eq!(v["ok"], true);
    assert_eq!(v["command"], "status");
    assert!(v["data"]["raw"].is_string());
}

#[test]
fn gb_status_forwards_extra_args() {
    let dir = common::init_repo();
    let actual = Command::cargo_bin("gb")
        .unwrap()
        .args(["status", "--porcelain"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(actual.status.success());
    let stdout = String::from_utf8(actual.stdout).unwrap();
    // Forwarding matches the shim: `-sb` is prepended, then user args.
    // So `gb status --porcelain` → `git -c color.ui=false status -sb --porcelain`,
    // which still emits the `## main` branch header and the porcelain file list.
    assert!(stdout.contains("dirty.txt"));
    assert!(stdout.contains("## main"));
}

#[test]
fn gb_status_outside_repo_exits_nonzero() {
    let dir = tempfile::tempdir().unwrap();
    let actual = Command::cargo_bin("gb")
        .unwrap()
        .args(["status"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        !actual.status.success(),
        "gb status must fail outside a repo"
    );
}
