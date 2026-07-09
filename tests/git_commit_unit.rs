use git_better::git::commit::{detect_conventional, detect_pr_number};

#[test]
fn detect_conventional_basic() {
  assert_eq!(
    detect_conventional("feat(auth): add OAuth2 PKCE flow"),
    (Some("feat".into()), Some("auth".into()))
  );
  assert_eq!(
    detect_conventional("fix: handle expired verifier"),
    (Some("fix".into()), None)
  );
  assert_eq!(detect_conventional("wip: something"), (None, None));
}

#[test]
fn detect_pr_in_subject() {
  assert_eq!(
    detect_pr_number("feat(auth): add OAuth2 PKCE flow (#142)", ""),
    Some(142)
  );
}

#[test]
fn detect_pr_in_body() {
  assert_eq!(
    detect_pr_number("feat: add thing", "Fixes #99\nCo-authored-by: foo"),
    Some(99)
  );
}
