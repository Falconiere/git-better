use serde::Serialize;

/// Per-file added/removed line counts parsed from numstat output.
#[derive(Debug, Clone, Serialize)]
pub struct FileStat {
  /// File path (rename-normalized).
  pub path: String,
  /// Number of lines added.
  pub added: u64,
  /// Number of lines removed.
  pub removed: u64,
}

/// Aggregate summary across a set of [`FileStat`] entries.
#[derive(Debug, Clone, Serialize, Default)]
pub struct DiffSummary {
  /// Total number of files changed.
  pub files_changed: u64,
  /// Total number of lines added.
  pub added: u64,
  /// Total number of lines removed.
  pub removed: u64,
}

/// A single line within a diff hunk, tagged by its change kind.
#[derive(Debug, Clone)]
pub enum DiffLine {
  /// Unchanged context line.
  Context(String),
  /// Added (`+`) line.
  Insertion(String),
  /// Removed (`-`) line.
  Deletion(String),
}

impl DiffLine {
  /// Returns the line's text content without its change sign.
  pub fn content(&self) -> &str {
    match self {
      DiffLine::Context(s) | DiffLine::Insertion(s) | DiffLine::Deletion(s) => s,
    }
  }
  /// Returns the leading diff sign character for the line.
  pub fn sign(&self) -> char {
    match self {
      DiffLine::Context(_) => ' ',
      DiffLine::Insertion(_) => '+',
      DiffLine::Deletion(_) => '-',
    }
  }
}

/// A diff hunk: its `@@` header plus the lines it contains.
#[derive(Debug, Clone)]
pub struct Hunk {
  /// The hunk header line (e.g. `@@ -1,3 +1,3 @@`).
  pub header: String,
  /// Lines belonging to the hunk.
  pub lines: Vec<DiffLine>,
}

/// A single file's diff: old/new paths plus its hunks.
#[derive(Debug, Clone)]
pub struct DiffFile {
  /// Path on the old side (`a/`).
  pub old_path: String,
  /// Path on the new side (`b/`).
  pub new_path: String,
  /// Hunks comprising the file's changes.
  pub hunks: Vec<Hunk>,
}

impl DiffFile {
  /// Returns the path to display, preferring the new path unless it is `/dev/null`.
  pub fn display_path(&self) -> &str {
    if self.new_path == "/dev/null" {
      &self.old_path
    } else {
      &self.new_path
    }
  }
}

/// Parses `git --numstat` output into a list of [`FileStat`] entries.
pub fn parse_numstat(output: &str) -> Vec<FileStat> {
  output
    .lines()
    .filter_map(|line| {
      let parts: Vec<&str> = line.split('\t').collect();
      if parts.len() < 3 {
        return None;
      }
      let added = parts[0].parse::<u64>().unwrap_or(0);
      let removed = parts[1].parse::<u64>().unwrap_or(0);
      let raw_path = parts[2];
      if added == 0 && removed == 0 && parts[0] == "-" && parts[1] == "-" {
        return None;
      }
      let path = normalize_path(raw_path);
      Some(FileStat {
        path,
        added,
        removed,
      })
    })
    .collect()
}

/// Normalizes a numstat path, resolving git rename `{old => new}` notation.
pub fn normalize_path(raw: &str) -> String {
  if let Some(open) = raw.find('{') {
    let rest = &raw[open + 1..];
    if let Some(arrow) = rest.find(" => ") {
      let after = &rest[arrow + " => ".len()..];
      if let Some(close) = after.find('}') {
        let prefix = &raw[..open];
        let new = &after[..close];
        let suffix = &after[close + 1..];
        return format!("{prefix}{new}{suffix}");
      }
    }
  }
  if let Some((_, new)) = raw.split_once(" => ") {
    return new.to_string();
  }
  raw.to_string()
}

/// Aggregates per-file stats into a single [`DiffSummary`].
pub fn summarize(files: &[FileStat]) -> DiffSummary {
  DiffSummary {
    files_changed: files.len() as u64,
    added: files.iter().map(|f| f.added).sum(),
    removed: files.iter().map(|f| f.removed).sum(),
  }
}

/// Classifies a hunk content line into a [`DiffLine`], or `None` if it is not one.
fn classify_diff_line(line: &str) -> Option<DiffLine> {
  if let Some(rest) = line.strip_prefix('+') {
    Some(DiffLine::Insertion(rest.to_string()))
  } else if let Some(rest) = line.strip_prefix('-') {
    Some(DiffLine::Deletion(rest.to_string()))
  } else if let Some(rest) = line.strip_prefix(' ') {
    Some(DiffLine::Context(rest.to_string()))
  } else if line.is_empty() {
    Some(DiffLine::Context(String::new()))
  } else {
    None
  }
}

