use anyhow::Result;
use serde_json::json;
use std::collections::BTreeMap;

use super::BranchArgs;
use crate::git::proc;
use crate::output::{OutputMode, better, theme};

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
    if line.is_empty() {
      continue;
    }
    let parts: Vec<&str> = line.split('\0').collect();
    if parts.len() < 5 {
      continue;
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
    let is_current = Some(&name) == current.as_ref();
    let stale = last_rel.as_deref().map(is_stale).unwrap_or(false);
    rows.push(BranchRow {
      name,
      current: is_current,
      upstream,
      ahead,
      behind,
      last_commit_relative: last_rel,
      last_commit_iso: last_iso,
      stale,
    });
  }
  Ok(rows)
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

fn is_stale(relative: &str) -> bool {
  if let Some(num) = relative.split_whitespace().next() {
    if let Ok(n) = num.parse::<u64>() {
      if relative.contains("month") || relative.contains("year") {
        return true;
      }
      if relative.contains("week") && n >= 4 {
        return true;
      }
    }
  }
  false
}

fn run_pretty(_args: BranchArgs, mode: OutputMode) -> Result<()> {
  let rows = collect_rows()?;
  let theme = theme::Theme::detect();
  let pretty = mode.is_pretty();

  for r in &rows {
    let star = if r.current { "*" } else { " " };
    let name = if pretty {
      if r.current {
        theme.branch(&r.name)
      } else {
        r.name.clone()
      }
    } else {
      r.name.clone()
    };
    let track = match (r.ahead, r.behind) {
      (0, 0) => String::new(),
      (a, 0) => format!(" ⇡{a}"),
      (0, b) => format!(" ⇣{b}"),
      (a, b) => format!(" ⇡{a}⇣{b}"),
    };
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
    println!("{star} {name}{track}{stale}  {when}");
  }
  Ok(())
}

fn run_better(_args: BranchArgs, _mode: OutputMode) -> Result<()> {
  let rows = collect_rows()?;
  let current = rows.iter().find(|r| r.current).map(|r| r.name.clone());
  let locals: Vec<&BranchRow> = rows
    .iter()
    .filter(|r| r.upstream.is_none() || !r.upstream.as_ref().unwrap().starts_with("origin/"))
    .collect();
  let remotes = BTreeMap::<String, Vec<&BranchRow>>::new();
  let stale: Vec<&BranchRow> = rows.iter().filter(|r| r.stale).collect();

  let hints = vec![
    "use `gb branch --better` to see upstream + ahead/behind".to_string(),
    "use `gb switch <name>` (passthrough) to change branches".to_string(),
  ];

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
