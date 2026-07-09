use git_better::git::porcelain::parse_log_format;

#[test]
fn parse_log_format_extracts_conventional_metadata() {
  let input = "\
COMMIT<<<
SHA:abc1234deadbeef
AUTHOR:Alex Doe
EMAIL:alex@foo.com
ISO:2025-01-15T10:30:00+00:00
REL:2 hours ago
SUBJECT:feat(auth): add OAuth2 PKCE flow
BODY:
Co-authored-by: Sam <sam@foo.com>

Ref: (#142)
";
  let v = parse_log_format(input);
  assert_eq!(v.len(), 1);
  assert_eq!(v[0].sha, "abc1234deadbeef");
  assert_eq!(v[0].author_name, "Alex Doe");
  assert_eq!(v[0].conventional_type, Some("feat".into()));
  assert_eq!(v[0].conventional_scope, Some("auth".into()));
  assert_eq!(v[0].pr_number, Some(142));
}
