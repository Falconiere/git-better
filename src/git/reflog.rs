use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ReflogEntry {
    pub sha: String,
    pub ref_selector: String,
    pub action: String,
    pub time: String,
}

pub fn parse_reflog(input: &str) -> Vec<ReflogEntry> {
    input.lines().filter_map(parse_reflog_line).collect()
}

fn parse_reflog_line(line: &str) -> Option<ReflogEntry> {
    let first_space = line.find(' ')?;
    let sha = line[..first_space].trim().to_string();
    let rest = &line[first_space + 1..];
    let close_brace = rest.rfind("}: ")?;
    let ref_sel = rest[..close_brace + 1].to_string();
    let action = rest[close_brace + 3..].to_string();
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
