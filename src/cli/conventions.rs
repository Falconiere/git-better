use std::io::{IsTerminal, Read};
use std::path::{Component, Path};
use std::time::{Duration, Instant};

use anyhow::{Result, anyhow};
use serde_json::json;

use super::ConventionsArgs;
use crate::conventions::{Profile, cache, detect};
use crate::output::{OutputMode, better, conventions_view};

/// Dispatch `gb conventions`: cached profile as a summary, raw JSON, or an envelope.
pub fn run(args: ConventionsArgs, mode: OutputMode) -> Result<()> {
  if args.json && mode.is_better() {
    return Err(anyhow!(
      "--json and --better are mutually exclusive; pick one JSON shape"
    ));
  }

  let root = detect::repo_root()?;
  let save = match args.save_prose.as_deref() {
    Some(arg) => Some(cache::ProseSave {
      file: repo_relative(&root, arg)?,
      rules: read_prose_from_stdin()?,
    }),
    None => None,
  };

  let started = Instant::now();
  let resolved = cache::resolve(&root, args.with_remote, args.refresh, save)?;
  emit(&resolved, mode, args.json, started.elapsed())
}

fn emit(resolved: &cache::Resolved, mode: OutputMode, json: bool, elapsed: Duration) -> Result<()> {
  if json {
    println!("{}", serde_json::to_string_pretty(&resolved.profile)?);
    return Ok(());
  }

  if mode.is_better() {
    let compact = serde_json::to_string(&resolved.profile)?;
    let env = better::envelope_with_hints(
      "conventions",
      serde_json::to_value(&resolved.profile)?,
      hints(&resolved.profile),
      json!({
          "duration_ms": elapsed.as_millis(),
          "bytes": compact.len(),
          "cache": if resolved.cache_hit { "hit" } else { "miss" },
      }),
    )?;
    println!("{env}");
    return Ok(());
  }

  conventions_view::print_summary(&resolved.profile, mode);
  Ok(())
}

fn hints(profile: &Profile) -> Vec<String> {
  let mut hints = Vec::new();
  for file in &profile.prose_pending {
    hints.push(format!(
      "read {file} once, then persist the rules: printf '%s' \"<rules>\" | gb conventions --save-prose {file}"
    ));
  }
  if profile.commit_format.convention != "conventional-commits" {
    hints.push("no dominant commit convention — match `commit_format.samples`".to_string());
  }
  if !profile.remote_consulted {
    hints.push("run `gb conventions --with-remote` to include recent PR titles".to_string());
  }
  hints
}

fn read_prose_from_stdin() -> Result<String> {
  if std::io::stdin().is_terminal() {
    return Err(anyhow!(
      "--save-prose reads the distilled rules from STDIN; pipe them in"
    ));
  }
  let mut buf = String::new();
  std::io::stdin().read_to_string(&mut buf)?;
  if buf.trim().is_empty() {
    return Err(anyhow!("--save-prose needs non-empty text on STDIN"));
  }
  Ok(buf.trim().to_string())
}

fn repo_relative(root: &Path, arg: &str) -> Result<String> {
  let candidate = Path::new(arg);
  let rel = if candidate.is_absolute() {
    candidate
      .strip_prefix(root)
      .map_err(|_| anyhow!("{arg} is outside the repository"))?
  } else {
    candidate
  };
  if rel
    .components()
    .any(|c| matches!(c, Component::ParentDir | Component::RootDir))
  {
    return Err(anyhow!("{arg} must be a path inside the repository"));
  }
  if !root.join(rel).is_file() {
    return Err(anyhow!("no such file in the repository: {arg}"));
  }
  Ok(rel.display().to_string())
}
