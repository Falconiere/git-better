use crate::git::commit::CommitRecord;
use crate::git::diff::FileStat;

pub fn parse_numstat(input: &str) -> Vec<FileStat> {
    crate::git::diff::parse_numstat(input)
}

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
            current = Some(CommitRecord {
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
            });
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
            in_body = false;
        } else if line == "BODY:" {
            in_body = true;
        } else if in_body {
            if !c.body.is_empty() {
                c.body.push('\n');
            }
            c.body.push_str(line);
        }
    }
    if let Some(c) = current.take() {
        records.push(c);
    }

    for c in &mut records {
        let (t, s) = crate::git::commit::detect_conventional(&c.subject);
        c.conventional_type = t;
        c.conventional_scope = s;
        c.pr_number = crate::git::commit::detect_pr_number(&c.subject, &c.body);
    }

    records
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