/// Flushes a pending hunk into the current file, if both are present.
fn flush_hunk(current_file: &mut Option<DiffFile>, current_hunk: &mut Option<Hunk>) {
  if let Some(h) = current_hunk.take() {
    if let Some(f) = current_file.as_mut() {
      f.hunks.push(h);
    }
  }
}

/// Parses a unified diff into a list of [`DiffFile`] values.
pub fn parse_unified_diff(input: &str) -> Vec<DiffFile> {
  let mut files = Vec::new();
  let mut current_file: Option<DiffFile> = None;
  let mut current_hunk: Option<Hunk> = None;

  let lines: Vec<&str> = input.lines().collect();
  let mut i = 0;
  while i < lines.len() {
    let line = lines[i];
    let is_file_header =
      line.starts_with("--- ") && lines.get(i + 1).is_some_and(|n| n.starts_with("+++ "));
    if is_file_header {
      flush_hunk(&mut current_file, &mut current_hunk);
      if let Some(f) = current_file.take() {
        files.push(f);
      }
      let old_path = line
        .strip_prefix("--- ")
        .unwrap_or(line)
        .trim()
        .trim_start_matches("a/");
      let next = lines[i + 1];
      let new_path = next
        .strip_prefix("+++ ")
        .unwrap_or(next)
        .trim()
        .trim_start_matches("b/");
      current_file = Some(DiffFile {
        old_path: old_path.to_string(),
        new_path: new_path.to_string(),
        hunks: Vec::new(),
      });
      i += 2;
      continue;
    }
    if line.starts_with("@@") {
      flush_hunk(&mut current_file, &mut current_hunk);
      current_hunk = Some(Hunk {
        header: line.to_string(),
        lines: Vec::new(),
      });
    } else if let Some(h) = current_hunk.as_mut() {
      if let Some(diff_line) = classify_diff_line(line) {
        h.lines.push(diff_line);
      }
    }
    i += 1;
  }
  flush_hunk(&mut current_file, &mut current_hunk);
  if let Some(f) = current_file.take() {
    files.push(f);
  }

  files
}

/// Estimates the rendered character size of a [`DiffFile`].
pub fn estimate_file_size(file: &DiffFile) -> usize {
  let header_size = file.old_path.len() + file.new_path.len() + 16;
  let hunk_size: usize = file
    .hunks
    .iter()
    .map(|h| h.header.len() + 1 + h.lines.iter().map(|l| l.content().len() + 2).sum::<usize>())
    .sum();
  header_size + hunk_size
}

/// Renders a [`DiffFile`] back into unified-diff text.
pub fn render_file(file: &DiffFile) -> String {
  let mut out = String::new();
  out.push_str(&format!("--- a/{}\n", file.old_path));
  out.push_str(&format!("+++ b/{}\n", file.new_path));
  for hunk in &file.hunks {
    out.push_str(&hunk.header);
    out.push('\n');
    for line in &hunk.lines {
      out.push(line.sign());
      out.push_str(line.content());
      out.push('\n');
    }
  }
  out
}

/// Truncates a unified diff to a token budget, returning the kept text, the
/// truncated file paths, and whether truncation occurred.
pub fn truncate_unified_diff(input: &str, budget_tokens: usize) -> (String, Vec<String>, bool) {
  let char_budget = budget_tokens.saturating_mul(4);
  if input.len() <= char_budget {
    return (input.to_string(), Vec::new(), false);
  }

  let files = parse_unified_diff(input);
  if files.is_empty() {
    let mut cut = char_budget.min(input.len());
    while cut > 0 && !input.is_char_boundary(cut) {
      cut -= 1;
    }
    let mut kept = input[..cut].to_string();
    kept.push_str("\n... [truncated]\n");
    return (kept, Vec::new(), true);
  }
  let mut kept = String::new();
  let mut truncated_paths = Vec::new();
  let file_sizes: Vec<usize> = files.iter().map(estimate_file_size).collect();
  let mut used = 0usize;
  let mut last_kept_index: Option<usize> = None;

  for (i, &size) in file_sizes.iter().enumerate() {
    if used + size > char_budget {
      break;
    }
    used += size;
    last_kept_index = Some(i);
  }

  for (i, file) in files.iter().enumerate() {
    match last_kept_index {
      Some(last) if i <= last => {
        kept.push_str(&render_file(file));
      },
      _ => {
        truncated_paths.push(file.display_path().to_string());
      },
    }
  }

  if !truncated_paths.is_empty() {
    kept.push_str(&format!(
      "\n... [truncated, {} more file(s); use `gb diff --full <path>` to see each one]\n",
      truncated_paths.len()
    ));
  }

  let truncated = !truncated_paths.is_empty();
  (kept, truncated_paths, truncated)
}
