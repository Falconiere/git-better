use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct FileStat {
    pub path: String,
    pub added: u64,
    pub removed: u64,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct DiffSummary {
    pub files_changed: u64,
    pub added: u64,
    pub removed: u64,
}

#[derive(Debug, Clone)]
pub enum DiffLine {
    Context(String),
    Insertion(String),
    Deletion(String),
}

impl DiffLine {
    pub fn content(&self) -> &str {
        match self {
            DiffLine::Context(s) | DiffLine::Insertion(s) | DiffLine::Deletion(s) => s,
        }
    }
    pub fn sign(&self) -> char {
        match self {
            DiffLine::Context(_) => ' ',
            DiffLine::Insertion(_) => '+',
            DiffLine::Deletion(_) => '-',
        }
    }
}

#[derive(Debug, Clone)]
pub struct Hunk {
    pub header: String,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone)]
pub struct DiffFile {
    pub old_path: String,
    pub new_path: String,
    pub hunks: Vec<Hunk>,
}

impl DiffFile {
    pub fn display_path(&self) -> &str {
        if self.new_path == "/dev/null" {
            &self.old_path
        } else {
            &self.new_path
        }
    }
}

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

pub fn normalize_path(raw: &str) -> String {
    if raw.contains(" => ") {
        let parts: Vec<&str> = raw.split(" => ").collect();
        if parts.len() == 2 {
            return parts[1].to_string();
        }
        if let Some(stripped) = raw.strip_prefix("{") {
            if let Some(end) = stripped.find(" => ") {
                let after = &stripped[end + " => ".len()..];
                if let Some(close) = after.find('}') {
                    let prefix = &stripped[..end];
                    let inner = &after[close + 1..];
                    return format!("{prefix}{inner}");
                }
            }
        }
    }
    raw.to_string()
}

pub fn summarize(files: &[FileStat]) -> DiffSummary {
    DiffSummary {
        files_changed: files.len() as u64,
        added: files.iter().map(|f| f.added).sum(),
        removed: files.iter().map(|f| f.removed).sum(),
    }
}

pub fn parse_unified_diff(input: &str) -> Vec<DiffFile> {
    let mut files = Vec::new();
    let mut current_file: Option<DiffFile> = None;
    let mut current_hunk: Option<Hunk> = None;

    for line in input.lines() {
        if let Some(rest) = line.strip_prefix("--- ") {
            if let Some(h) = current_hunk.take() {
                if let Some(f) = current_file.as_mut() {
                    f.hunks.push(h);
                }
            }
            if let Some(f) = current_file.take() {
                files.push(f);
            }
            let old_path = rest.trim().trim_start_matches("a/").to_string();
            current_file = Some(DiffFile {
                old_path,
                new_path: String::new(),
                hunks: Vec::new(),
            });
        } else if let Some(rest) = line.strip_prefix("+++ ") {
            if let Some(f) = current_file.as_mut() {
                f.new_path = rest.trim().trim_start_matches("b/").to_string();
            }
        } else if line.starts_with("@@") {
            if let Some(h) = current_hunk.take() {
                if let Some(f) = current_file.as_mut() {
                    f.hunks.push(h);
                }
            }
            current_hunk = Some(Hunk {
                header: line.to_string(),
                lines: Vec::new(),
            });
        } else if let Some(h) = current_hunk.as_mut() {
            if let Some(rest) = line.strip_prefix('+') {
                h.lines.push(DiffLine::Insertion(rest.to_string()));
            } else if let Some(rest) = line.strip_prefix('-') {
                h.lines.push(DiffLine::Deletion(rest.to_string()));
            } else if let Some(rest) = line.strip_prefix(' ') {
                h.lines.push(DiffLine::Context(rest.to_string()));
            } else if line.is_empty() {
                h.lines.push(DiffLine::Context(String::new()));
            }
        }
    }
    if let Some(h) = current_hunk.take() {
        if let Some(f) = current_file.as_mut() {
            f.hunks.push(h);
        }
    }
    if let Some(f) = current_file.take() {
        files.push(f);
    }

    files
}

pub fn estimate_file_size(file: &DiffFile) -> usize {
    let header_size = file.old_path.len() + file.new_path.len() + 16;
    let hunk_size: usize = file
        .hunks
        .iter()
        .map(|h| h.header.len() + 1 + h.lines.iter().map(|l| l.content().len() + 2).sum::<usize>())
        .sum();
    header_size + hunk_size
}

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

pub fn truncate_unified_diff(input: &str, budget_tokens: usize) -> (String, Vec<String>, bool) {
    let char_budget = budget_tokens.saturating_mul(4);
    if input.len() <= char_budget {
        return (input.to_string(), Vec::new(), false);
    }

    let files = parse_unified_diff(input);
    let mut kept = String::new();
    let mut truncated_paths = Vec::new();
    let mut used = 0usize;
    let mut last_kept_index: Option<usize> = None;

    for (i, file) in files.iter().enumerate() {
        let size = estimate_file_size(file);
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
            }
            _ => {
                truncated_paths.push(file.display_path().to_string());
            }
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
