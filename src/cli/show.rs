use anyhow::Result;
use serde_json::json;

use super::ShowArgs;
use crate::git::{commit, diff, porcelain, proc};
use crate::output::{OutputMode, better, human};

/// Dispatch `gb show`: structured `--better`, full unified render, or lean stat.
pub fn run(args: ShowArgs, mode: OutputMode) -> Result<()> {
  if args.budget.is_some() && !mode.is_better() {
    anyhow::bail!("--budget only applies with --better");
  }
  if mode.is_better() {
    return run_better(args, mode);
  }

  if args.full {
    return run_full_unified(args, mode);
  }

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

  run_lean_stat(args, mode)
}

fn fetch_show(
  target: &str,
) -> Result<(commit::CommitRecord, Vec<diff::FileStat>, diff::DiffSummary)> {
  let numstat_out = proc::run_git(&[
    "show".to_string(),
    "--numstat".to_string(),
    "--format=".to_string(),
    target.to_string(),
  ])?;
  let files = diff::parse_numstat(&numstat_out);
  let summary = diff::summarize(&files);

  let log = proc::run_git(&[
    "show".to_string(),
    "--no-patch".to_string(),
    "--format=COMMIT<<<\nSHA:%H\nAUTHOR:%an\nEMAIL:%ae\nISO:%aI\nREL:%ar\nSUBJECT:%s\nBODY:\n%b"
      .to_string(),
    target.to_string(),
  ])?;
  let mut records = porcelain::parse_log_format(&log);
  if records.is_empty() {
    anyhow::bail!("no commit found for target `{target}`");
  }
  Ok((records.remove(0), files, summary))
}

fn run_lean_stat(args: ShowArgs, mode: OutputMode) -> Result<()> {
  let target = args.target.unwrap_or_else(|| "HEAD".to_string());
  let (record, files, summary) = fetch_show(&target)?;
  human::print_show(&record, &files, &summary, mode);
  Ok(())
}

fn run_full_unified(args: ShowArgs, mode: OutputMode) -> Result<()> {
  let target = args.target.unwrap_or_else(|| "HEAD".to_string());
  let mut all = vec!["show".to_string(), target];
  all.extend(args.passthrough);
  let raw = proc::run_git(&all)?;
  let files = diff::parse_unified_diff(&raw);
  human::print_diff_full(&files, mode);
  Ok(())
}

fn run_better(args: ShowArgs, _mode: OutputMode) -> Result<()> {
  let target = args.target.clone().unwrap_or_else(|| "HEAD".to_string());
  let (record, files, summary) = fetch_show(&target)?;

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
