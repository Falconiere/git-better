mod common;

use assert_cmd::Command;

#[test]
fn diff_better_returns_valid_envelope() {
    let dir = common::init_repo_with_tracked_change();
    let actual = Command::cargo_bin("gb")
        .unwrap()
        .args(["--better", "diff"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(actual.status.success());
    let stdout = String::from_utf8(actual.stdout).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("JSON");
    assert_eq!(v["ok"], true);
    assert_eq!(v["command"], "diff");
    assert!(v["data"]["files"].is_array());
    assert!(v["data"]["summary"].is_object());
    assert_eq!(v["data"]["summary"]["files_changed"], 1);
    assert_eq!(v["data"]["summary"]["added"], 1);
    assert_eq!(v["data"]["summary"]["removed"], 1);
    assert!(v["data"]["patch"].is_string());
}

#[test]
fn diff_better_excludes_lockfiles_by_default() {
    let dir = common::init_lockfile_heavy();
    let actual = Command::cargo_bin("gb")
        .unwrap()
        .args(["--better", "diff"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(actual.status.success());
    let stdout = String::from_utf8(actual.stdout).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let files = v["data"]["files"].as_array().unwrap();
    let paths: Vec<&str> = files.iter().map(|f| f["path"].as_str().unwrap()).collect();
    assert!(
        !paths
            .iter()
            .any(|p| p.ends_with(".lock") || p.contains("lock.json") || p.contains("bun.lock")),
        "lockfile must be excluded by default; got {paths:?}"
    );
    assert!(
        paths.contains(&"main.rs"),
        "non-lockfile change must appear; got {paths:?}"
    );
}

#[test]
fn diff_better_full_includes_lockfiles() {
    let dir = common::init_lockfile_heavy();
    let actual = Command::cargo_bin("gb")
        .unwrap()
        .args(["--better", "diff", "--full"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8(actual.stdout).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let files = v["data"]["files"].as_array().unwrap();
    let paths: Vec<&str> = files.iter().map(|f| f["path"].as_str().unwrap()).collect();
    assert!(
        paths.iter().any(|p| p.contains("Cargo.lock")),
        "`--full` must include lockfile diff; got {paths:?}"
    );
}

#[test]
fn diff_better_budget_truncates_large_diff() {
    let dir = common::init_syntax_heavy();
    let actual = Command::cargo_bin("gb")
        .unwrap()
        .args(["--better", "diff", "--budget", "50"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8(actual.stdout).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(v["data"]["truncated"], true);
    let arr = v["data"]["truncated_files"].as_array().unwrap();
    assert!(
        !arr.is_empty(),
        "truncated_files must be non-empty when budget is tight"
    );
    let meta = &v["meta"];
    assert_eq!(meta["budget"], 50);
    let bytes = meta["bytes"].as_u64().unwrap();
    assert!(
        bytes <= 50 * 4,
        "truncated bytes ({bytes}) must fit in budget"
    );
    let hints = v["hints"].as_array().unwrap();
    assert!(hints
        .iter()
        .any(|h| h.as_str().unwrap().contains("gb diff --full")));
}

#[test]
fn diff_better_budget_unlimited_keeps_full_patch() {
    let dir = common::init_syntax_heavy();
    let actual = Command::cargo_bin("gb")
        .unwrap()
        .args(["--better", "diff"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8(actual.stdout).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(v["data"]["truncated"], false);
    assert!(!v["data"]["patch"].as_str().unwrap().is_empty());
}
