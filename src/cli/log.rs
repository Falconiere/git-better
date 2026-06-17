use anyhow::Result;
use serde_json::json;
use std::collections::BTreeMap;

use super::LogArgs;
use crate::git::{commit, porcelain, proc};
use crate::output::{OutputMode, better, human};

pub fn run(args: LogArgs, mode: OutputMode) -> Result<()> {
  if !args.passthrough.is_empty() {
    let mut all = vec!["log".to_string()];
    all.extend(args.passthrough);
    let out = proc::run_git(&all)?;
    proc::write_to_stdout(&out);
    if !out.ends_with('\n') {
      println!();
    }
    return Ok(());
  }

  if mode.is_better() || args.story {
    return run_better(args, mode);
  }

  run_pretty(&args, mode)
}

fn build_log_format(n: usize) -> Vec<String> {
  vec![
    "log".to_string(),
    format!("-n{n}"),
    "--no-merges".to_string(),
    "--format=COMMIT<<<\nSHA:%H\nAUTHOR:%an\nEMAIL:%ae\nISO:%aI\nREL:%ar\nSUBJECT:%s\nBODY:\n%b"
      .to_string(),
  ]
}

fn run_pretty(_args: &LogArgs, mode: OutputMode) -> Result<()> {
  let raw = proc::run_git(&build_log_format(20))?;
  let records = porcelain::parse_log_format(&raw);
  human::print_log(&records, mode);
  Ok(())
}

fn run_better(args: LogArgs, mode: OutputMode) -> Result<()> {
  let n: usize = if args.story { 50 } else { 20 };
  let raw = proc::run_git(&build_log_format(n))?;
  let records = porcelain::parse_log_format(&raw);

  if args.story {
    let (branch, base_ref, base_display) = detect_branch_and_base()
      .unwrap_or_else(|| ("(current)".into(), "HEAD~1".into(), "(base)".into()));
    let total = records.len() as u64;
    let mut by_type: BTreeMap<String, u64> = BTreeMap::new();
    let mut by_author: BTreeMap<String, u64> = BTreeMap::new();
    let first_subject = records
      .first()
      .map(|r| r.subject.clone())
      .unwrap_or_default();
    let pr = records.iter().find_map(|r| r.pr_number);

    for r in &records {
      if let Some(t) = &r.conventional_type {
        *by_type.entry(t.clone()).or_insert(0) += 1;
      }
      *by_author.entry(r.author_name.clone()).or_insert(0) += 1;
    }

    let (files_changed, net_added, net_removed) = diff_stats_for(&base_ref);

    if mode.is_better() {
      let env = better::envelope_with_hints(
        "log",
        json!({
            "branch": branch,
            "base": base_display,
            "story": format!(
                "{} commit(s) · {} file(s) · +{} -{}",
                total, files_changed, net_added, net_removed
            ),
            "groups": group_by_pr(&records),
            "by_type": by_type,
            "by_author": by_author,
            "first_subject": first_subject,
            "pr": pr,
        }),
        vec![
          "use `gb log` for a quick pretty oneline list".to_string(),
          "use `gb log -n 5` to scope the size".to_string(),
        ],
        json!({"duration_ms": 0, "bytes": 0, "budget": args.budget}),
      )?;
      println!("{env}");
      return Ok(());
    }

    human::print_log_story(
      &branch,
      &base_display,
      total,
      &by_type,
      files_changed,
      net_added,
      net_removed,
      &first_subject,
      pr,
      mode,
    );
    return Ok(());
  }

  let mut by_type: BTreeMap<String, u64> = BTreeMap::new();
  let mut by_author: BTreeMap<String, u64> = BTreeMap::new();
  for r in &records {
    if let Some(t) = &r.conventional_type {
      *by_type.entry(t.clone()).or_insert(0) += 1;
    }
    *by_author.entry(r.author_name.clone()).or_insert(0) += 1;
  }

  let env = better::envelope_with_hints(
    "log",
    json!({
        "commits": records,
        "groups": group_by_pr(&records),
        "by_type": by_type,
        "by_author": by_author,
        "total": records.len(),
    }),
    vec![
      "use `gb log -n N` to scope the size".to_string(),
      "use `gb log --story` for a one-line branch summary".to_string(),
    ],
    json!({"duration_ms": 0, "bytes": 0, "budget": args.budget}),
  )?;
  println!("{env}");
  Ok(())
}

