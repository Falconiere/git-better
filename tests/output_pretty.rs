mod common;

use assert_cmd::Command;
use git_better::output::OutputMode;

#[test]
fn output_mode_plain_disables_pretty() {
    assert_eq!(
        OutputMode::from_flags(true, false),
        OutputMode::Human { pretty: false }
    );
}

#[test]
fn output_mode_better_overrides_plain() {
    assert_eq!(
        OutputMode::from_flags(true, true),
        OutputMode::Better { budget: None }
    );
}

#[test]
fn output_mode_default_in_test_is_plain() {
    assert_eq!(
        OutputMode::from_flags(false, false),
        OutputMode::Human { pretty: false },
        "cargo test stdout is not a TTY, so default should be plain"
    );
}

#[test]
fn gb_status_no_color_env_matches_git() {
    let dir = common::init_repo();
    let expected = common::git_stdout(&["status", "-sb"], dir.path());
    let actual = Command::cargo_bin("gb")
        .unwrap()
        .env("NO_COLOR", "1")
        .args(["status"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(actual.status.success());
    assert_eq!(String::from_utf8(actual.stdout).unwrap(), expected);
}

#[test]
fn gb_status_pretty_default_does_not_break_output() {
    // M0: pretty mode and plain mode produce identical text.
    // M1: this test will additionally assert presence of color escapes
    // in a forced-TTY context.
    let dir = common::init_repo();
    let expected = common::git_stdout(&["status", "-sb"], dir.path());
    let actual = Command::cargo_bin("gb")
        .unwrap()
        .args(["status"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert_eq!(String::from_utf8(actual.stdout).unwrap(), expected);
}
