use anyhow::Result;
use serde_json::json;

use super::ShowArgs;
use crate::git::{commit, diff, porcelain, proc};
use crate::output::{OutputMode, better, human};

pub fn run(args: ShowArgs, mode: OutputMode) -> Result<()> {
  if !args.passthrough.is_empty() {
    let mut all = vec!["show".to_string()];
    all.extend(args.passthrough);
    let out = proc::run_git(&all)?;
    proc::write_to_stdout(&out);
    if !out.ends_with('\n') {
      println!();
    }
    return Ok(());
  }

  if mode.is_better() || args.budget.is_some() {
    return run_better(args, mode);
  }

  if args.full {
    return run_full_unified(args, mode);
  }

  run_lean_stat(args, mode)
}

fn run_lean_stat(args: ShowArgs, mode: OutputMode) -> Result<()> {
  let target = args.target.clone().unwrap_or_else(|| "HEAD".to_string());
  let numstat_out = proc::run_git(&[
    "show".to_string(),
    "--numstat".to_string(),
    "--format=".to_string(),
    target.clone(),
  ])?;
  let files = diff::parse_numstat(&numstat_out);
  let summary = diff::summarize(&files);

  let log = proc::run_git(&[
    "show".to_string(),
    "--no-patch".to_string(),
    "--format=COMMIT<<<\nSHA:%H\nAUTHOR:%an\nEMAIL:%ae\nISO:%aI\nREL:%ar\nSUBJECT:%s\nBODY:\n%b"
      .to_string(),
    target.clone(),
  ])?;
  let mut records = porcelain::parse_log_format(&log);
  if records.is_empty() {
    anyhow::bail!("no commit found for target `{target}`");
  }
  let record = records.remove(0);

  human::print_show(&record, &files, &summary, mode);
  Ok(())
}

fn run_full_unified(args: ShowArgs, mode: OutputMode) -> Result<()> {
  let target = args.target.unwrap_or_else(|| "HEAD".to_string());
  let raw = proc::run_git(&["show".to_string(), target.clone()])?;
  let files = diff::parse_unified_diff(&raw);
  let pretty = mode.is_pretty();
  human::print_diff_full(&files, mode);
  let _ = (pretty, target);
  Ok(())
}

fn run_better(args: ShowArgs, _mode: OutputMode) -> Result<()> {
  let target = args.target.clone().unwrap_or_else(|| "HEAD".to_string());
  let numstat_out = proc::run_git(&[
    "show".to_string(),
    "--numstat".to_string(),
    "--format=".to_string(),
    target.clone(),
  ])?;
  let files = diff::parse_numstat(&numstat_out);
  let summary = diff::summarize(&files);

  let log = proc::run_git(&[
    "show".to_string(),
    "--no-patch".to_string(),
    "--format=COMMIT<<<\nSHA:%H\nAUTHOR:%an\nEMAIL:%ae\nISO:%aI\nREL:%ar\nSUBJECT:%s\nBODY:\n%b"
      .to_string(),
    target.clone(),
  ])?;
  let mut records = porcelain::parse_log_format(&log);
  if records.is_empty() {
    anyhow::bail!("no commit found for target `{target}`");
  }
  let record = records.remove(0);

  let full = proc::run_git(&["show".to_string(), target.clone()])?;
  let (text, truncated_paths, truncated) = match args.budget {
    Some(b) => diff::truncate_unified_diff(&full, b),
    None => (full, Vec::new(), false),
  };

  let mut hints: Vec<String> = Vec::new();
  if truncated {
    hints.push(format!(
      "{} file(s) truncated; use `gb show {target} --full` to see each one",
      truncated_paths.len()
    ));
  }

  let env = better::envelope_with_hints(
    "show",
    json!({
        "commit": record,
        "files": files,
        "summary": summary,
        "truncated": truncated,
        "truncated_files": truncated_paths,
        "patch": text,
    }),
    hints,
    json!({"duration_ms": 0, "bytes": text.len(), "budget": args.budget}),
  )?;
  println!("{env}");
  Ok(())
}

impl commit::CommitRecord {
  #[allow(dead_code)]
  pub fn short_subject(&self) -> &str {
    &self.subject
  }
}
