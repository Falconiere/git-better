use regex::Regex;
use serde::Serialize;
use std::sync::LazyLock;

static CONVENTIONAL_RE: LazyLock<Option<Regex>> = LazyLock::new(|| {
  Regex::new(
    r"^(?P<type>feat|fix|docs|style|refactor|perf|test|chore|build|ci|revert)(?:\((?P<scope>[^)]+)\))?(?P<bang>!)?:\s*(?P<subject>.+?)\s*$",
  )
  .ok()
});

static PR_TRAILER_RE: LazyLock<Option<Regex>> = LazyLock::new(|| Regex::new(r"#(\d+)").ok());

/// A single parsed commit with its metadata and derived conventional-commit fields.
#[derive(Debug, Clone, Serialize)]
pub struct CommitRecord {
  /// Full commit SHA.
  pub sha: String,
  /// Abbreviated 7-character commit SHA.
  pub short_sha: String,
  /// Author display name.
  pub author_name: String,
  /// Author email address.
  pub author_email: String,
  /// Author timestamp in ISO 8601 format.
  pub time_iso: String,
  /// Author timestamp as a human-relative string (e.g. "2 hours ago").
  pub time_relative: String,
  /// Commit subject (first line of the message).
  pub subject: String,
  /// Commit body (remaining lines of the message).
  pub body: String,
  /// Detected conventional-commit type, if any.
  pub conventional_type: Option<String>,
  /// Detected conventional-commit scope, if any.
  pub conventional_scope: Option<String>,
  /// Detected pull-request number, if any.
  pub pr_number: Option<u64>,
}

/// Net line counts for a change (lines added and removed).
#[derive(Debug, Clone, Serialize)]
pub struct NetChange {
  /// Number of lines added.
  pub added: u64,
  /// Number of lines removed.
  pub removed: u64,
}

/// A group of commits aggregated under a single pull request or title.
#[derive(Debug, Clone, Serialize)]
pub struct CommitGroup {
  /// Associated pull-request number, if any.
  pub pr: Option<u64>,
  /// Display title for the group.
  pub title: String,
  /// Conventional-commit type for the group, if any.
  pub conventional_type: Option<String>,
  /// Conventional-commit scope for the group, if any.
  pub conventional_scope: Option<String>,
  /// SHAs of the commits in this group.
  pub commits: Vec<String>,
  /// Number of distinct files touched by the group.
  pub files_touched: u64,
  /// Net line counts across the group.
  pub net: NetChange,
}

/// Aggregate statistics over a set of parsed commits.
#[derive(Debug, Clone, Serialize, Default)]
pub struct LogStats {
  /// Total number of commits.
  pub total: u64,
  /// Commit counts grouped by conventional-commit type.
  pub by_type: std::collections::BTreeMap<String, u64>,
  /// Commit counts grouped by author.
  pub by_author: std::collections::BTreeMap<String, u64>,
}

/// Detects the conventional-commit type and scope from a commit subject.
pub fn detect_conventional(subject: &str) -> (Option<String>, Option<String>) {
  let Some(re) = CONVENTIONAL_RE.as_ref() else {
    return (None, None);
  };
  if let Some(caps) = re.captures(subject.trim()) {
    let t = caps.name("type").map(|m| m.as_str().to_string());
    let s = caps.name("scope").map(|m| m.as_str().to_string());
    return (t, s);
  }
  (None, None)
}

/// Detects a pull-request number from a commit subject, falling back to the body.
pub fn detect_pr_number(subject: &str, body: &str) -> Option<u64> {
  let re = PR_TRAILER_RE.as_ref()?;
  if let Some(caps) = re.captures(subject) {
    if let Some(m) = caps.get(1) {
      return m.as_str().parse().ok();
    }
  }
  if let Some(caps) = re.captures(body) {
    if let Some(m) = caps.get(1) {
      return m.as_str().parse().ok();
    }
  }
  None
}
