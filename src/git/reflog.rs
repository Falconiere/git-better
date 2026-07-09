use serde::Serialize;

/// A single parsed entry from `git reflog`.
#[derive(Debug, Clone, Serialize)]
pub struct ReflogEntry {
  /// Commit SHA the entry points to.
  pub sha: String,
  /// Reflog selector (e.g. `HEAD@{2 hours ago}`).
  pub ref_selector: String,
  /// Action description that produced the entry.
  pub action: String,
  /// Human-readable time component, empty when only an index is present.
  pub time: String,
}

/// Parses `git reflog` output into a list of [`ReflogEntry`] values.
pub fn parse_reflog(input: &str) -> Vec<ReflogEntry> {
  input.lines().filter_map(parse_reflog_line).collect()
}

fn parse_reflog_line(line: &str) -> Option<ReflogEntry> {
  let first_space = line.find(' ')?;
  let sha = line[..first_space].trim().to_string();
  let rest = &line[first_space + 1..];
  let close_brace = rest.find("}: ")?;
  let ref_sel = rest[..close_brace + 1].to_string();
  let action = rest[close_brace + 2..].trim().to_string();
  let time = if let Some(start) = ref_sel.find('{') {
    let inside = &ref_sel[start + 1..ref_sel.len() - 1];
    if inside.chars().all(|c| c.is_ascii_digit()) {
      String::new()
    } else {
      inside.to_string()
    }
  } else {
    String::new()
  };
  Some(ReflogEntry {
    sha,
    ref_selector: ref_sel,
    action,
    time,
  })
}
