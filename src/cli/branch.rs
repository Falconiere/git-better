use anyhow::Result;
use serde_json::json;
use std::collections::BTreeMap;

use super::BranchArgs;
use crate::git::proc;
use crate::output::{OutputMode, better, icons, theme};

#[derive(Debug, Clone, serde::Serialize)]
struct BranchRow {
  name: String,
  current: bool,
  upstream: Option<String>,
  ahead: i64,
  behind: i64,
  last_commit_relative: Option<String>,
  last_commit_iso: Option<String>,
  stale: bool,
}

/// Dispatch `gb branch`: passthrough, structured `--better`, or the pretty list.
pub fn run(args: BranchArgs, mode: OutputMode) -> Result<()> {
  if !args.passthrough.is_empty() {
    let mut all = vec!["branch".to_string()];
    all.extend(args.passthrough);
    let out = proc::run_git(&all)?;
    proc::write_to_stdout(&out);
    if !out.ends_with('\n') {
      println!();
    }
    return Ok(());
  }

  if mode.is_better() {
    return run_better(args, mode);
  }

  run_pretty(args, mode)
}

fn collect_rows() -> Result<Vec<BranchRow>> {
  let current = proc::run_git(&["branch".into(), "--show-current".into()])
    .ok()
    .map(|s| s.trim().to_string())
    .filter(|s| !s.is_empty());

  let format_out = proc::run_git(&[
        "for-each-ref".into(),
        "--format=%(refname:short)%00%(upstream:short)%00%(upstream:track)%00%(committerdate:relative)%00%(committerdate:iso)".into(),
        "refs/heads".into(),
    ])?;
  let mut rows = Vec::new();
  for line in format_out.lines() {
    if let Some(row) = parse_branch_row(line, current.as_ref()) {
      rows.push(row);
    }
  }
  Ok(rows)
}

fn parse_branch_row(line: &str, current: Option<&String>) -> Option<BranchRow> {
  if line.is_empty() {
    return None;
  }
  let parts: Vec<&str> = line.split('\0').collect();
  if parts.len() < 5 {
    return None;
  }
  let name = parts[0].to_string();
  let upstream = if parts[1].is_empty() {
    None
  } else {
    Some(parts[1].to_string())
  };
  let (ahead, behind) = parse_track(parts[2]);
  let last_rel = if parts[3] == "-" {
    None
  } else {
    Some(parts[3].to_string())
  };
  let last_iso = if parts[4].is_empty() {
    None
  } else {
    Some(parts[4].to_string())
  };
  let is_current = Some(&name) == current;
  let stale = last_iso.as_deref().map(is_stale_iso).unwrap_or(false);
  Some(BranchRow {
    name,
    current: is_current,
    upstream,
    ahead,
    behind,
    last_commit_relative: last_rel,
    last_commit_iso: last_iso,
    stale,
  })
}

fn parse_track(track: &str) -> (i64, i64) {
  if track.is_empty() {
    return (0, 0);
  }
  let mut ahead = 0i64;
  let mut behind = 0i64;
  for token in track.split(", ") {
    if let Some(rest) = token.strip_prefix("ahead ") {
      ahead = rest.trim().parse().unwrap_or(0);
    } else if let Some(rest) = token.strip_prefix("behind ") {
      behind = rest.trim().parse().unwrap_or(0);
    }
  }
  (ahead, behind)
}

fn is_stale_iso(iso: &str) -> bool {
  let Some((y, m, d)) = parse_ymd(iso) else {
    return false;
  };
  let commit_day = days_from_civil(y, m, d);
  let Some(today_day) = today_civil_day() else {
    return false;
  };
  today_day.saturating_sub(commit_day) >= 28
}

fn today_civil_day() -> Option<i32> {
  let dur = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .ok()?;
  Some(unix_epoch_civil_day() + (dur.as_secs() / 86_400) as i32)
}

fn unix_epoch_civil_day() -> i32 {
  days_from_civil(1970, 1, 1)
}

fn parse_ymd(iso: &str) -> Option<(i32, u32, u32)> {
  let s = iso.get(..10)?;
  let mut parts = s.split('-');
  let y: i32 = parts.next()?.parse().ok()?;
  let m: u32 = parts.next()?.parse().ok()?;
  let d: u32 = parts.next()?.parse().ok()?;
  if (1..=12).contains(&m) && (1..=31).contains(&d) {
    Some((y, m, d))
  } else {
    None
  }
}

