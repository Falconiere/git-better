use anyhow::Result;
use serde_json::json;

use super::ReflogArgs;
use crate::git::{proc, reflog as reflog_mod};
use crate::output::{better, human, OutputMode};

pub fn run(args: ReflogArgs, mode: OutputMode) -> Result<()> {
    if !args.passthrough.is_empty() {
        let mut all = vec!["reflog".to_string()];
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

fn run_pretty(args: ReflogArgs, mode: OutputMode) -> Result<()> {
    let n = args.n.unwrap_or(50);
    let raw = proc::run_git(&[
        "reflog".to_string(),
        format!("-n{n}"),
        "--date=iso-strict".to_string(),
        "--abbrev=7".to_string(),
    ])?;
    let entries = reflog_mod::parse_reflog(&raw);
    human::print_reflog(&entries, mode);
    Ok(())
}

fn run_better(_args: ReflogArgs, _mode: OutputMode) -> Result<()> {
    let raw = proc::run_git(&[
        "reflog".to_string(),
        "-n50".to_string(),
        "--date=iso-strict".to_string(),
        "--abbrev=12".to_string(),
    ])?;
    let entries = reflog_mod::parse_reflog(&raw);
    let env = better::envelope_with_hints(
        "reflog",
        json!({
            "entries": entries,
        }),
        vec!["use `gb reflog -n 200` to expand the window".to_string()],
        json!({"duration_ms": 0, "bytes": 0}),
    )?;
    println!("{env}");
    Ok(())
}
