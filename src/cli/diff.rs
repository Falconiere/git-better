use anyhow::Result;
use serde_json::json;

use super::DiffArgs;
use crate::git::{diff, proc};
use crate::output::{OutputMode, better, human};

pub fn run(args: DiffArgs, mode: OutputMode) -> Result<()> {
  if !args.passthrough.is_empty() {
    let mut all = vec!["diff".to_string()];
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
    return run_full_unified(mode.is_pretty());
  }

  run_lean_stat(&args, mode)
}

fn run_lean_stat(_args: &DiffArgs, mode: OutputMode) -> Result<()> {
  let numstat_out = proc::run_git_with_excludes(&["diff".to_string(), "--numstat".to_string()])?;
  let files = diff::parse_numstat(&numstat_out);
  let summary = diff::summarize(&files);
  human::print_diff_stat(&files, &summary, mode);
  Ok(())
}

fn run_full_unified(pretty: bool) -> Result<()> {
  let raw = proc::run_git(&["diff".to_string()])?;
  let files = diff::parse_unified_diff(&raw);
  let mode = if pretty {
    OutputMode::Human { pretty: true }
  } else {
    OutputMode::Human { pretty: false }
  };
  human::print_diff_full(&files, mode);
  Ok(())
}

fn run_better(args: DiffArgs, _mode: OutputMode) -> Result<()> {
  let numstat_out = if args.full {
    proc::run_git(&["diff".to_string(), "--numstat".to_string()])?
  } else {
    proc::run_git_with_excludes(&["diff".to_string(), "--numstat".to_string()])?
  };
  let files = diff::parse_numstat(&numstat_out);
  let summary = diff::summarize(&files);

  let full = if args.full {
    proc::run_git(&["diff".to_string()])?
  } else {
    proc::run_git_with_excludes(&["diff".to_string()])?
  };

  let (text, truncated_paths, truncated) = match args.budget {
    Some(b) => diff::truncate_unified_diff(&full, b),
    None => (full, Vec::new(), false),
  };

  let mut hints: Vec<String> = Vec::new();
  if truncated {
    hints.push(format!(
      "run `gb diff --full <path>` to see the rest ({} file(s) truncated)",
      truncated_paths.len()
    ));
  } else if !args.full {
    hints.push("lockfiles excluded by default; pass `--full` to include them".to_string());
  }
  if !args.full {
    hints.push(format!(
      "excluded patterns: {}",
      proc::LOCKFILE_EXCLUDES.join(", ")
    ));
  }

  let env = better::envelope_with_hints(
    "diff",
    json!({
        "files": files,
        "summary": summary,
        "lockfiles_excluded": proc::LOCKFILE_EXCLUDES,
        "truncated": truncated,
        "truncated_files": truncated_paths,
        "full": args.full,
        "patch": text,
    }),
    hints,
    json!({
        "duration_ms": 0,
        "bytes": text.len(),
        "budget": args.budget,
    }),
  )?;
  println!("{env}");
  Ok(())
}

impl DiffArgs {
  pub fn dispatch_full(pretty: bool) -> Result<()> {
    run_full_unified(pretty)
  }
}
