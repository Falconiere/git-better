mod common;

use assert_cmd::Command;

#[test]
fn no_color_disables_theme() {
    let actual = Command::cargo_bin("gb")
        .unwrap()
        .env("NO_COLOR", "1")
        .args(["status"])
        .current_dir(common::init_repo().path())
        .output()
        .unwrap();
    let stdout = String::from_utf8(actual.stdout).unwrap();
    assert!(
        !stdout.contains("\u{1b}["),
        "NO_COLOR=1 must suppress ANSI escapes; got: {stdout}"
    );
}

#[test]
fn no_color_with_better_still_emits_clean_json() {
    let actual = Command::cargo_bin("gb")
        .unwrap()
        .env("NO_COLOR", "1")
        .args(["--better", "status"])
        .current_dir(common::init_repo().path())
        .output()
        .unwrap();
    let stdout = String::from_utf8(actual.stdout).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("JSON");
    assert_eq!(v["ok"], true);
    assert!(!stdout.contains("\u{1b}["), "no escapes in JSON mode");
}

#[test]
fn piped_output_is_clean_even_without_no_color() {
    // cargo test stdout is not a TTY → OutputMode auto-flips to plain.
    let actual = Command::cargo_bin("gb")
        .unwrap()
        .args(["status"])
        .current_dir(common::init_repo().path())
        .output()
        .unwrap();
    let stdout = String::from_utf8(actual.stdout).unwrap();
    assert!(
        !stdout.contains("\u{1b}["),
        "piped output must be plain even without NO_COLOR; got: {stdout}"
    );
}
