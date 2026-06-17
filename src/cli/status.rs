use anyhow::Result;
use serde_json::json;

use super::StatusArgs;
use crate::git::proc;
use crate::output::{OutputMode, better};

pub fn run(args: StatusArgs, mode: OutputMode) -> Result<()> {
  if !args.passthrough.is_empty() {
    let mut all: Vec<String> = vec!["status".to_string(), "-sb".to_string()];
    all.extend(args.passthrough);
    let out = proc::run_git(&all)?;
    proc::write_to_stdout(&out);
    if !out.ends_with('\n') {
      println!();
    }
    return Ok(());
  }

  if mode.is_better() {
    let raw = proc::run_git(&["status".to_string(), "-sb".to_string()])?;
    let env = better::envelope(
      "status",
      json!({
          "raw": raw,
          "note": "M0 stub: structured fields land in M1",
      }),
    )?;
    println!("{env}");
    return Ok(());
  }

  let out = proc::run_git(&["status".to_string(), "-sb".to_string()])?;
  proc::write_to_stdout(&out);
  if !out.ends_with('\n') {
    println!();
  }
  Ok(())
}
