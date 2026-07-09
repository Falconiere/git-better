use git_better::git::diff::{
  FileStat, parse_numstat, parse_unified_diff, summarize, truncate_unified_diff,
};

#[test]
fn parse_numstat_simple() {
  let s = "5\t3\tsrc/main.rs\n2\t1\tREADME.md\n";
  let v = parse_numstat(s);
  assert_eq!(v.len(), 2);
  assert_eq!(v[0].path, "src/main.rs");
  assert_eq!(v[0].added, 5);
  assert_eq!(v[0].removed, 3);
}

#[test]
fn parse_numstat_skips_binary() {
  let s = "5\t3\tsrc/main.rs\n-\t-\timage.png\n";
  let v = parse_numstat(s);
  assert_eq!(v.len(), 1);
}

#[test]
fn parse_numstat_handles_rename() {
  let s = "5\t3\told.rs => new.rs\n";
  let v = parse_numstat(s);
  assert_eq!(v[0].path, "new.rs");
}

#[test]
fn summarize_aggregates() {
  let v = vec![
    FileStat {
      path: "a".into(),
      added: 5,
      removed: 3,
    },
    FileStat {
      path: "b".into(),
      added: 10,
      removed: 2,
    },
  ];
  let s = summarize(&v);
  assert_eq!(s.files_changed, 2);
  assert_eq!(s.added, 15);
  assert_eq!(s.removed, 5);
}

#[test]
fn parse_unified_diff_minimal() {
  let s = "--- a/foo.rs\n+++ b/foo.rs\n@@ -1,3 +1,3 @@\n line1\n-old\n+new\n line3\n";
  let files = parse_unified_diff(s);
  assert_eq!(files.len(), 1);
  assert_eq!(files[0].old_path, "foo.rs");
  assert_eq!(files[0].new_path, "foo.rs");
  assert_eq!(files[0].hunks.len(), 1);
  assert_eq!(files[0].hunks[0].lines.len(), 4);
}

#[test]
fn truncate_under_budget_is_passthrough() {
  let s = "--- a/x\n+++ b/x\n@@ -0,0 +1,1 @@\n+hello\n";
  let (out, paths, truncated) = truncate_unified_diff(s, 100);
  assert!(!truncated);
  assert!(paths.is_empty());
  assert_eq!(out, s);
}

#[test]
fn truncate_over_budget_marks_excess() {
  let mut big = String::new();
  for i in 0..20 {
    big.push_str(&format!(
            "--- a/file{i}\n+++ b/file{i}\n@@ -0,0 +1,1 @@\n+content for file {i} is long enough to consume tokens\n"
        ));
  }
  let (_out, paths, truncated) = truncate_unified_diff(&big, 50);
  assert!(truncated);
  assert!(!paths.is_empty());
}
