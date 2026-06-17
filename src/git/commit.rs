use once_cell::sync::Lazy;
use regex::Regex;
use serde::Serialize;

static CONVENTIONAL_RE: Lazy<Regex> = Lazy::new(|| {
  Regex::new(
        r"^(?P<type>feat|fix|docs|style|refactor|perf|test|chore|build|ci|revert)(?:\((?P<scope>[^)]+)\))?(?P<bang>!)?:\s*(?P<subject>.+?)\s*$",
    )
    .unwrap()
});

static PR_TRAILER_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"#(\d+)").unwrap());

#[derive(Debug, Clone, Serialize)]
pub struct CommitRecord {
  pub sha: String,
  pub short_sha: String,
  pub author_name: String,
  pub author_email: String,
  pub time_iso: String,
  pub time_relative: String,
  pub subject: String,
  pub body: String,
  pub conventional_type: Option<String>,
  pub conventional_scope: Option<String>,
  pub pr_number: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NetChange {
  pub added: u64,
  pub removed: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CommitGroup {
  pub pr: Option<u64>,
  pub title: String,
  pub conventional_type: Option<String>,
  pub conventional_scope: Option<String>,
  pub commits: Vec<String>,
  pub files_touched: u64,
  pub net: NetChange,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct LogStats {
  pub total: u64,
  pub by_type: std::collections::BTreeMap<String, u64>,
  pub by_author: std::collections::BTreeMap<String, u64>,
}

pub fn detect_conventional(subject: &str) -> (Option<String>, Option<String>) {
  if let Some(caps) = CONVENTIONAL_RE.captures(subject.trim()) {
    let t = caps.name("type").map(|m| m.as_str().to_string());
    let s = caps.name("scope").map(|m| m.as_str().to_string());
    return (t, s);
  }
  (None, None)
}

pub fn detect_pr_number(subject: &str, body: &str) -> Option<u64> {
  if let Some(caps) = PR_TRAILER_RE.captures(subject) {
    if let Some(m) = caps.get(1) {
      return m.as_str().parse().ok();
    }
  }
  if let Some(caps) = PR_TRAILER_RE.captures(body) {
    if let Some(m) = caps.get(1) {
      return m.as_str().parse().ok();
    }
  }
  None
}

#[cfg(test)]
mod tests {
  use super::*;

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
}
