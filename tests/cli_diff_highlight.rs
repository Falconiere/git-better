mod common;

use assert_cmd::Command;

#[test]
fn diff_full_contains_added_and_removed_lines() {
    let dir = common::init_syntax_heavy();
    let actual = Command::cargo_bin("gb")
        .unwrap()
        .args(["diff", "--full"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(actual.status.success());
    let stdout = String::from_utf8(actual.stdout).unwrap();
    assert!(
        stdout.contains("+"),
        "expected a `+` insertion marker; got: {stdout}"
    );
    assert!(
        stdout.contains("-"),
        "expected a `-` deletion marker; got: {stdout}"
    );
    assert!(
        stdout.contains("@@"),
        "expected a hunk header `@@`; got: {stdout}"
    );
}

#[test]
fn diff_lean_prints_bar_chart_in_pretty_mode() {
    let dir = common::init_syntax_heavy();
    let actual = Command::cargo_bin("gb")
        .unwrap()
        .args(["diff"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(actual.status.success());
    let stdout = String::from_utf8(actual.stdout).unwrap();
    assert!(
        stdout.contains("▌"),
        "expected a bar `▌` chart character; got: {stdout}"
    );
    assert!(
        stdout.contains("file(s)"),
        "expected file count summary; got: {stdout}"
    );
    assert!(
        stdout.contains("excluded"),
        "expected lockfile excluded note; got: {stdout}"
    );
}

#[test]
fn diff_full_syntax_highlight_off_in_plain_mode() {
    let dir = common::init_syntax_heavy();
    let actual = Command::cargo_bin("gb")
        .unwrap()
        .args(["--plain", "diff", "--full"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8(actual.stdout).unwrap();
    assert!(!stdout.contains("\u{1b}["));
    assert!(stdout.contains("+"));
    assert!(stdout.contains("-"));
}
