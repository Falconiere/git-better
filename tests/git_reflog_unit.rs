use git_better::git::reflog::parse_reflog;

#[test]
fn parse_reflog_relative_date() {
  let line = "a1b2c3d HEAD@{2 hours ago}: commit: feat(auth): add OAuth2";
  let v = parse_reflog(line);
  assert_eq!(v.len(), 1);
  assert_eq!(v[0].sha, "a1b2c3d");
  assert_eq!(v[0].ref_selector, "HEAD@{2 hours ago}");
  assert_eq!(v[0].time, "2 hours ago");
  assert_eq!(v[0].action, "commit: feat(auth): add OAuth2");
}

#[test]
fn parse_reflog_no_date() {
  let line = "a1b2c3d HEAD@{0}: commit: init";
  let v = parse_reflog(line);
  assert_eq!(v.len(), 1);
  assert_eq!(v[0].sha, "a1b2c3d");
  assert_eq!(v[0].action, "commit: init");
  assert_eq!(v[0].time, "");
}

#[test]
fn parse_reflog_iso_date() {
  let line = "a1b2c3d HEAD@{2025-01-15 10:30:00 +0000}: commit: feat: add";
  let v = parse_reflog(line);
  assert_eq!(v.len(), 1);
  assert_eq!(v[0].time, "2025-01-15 10:30:00 +0000");
}
