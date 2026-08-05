use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use serde_json::json;

use super::{SkillArgs, SkillCmd, SkillInstallArgs};
use crate::conventions::detect;
use crate::output::{OutputMode, better};

const SKILL_TEXT: &str = include_str!("../../SKILL.md");
const BEGIN: &str = "<!-- git-better:begin -->";
const END: &str = "<!-- git-better:end -->";
const CURSOR_FRONTMATTER: &str =
  "---\ndescription: Token-lean git protocol (gb)\nalwaysApply: true\n---\n\n";

/// How a target file carries the protocol text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Style {
  Whole,
  CursorRule,
  Fenced,
}

/// Whether a target lives in the user's config or in the repository.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scope {
  User,
  Project,
}

struct Target {
  name: &'static str,
  style: Style,
  scope: Scope,
}

const TARGETS: &[Target] = &[
  Target {
    name: "claude-user",
    style: Style::Whole,
    scope: Scope::User,
  },
  Target {
    name: "claude-project",
    style: Style::Whole,
    scope: Scope::Project,
  },
  Target {
    name: "cursor",
    style: Style::CursorRule,
    scope: Scope::Project,
  },
  Target {
    name: "windsurf",
    style: Style::Whole,
    scope: Scope::Project,
  },
  Target {
    name: "copilot",
    style: Style::Fenced,
    scope: Scope::Project,
  },
  Target {
    name: "codex",
    style: Style::Fenced,
    scope: Scope::User,
  },
  Target {
    name: "agents-md",
    style: Style::Fenced,
    scope: Scope::Project,
  },
];

#[derive(Debug, Clone, PartialEq, Eq)]
enum Outcome {
  Wrote,
  Unchanged,
  Skipped(&'static str),
}

struct Entry {
  name: &'static str,
  path: String,
  outcome: Outcome,
}

/// Dispatch `gb skill`: print the protocol, list target paths, or install it.
pub fn run(args: SkillArgs, mode: OutputMode) -> Result<()> {
  match args.cmd {
    SkillCmd::Print => {
      print!("{SKILL_TEXT}");
      Ok(())
    },
    SkillCmd::Path => list_paths(),
    SkillCmd::Install(install) => run_install(install, mode),
  }
}

fn list_paths() -> Result<()> {
  let root = detect::repo_root().ok();
  for target in TARGETS {
    match resolve(target, root.as_deref()) {
      Some((path, _)) => println!("{}\t{}", target.name, path.display()),
      None => println!("{}\t-", target.name),
    }
  }
  Ok(())
}

fn run_install(args: SkillInstallArgs, mode: OutputMode) -> Result<()> {
  let selected = select(&args)?;
  let root = detect::repo_root().ok();
  let explicit = args.all || !args.target.is_empty();

  let mut entries = Vec::new();
  for target in selected {
    entries.push(install_one(target, root.as_deref(), &args, explicit)?);
  }

  if mode.is_better() {
    report_json(&entries)
  } else {
    report_human(&entries);
    Ok(())
  }
}

fn select(args: &SkillInstallArgs) -> Result<Vec<&'static Target>> {
  if args.target.is_empty() {
    return Ok(TARGETS.iter().collect());
  }
  let mut chosen = Vec::new();
  for name in &args.target {
    match TARGETS.iter().find(|t| t.name == name) {
      Some(target) => chosen.push(target),
      None => {
        let valid: Vec<&str> = TARGETS.iter().map(|t| t.name).collect();
        return Err(anyhow!(
          "unknown target '{name}'; valid targets: {}",
          valid.join(", ")
        ));
      },
    }
  }
  Ok(chosen)
}

fn install_one(
  target: &'static Target,
  root: Option<&Path>,
  args: &SkillInstallArgs,
  explicit: bool,
) -> Result<Entry> {
  let Some((path, marker)) = resolve(target, root) else {
    let reason = if target.scope == Scope::Project {
      "no repository"
    } else {
      "no home directory"
    };
    return Ok(Entry {
      name: target.name,
      path: "-".to_string(),
      outcome: Outcome::Skipped(reason),
    });
  };

  if !explicit && !marker.exists() {
    return Ok(Entry {
      name: target.name,
      path: path.display().to_string(),
      outcome: Outcome::Skipped("not detected"),
    });
  }

  let outcome = match target.style {
    Style::Whole => apply_whole(&path, SKILL_TEXT, args),
    Style::CursorRule => apply_whole(&path, &format!("{CURSOR_FRONTMATTER}{}", body()), args),
    Style::Fenced => apply_fenced(&path, args),
  }?;

  Ok(Entry {
    name: target.name,
    path: path.display().to_string(),
    outcome,
  })
}