fn detect_branch_and_base() -> Option<(String, String, String)> {
  let branch = proc::run_git(&["branch".into(), "--show-current".into()])
    .ok()
    .map(|s| s.trim().to_string())
    .filter(|s| !s.is_empty())?;
  let upstream = proc::run_git(&["rev-parse".into(), "--abbrev-ref".into(), "@{u}".into()])
    .ok()
    .map(|s| s.trim().to_string())
    .filter(|s| !s.is_empty());
  if let Some(up) = upstream {
    let mb_result = proc::run_git(&["merge-base".into(), "HEAD".into(), up.clone()])
      .ok()
      .map(|s| s.trim().to_string())
      .filter(|s| !s.is_empty());
    let base_ref = mb_result.clone().unwrap_or_else(|| "HEAD~1".to_string());
    let base_display = match mb_result {
      Some(mb) => {
        let short = &mb[..7.min(mb.len())];
        format!("{up} ({short})")
      },
      None => "HEAD~1".to_string(),
    };
    return Some((branch, base_ref, base_display));
  }
  for candidate in ["origin/main", "origin/master", "main", "master"] {
    if proc::run_git(&[
      "rev-parse".into(),
      "--verify".into(),
      "--quiet".into(),
      candidate.into(),
    ])
    .is_ok()
    {
      if let Ok(mb) = proc::run_git(&["merge-base".into(), "HEAD".into(), candidate.into()]) {
        let mb = mb.trim().to_string();
        if !mb.is_empty() {
          let short = &mb[..7.min(mb.len())];
          return Some((
            branch,
            candidate.to_string(),
            format!("{candidate} ({short})"),
          ));
        }
      }
    }
  }
  Some((branch, "HEAD~1".to_string(), "HEAD~1".to_string()))
}

fn diff_stats_for(base: &str) -> (u64, u64, u64) {
  let numstat = proc::run_git(&[
    "diff".to_string(),
    "--numstat".to_string(),
    base.to_string(),
    "HEAD".to_string(),
  ])
  .or_else(|_| {
    proc::run_git(&[
      "diff".to_string(),
      "--numstat".to_string(),
      "HEAD~1".to_string(),
      "HEAD".to_string(),
    ])
  })
  .unwrap_or_default();
  let files = crate::git::diff::parse_numstat(&numstat);
  let s = crate::git::diff::summarize(&files);
  (s.files_changed, s.added, s.removed)
}

type PrGroup = (
  Option<u64>,
  String,
  Option<String>,
  Option<String>,
  Vec<String>,
);

fn group_by_pr(records: &[commit::CommitRecord]) -> Vec<serde_json::Value> {
  let mut groups: Vec<PrGroup> = Vec::new();
  for r in records {
    let pr = r.pr_number;
    let short = r.short_sha.clone();
    if let Some(last) = groups.last_mut() {
      if last.0 == pr && pr.is_some() {
        last.4.push(short);
        continue;
      }
    }
    groups.push((
      pr,
      r.subject.clone(),
      r.conventional_type.clone(),
      r.conventional_scope.clone(),
      vec![short],
    ));
  }
  groups
    .into_iter()
    .map(|(pr, title, ct, cs, commits)| {
      json!({
          "pr": pr,
          "title": title,
          "conventional_type": ct,
          "conventional_scope": cs,
          "commits": commits,
      })
    })
    .collect()
}
