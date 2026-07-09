use crate::git::commit::CommitRecord;
use crate::git::diff::FileStat;

/// Parses `git`'s numstat output into per-file statistics.
pub fn parse_numstat(input: &str) -> Vec<FileStat> {
  crate::git::diff::parse_numstat(input)
}

/// Creates an empty [`CommitRecord`] with all fields cleared.
fn empty_commit_record() -> CommitRecord {
  CommitRecord {
    sha: String::new(),
    short_sha: String::new(),
    author_name: String::new(),
    author_email: String::new(),
    time_iso: String::new(),
    time_relative: String::new(),
    subject: String::new(),
    body: String::new(),
    conventional_type: None,
    conventional_scope: None,
    pr_number: None,
  }
}

/// Applies a single field line to `c`; returns `true` when body mode should start.
fn apply_field_line(c: &mut CommitRecord, line: &str) -> bool {
  if let Some(rest) = line.strip_prefix("SHA:") {
    c.sha = rest.trim().to_string();
    c.short_sha = c.sha.chars().take(7).collect();
  } else if let Some(rest) = line.strip_prefix("AUTHOR:") {
    c.author_name = rest.trim().to_string();
  } else if let Some(rest) = line.strip_prefix("EMAIL:") {
    c.author_email = rest.trim().to_string();
  } else if let Some(rest) = line.strip_prefix("ISO:") {
    c.time_iso = rest.trim().to_string();
  } else if let Some(rest) = line.strip_prefix("REL:") {
    c.time_relative = rest.trim().to_string();
  } else if let Some(rest) = line.strip_prefix("SUBJECT:") {
    c.subject = rest.trim().to_string();
  } else if line == "BODY:" {
    return true;
  }
  false
}

/// Enriches a record in place with detected conventional-commit and PR metadata.
fn enrich_record(c: &mut CommitRecord) {
  let (t, s) = crate::git::commit::detect_conventional(&c.subject);
  c.conventional_type = t;
  c.conventional_scope = s;
  c.pr_number = crate::git::commit::detect_pr_number(&c.subject, &c.body);
}

/// Parses the delimited `git log` format into a list of [`CommitRecord`] values.
pub fn parse_log_format(input: &str) -> Vec<CommitRecord> {
  let mut records = Vec::new();
  let mut current: Option<CommitRecord> = None;
  let mut in_body = false;

  let mut commit_seen = false;

  for line in input.lines() {
    if line.starts_with("COMMIT<<<") {
      if let Some(c) = current.take() {
        records.push(c);
      }
      current = Some(empty_commit_record());
      in_body = false;
      commit_seen = true;
      continue;
    }
    if !commit_seen {
      continue;
    }
    let c = match current.as_mut() {
      Some(c) => c,
      None => break,
    };
    if in_body {
      if !c.body.is_empty() {
        c.body.push('\n');
      }
      c.body.push_str(line);
    } else if apply_field_line(c, line) {
      in_body = true;
    }
  }
  if let Some(c) = current.take() {
    records.push(c);
  }

  for c in &mut records {
    enrich_record(c);
  }

  records
}