fn resolve(target: &Target, root: Option<&Path>) -> Option<(PathBuf, PathBuf)> {
  match target.name {
    "claude-user" => {
      let base = config_home("CLAUDE_CONFIG_DIR", ".claude")?;
      Some((base.join("skills/git-better/SKILL.md"), base))
    },
    "claude-project" => {
      let root = root?;
      Some((
        root.join(".claude/skills/git-better/SKILL.md"),
        root.join(".claude"),
      ))
    },
    "cursor" => {
      let root = root?;
      Some((
        root.join(".cursor/rules/git-better.mdc"),
        root.join(".cursor"),
      ))
    },
    "windsurf" => {
      let root = root?;
      Some((
        root.join(".windsurf/rules/git-better.md"),
        root.join(".windsurf"),
      ))
    },
    "copilot" => {
      let file = root?.join(".github/copilot-instructions.md");
      Some((file.clone(), file))
    },
    "codex" => {
      let file = config_home("CODEX_HOME", ".codex")?.join("AGENTS.md");
      Some((file.clone(), file))
    },
    "agents-md" => {
      let file = root?.join("AGENTS.md");
      Some((file.clone(), file))
    },
    _ => None,
  }
}

fn config_home(env_key: &str, fallback: &str) -> Option<PathBuf> {
  if let Some(dir) = std::env::var_os(env_key).filter(|v| !v.is_empty()) {
    return Some(PathBuf::from(dir));
  }
  std::env::var_os("HOME")
    .filter(|v| !v.is_empty())
    .map(|home| PathBuf::from(home).join(fallback))
}

fn body() -> &'static str {
  match SKILL_TEXT.strip_prefix("---\n") {
    Some(rest) => match rest.split_once("\n---\n") {
      Some((_, after)) => after.trim_start_matches('\n'),
      None => SKILL_TEXT,
    },
    None => SKILL_TEXT,
  }
}

fn apply_whole(path: &Path, content: &str, args: &SkillInstallArgs) -> Result<Outcome> {
  match std::fs::read_to_string(path) {
    Ok(existing) if existing == content && !args.force => Ok(Outcome::Unchanged),
    Ok(existing) if !args.force && !is_ours(&existing) => {
      Ok(Outcome::Skipped("foreign content, use --force"))
    },
    Ok(_) => write(path, content, args.dry_run),
    Err(err) if err.kind() == ErrorKind::NotFound => write(path, content, args.dry_run),
    Err(_) => Ok(Outcome::Skipped("unreadable")),
  }
}

fn apply_fenced(path: &Path, args: &SkillInstallArgs) -> Result<Outcome> {
  let existing = match std::fs::read_to_string(path) {
    Ok(text) => text,
    Err(err) if err.kind() == ErrorKind::NotFound => String::new(),
    Err(_) => return Ok(Outcome::Skipped("unreadable")),
  };

  let updated = splice(&existing, &block());
  if updated == existing && !args.force {
    return Ok(Outcome::Unchanged);
  }
  write(path, &updated, args.dry_run)
}

fn splice(existing: &str, block: &str) -> String {
  match (existing.find(BEGIN), existing.find(END)) {
    (Some(start), Some(end)) if end > start => {
      let tail = &existing[end + END.len()..];
      format!(
        "{}{block}{}",
        &existing[..start],
        tail.trim_start_matches('\n')
      )
    },
    _ if existing.trim().is_empty() => block.to_string(),
    _ => format!("{}\n\n{block}", existing.trim_end()),
  }
}

fn block() -> String {
  format!("{BEGIN}\n{}\n{END}\n", body().trim_end())
}

fn is_ours(existing: &str) -> bool {
  existing.contains("name: git-better") || existing.contains(BEGIN)
}

fn write(path: &Path, content: &str, dry_run: bool) -> Result<Outcome> {
  if dry_run {
    return Ok(Outcome::Wrote);
  }
  if let Some(parent) = path.parent() {
    std::fs::create_dir_all(parent)?;
  }
  std::fs::write(path, content)?;
  Ok(Outcome::Wrote)
}

fn report_human(entries: &[Entry]) {
  for entry in entries {
    let label = match &entry.outcome {
      Outcome::Wrote => "wrote".to_string(),
      Outcome::Unchanged => "unchanged".to_string(),
      Outcome::Skipped(reason) => format!("skipped ({reason})"),
    };
    println!("{:<28} {:<12} {}", entry.name, label, entry.path);
  }
}

fn report_json(entries: &[Entry]) -> Result<()> {
  let installed: Vec<_> = entries
    .iter()
    .filter(|e| e.outcome == Outcome::Wrote)
    .map(|e| json!({"target": e.name, "path": e.path}))
    .collect();
  let unchanged: Vec<_> = entries
    .iter()
    .filter(|e| e.outcome == Outcome::Unchanged)
    .map(|e| json!({"target": e.name, "path": e.path}))
    .collect();
  let skipped: Vec<_> = entries
    .iter()
    .filter_map(|e| match &e.outcome {
      Outcome::Skipped(reason) => Some(json!({"target": e.name, "path": e.path, "reason": reason})),
      _ => None,
    })
    .collect();

  let env = better::envelope_with_hints(
    "skill install",
    json!({"installed": installed, "unchanged": unchanged, "skipped": skipped}),
    vec!["pass `--all` to install every target, or `--target <name>` for one".to_string()],
    json!({"targets": TARGETS.len()}),
  )?;
  println!("{env}");
  Ok(())
}
