use anyhow::Result;
use serde_json::json;
use std::collections::BTreeMap;

use super::LogArgs;
use crate::git::{commit, porcelain, proc};
use crate::output::{OutputMode, better, human};

pub fn run(args: LogArgs, mode: OutputMode) -> Result<()> {
  if args.budget.is_some() && !mode.is_better() {
    anyhow::bail!("--budget only applies with --better");
  }
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

fn collect_by_type_author(
  records: &[commit::CommitRecord],
) -> (BTreeMap<String, u64>, BTreeMap<String, u64>) {
  let mut by_type: BTreeMap<String, u64> = BTreeMap::new();
  let mut by_author: BTreeMap<String, u64> = BTreeMap::new();
  for r in records {
    if let Some(t) = &r.conventional_type {
      *by_type.entry(t.clone()).or_insert(0) += 1;
    }
    *by_author.entry(r.author_name.clone()).or_insert(0) += 1;
  }
  (by_type, by_author)
}

fn run_better(args: LogArgs, mode: OutputMode) -> Result<()> {
  let n: usize = if args.story { 50 } else { 20 };
  let raw = proc::run_git(&build_log_format(n))?;
  let records = porcelain::parse_log_format(&raw);

  if args.story {
    return run_story(&args, &records, mode);
  }

  print_plain_log_envelope(&args, records)
}

fn apply_log_budget(
  records: Vec<commit::CommitRecord>,
  budget: Option<usize>,
) -> (Vec<commit::CommitRecord>, bool) {
  let Some(budget) = budget else {
    return (records, false);
  };
  let char_budget = budget.saturating_mul(4);
  if records.is_empty() {
    return (records, false);
  }
  let full_len = serde_json::to_string(&records)
    .map(|s| s.len())
    .unwrap_or(usize::MAX);
  if full_len <= char_budget {
    return (records, false);
  }
  let mut lo = 0usize;
  let mut hi = records.len();
  while lo < hi {
    let mid = lo + (hi - lo).div_ceil(2);
    let len = serde_json::to_string(&records[..mid])
      .map(|s| s.len())
      .unwrap_or(usize::MAX);
    if len <= char_budget {
      lo = mid;
    } else {
      hi = mid - 1;
    }
  }
  let truncated = lo < records.len();
  (records.into_iter().take(lo).collect(), truncated)
}

fn commit_count_since(base: &str) -> u64 {
  if !proc::is_safe_git_ref(base) {
    return 0;
  }
  proc::run_git(&["rev-list".into(), "--count".into(), format!("{base}..HEAD")])
    .ok()
    .and_then(|s| s.trim().parse().ok())
    .unwrap_or(0)
}

fn print_story_envelope(
  args: &LogArgs,
  records: &[commit::CommitRecord],
  story: &human::LogStory<'_>,
  by_author: &BTreeMap<String, u64>,
) -> Result<()> {
  let env = better::envelope_with_hints(
    "log",
    json!({
        "branch": story.branch,
        "base": story.base,
        "story": format!(
            "{} commit(s) · {} file(s) · +{} -{}",
            story.total, story.files_changed, story.net_added, story.net_removed
        ),
        "groups": group_by_pr(records),
        "by_type": story.by_type,
        "by_author": by_author,
        "first_subject": story.first_subject,
        "pr": story.pr,
    }),
    vec![
      "use `gb log` for a quick pretty oneline list".to_string(),
      "use `gb log -n 5` to scope the size".to_string(),
    ],
    json!({"duration_ms": 0, "bytes": 0, "budget": args.budget}),
  )?;
  println!("{env}");
  Ok(())
}

fn run_story(args: &LogArgs, records: &[commit::CommitRecord], mode: OutputMode) -> Result<()> {
  let (branch, base_ref, base_display) = detect_branch_and_base()
    .unwrap_or_else(|| ("(current)".into(), "HEAD~1".into(), "(base)".into()));
  let fetched = records.len() as u64;
  let total = commit_count_since(&base_ref);
  let total = if total > 0 { total } else { fetched };
  let (by_type, by_author) = collect_by_type_author(records);
  let first_subject = records
    .first()
    .map(|r| r.subject.clone())
    .unwrap_or_default();
  let pr = records.iter().find_map(|r| r.pr_number);
  let (files_changed, net_added, net_removed) = diff_stats_for(&base_ref);

  let story = human::LogStory {
    branch: &branch,
    base: &base_display,
    total,
    by_type: &by_type,
    files_changed,
    net_added,
    net_removed,
    first_subject: &first_subject,
    pr,
  };

  if mode.is_better() {
    return print_story_envelope(args, records, &story, &by_author);
  }

  human::print_log_story(&story, mode);
  Ok(())
}

fn print_plain_log_envelope(args: &LogArgs, records: Vec<commit::CommitRecord>) -> Result<()> {
  let (records, truncated) = apply_log_budget(records, args.budget);
  let (by_type, by_author) = collect_by_type_author(&records);
  let mut hints = vec![
    "use `gb log -n N` to scope the size".to_string(),
    "use `gb log --story` for a one-line branch summary".to_string(),
  ];
  if truncated {
    hints.insert(
      0,
      "log output truncated to fit --budget; raise budget or use `gb log -n N`".to_string(),
    );
  }
  let env = better::envelope_with_hints(
    "log",
    json!({
        "commits": records,
        "groups": group_by_pr(&records),
        "by_type": by_type,
        "by_author": by_author,
        "total": records.len(),
        "truncated": truncated,
    }),
    hints,
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
        let short: String = mb.chars().take(7).collect();
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
          let short: String = mb.chars().take(7).collect();
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
  let numstat = if proc::is_safe_git_ref(base) {
    proc::run_git(&[
      "diff".to_string(),
      "--numstat".to_string(),
      base.to_string(),
      "HEAD".to_string(),
    ])
  } else {
    Err(crate::error::GbError::GitFailed {
      code: 1,
      stderr: "invalid git ref".into(),
    })
  }
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
