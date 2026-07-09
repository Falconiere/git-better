mod common;

use assert_cmd::Command;

#[test]
fn gb_ascii_env_keeps_output_ascii() {
  let actual = Command::cargo_bin("gb")
    .unwrap()
    .env("GB_ASCII", "1")
    .args(["status"])
    .current_dir(common::init_repo().path())
    .output()
    .unwrap();
  let stdout = String::from_utf8(actual.stdout).unwrap();
  // In M0, status output is raw git text — so this is trivially true.
  // M1 will tighten this to also assert absence of ●/◐/⇡/⇣/✨ after
  // prettification lands.
  for ch in ["●", "◐", "⇡", "⇣", "✨", "🐛", "📝"] {
    assert!(
      !stdout.contains(ch),
      "found forbidden unicode glyph {ch} in {stdout}"
    );
  }
}

#[test]
fn gb_ascii_env_with_better_still_valid_json() {
  let actual = Command::cargo_bin("gb")
    .unwrap()
    .env("GB_ASCII", "1")
    .args(["--better", "status"])
    .current_dir(common::init_repo().path())
    .output()
    .unwrap();
  let stdout = String::from_utf8(actual.stdout).unwrap();
  assert!(stdout.is_ascii(), "GB_ASCII=1 must produce ASCII output; got: {stdout}");
  let v: serde_json::Value = serde_json::from_str(&stdout).expect("JSON");
  assert_eq!(v["ok"], true);
}

#[test]
fn unit_icons_default_unicode_when_no_env() {
  // This test relies on GB_ASCII NOT being set in the inherited env.
  // We can't safely mutate env from inside a test in recent Rust,
  // so we just assert that without the var, default is unicode.
  if std::env::var_os("GB_ASCII").is_none() {
    let ic = git_better::output::icons::detect(false);
    assert_eq!(ic.staged, "●");
    assert_eq!(ic.untracked, "?");
    assert_eq!(ic.type_feat, "✨ feat");
    assert_eq!(ic.type_fix, "🐛 fix");
  }
}

#[test]
fn unit_type_tag_mapping() {
  let ic = git_better::output::icons::detect(false);
  assert_eq!(
    git_better::output::icons::type_tag(&ic, "feat"),
    ic.type_feat
  );
  assert_eq!(git_better::output::icons::type_tag(&ic, "fix"), ic.type_fix);
  assert_eq!(
    git_better::output::icons::type_tag(&ic, "docs"),
    ic.type_docs
  );
  assert_eq!(
    git_better::output::icons::type_tag(&ic, "refactor"),
    ic.type_refactor
  );
  assert_eq!(
    git_better::output::icons::type_tag(&ic, "perf"),
    ic.type_perf
  );
  assert_eq!(
    git_better::output::icons::type_tag(&ic, "test"),
    ic.type_test
  );
  assert_eq!(
    git_better::output::icons::type_tag(&ic, "chore"),
    ic.type_chore
  );
  assert_eq!(
    git_better::output::icons::type_tag(&ic, "build"),
    ic.type_build
  );
  assert_eq!(git_better::output::icons::type_tag(&ic, "ci"), ic.type_ci);
  assert_eq!(
    git_better::output::icons::type_tag(&ic, "unknown-type"),
    ic.type_other
  );
}