fn days_from_civil(y: i32, m: u32, d: u32) -> i32 {
  let (y, m) = if m <= 2 { (y - 1, m + 12) } else { (y, m) };
  let era = (if y >= 0 { y } else { y - 399 }) / 400;
  let yoe = y - era * 400;
  let doy = (153 * (m as i32 - 3) + 2) / 5 + d as i32 - 1;
  let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
  era * 146097 + doe - 719468
}

fn run_pretty(_args: BranchArgs, mode: OutputMode) -> Result<()> {
  let rows = collect_rows()?;
  let theme = theme::Theme::detect();
  let pretty = mode.is_pretty();
  let ic = icons::detect(!pretty);
  for r in &rows {
    println!("{}", render_row(r, &theme, &ic, pretty));
  }
  Ok(())
}

fn render_track(r: &BranchRow, ic: &icons::Icons) -> String {
  match (r.ahead, r.behind) {
    (0, 0) => String::new(),
    (a, 0) => format!(" {}{a}", ic.ahead),
    (0, b) => format!(" {}{b}", ic.behind),
    (a, b) => format!(" {}{a}{}{b}", ic.ahead, ic.behind),
  }
}

fn render_row(r: &BranchRow, theme: &theme::Theme, ic: &icons::Icons, pretty: bool) -> String {
  let star = if r.current { "*" } else { " " };
  let name = if pretty && r.current {
    theme.branch(&r.name)
  } else {
    r.name.clone()
  };
  let track = render_track(r, ic);
  let track = if pretty {
    match (r.ahead, r.behind) {
      (_, b) if b > 0 => theme.warn(&track),
      (a, _) if a > 0 => theme.accent(&track),
      _ => track,
    }
  } else {
    track
  };
  let stale = if r.stale {
    if pretty {
      theme.warn(" stale")
    } else {
      " stale".to_string()
    }
  } else {
    String::new()
  };
  let when = r
    .last_commit_relative
    .as_deref()
    .map(|s| if pretty { theme.dim(s) } else { s.to_string() })
    .unwrap_or_default();
  format!("{star} {name}{track}{stale}  {when}")
}

fn run_better(_args: BranchArgs, _mode: OutputMode) -> Result<()> {
  let rows = collect_rows()?;
  let current = rows.iter().find(|r| r.current).map(|r| r.name.clone());
  let locals: Vec<&BranchRow> = rows
    .iter()
    .filter(|r| {
      r.upstream
        .as_deref()
        .is_none_or(|u| !u.starts_with("origin/"))
    })
    .collect();
  let mut remotes: BTreeMap<String, Vec<&BranchRow>> = BTreeMap::new();
  for r in &rows {
    if let Some(up) = &r.upstream {
      if let Some((remote, _)) = up.split_once('/') {
        remotes.entry(remote.to_string()).or_default().push(r);
      }
    }
  }
  let stale: Vec<&BranchRow> = rows.iter().filter(|r| r.stale).collect();

  let hints = vec!["use `gb switch <name>` (passthrough) to change branches".to_string()];

  let env = better::envelope_with_hints(
    "branch",
    json!({
        "current": current,
        "locals": locals,
        "remotes": remotes,
        "stale": stale,
    }),
    hints,
    json!({"duration_ms": 0, "bytes": 0}),
  )?;
  println!("{env}");
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::{days_from_civil, is_stale_iso, parse_ymd};

  #[test]
  fn parse_ymd_reads_git_iso_prefix() {
    assert_eq!(parse_ymd("2024-06-15 18:30:45 -0400"), Some((2024, 6, 15)));
  }

  #[test]
  fn is_stale_iso_flags_old_dates() {
    assert!(is_stale_iso("2000-01-01 00:00:00 +0000"));
    assert!(!is_stale_iso("2099-01-01 00:00:00 +0000"));
  }

  #[test]
  fn days_from_civil_round_trips() {
    let day = days_from_civil(2024, 6, 15);
    assert!(day > 0);
  }
}
