use clap::{Parser, Subcommand};

use crate::output::OutputMode;

/// Parsed command line: global output flags plus the chosen subcommand.
#[derive(Debug, Parser)]
#[command(
  name = "gb",
  version,
  about = "Token-lean git companion for humans and LLM agents (macOS only)",
  long_about = "Drop-in `git` wrapper. Prettified read commands for humans. \
                  Append `--better` for token-budgeted JSON for LLM agents. \
                  Unknown subcommands forward verbatim to `git`."
)]
pub struct Cli {
  /// Strip all colors, icons, and box-drawing characters
  #[arg(long, global = true)]
  pub plain: bool,

  /// Output as JSON for LLM agents (envelope: {ok, command, data, hints, meta})
  #[arg(long, global = true)]
  pub better: bool,

  #[command(subcommand)]
  pub cmd: Cmd,
}

/// Subcommands `gb` handles itself; everything else forwards to `git`.
#[derive(Debug, Subcommand)]
pub enum Cmd {
  /// Show working-tree status (lean: `git status -sb`, no color)
  Status(StatusArgs),

  /// Show changes (default: stat + lockfile excludes, pretty)
  Diff(DiffArgs),

  /// Show commit logs (default: 20 oneline, pretty with type tags)
  Log(LogArgs),

  /// Show various types of objects (default: `git show --stat HEAD`, pretty)
  Show(ShowArgs),

  /// List, create, or delete branches (pretty, with ahead/behind + stale)
  Branch(BranchArgs),

  /// Manage the reflog (default: last 50, pretty)
  Reflog(ReflogArgs),

  /// Report this repository's commit, branch, PR, and release conventions (cached)
  Conventions(ConventionsArgs),

  /// Print or install the `gb` protocol document for coding agents
  Skill(SkillArgs),

  /// Pass through to `git` (anything not handled above)
  #[command(external_subcommand)]
  Passthrough(Vec<String>),
}

/// Arguments for `gb status`.
#[derive(Debug, clap::Args)]
pub struct StatusArgs {
  /// Pass remaining args to `git status`
  #[arg(allow_hyphen_values = true, trailing_var_arg = true)]
  pub passthrough: Vec<String>,
}

/// Arguments for `gb diff`.
#[derive(Debug, clap::Args)]
pub struct DiffArgs {
  /// Show full diff with syntax-highlighted hunks instead of the lean stat
  #[arg(long)]
  pub full: bool,

  /// Truncate the patch payload to ~N tokens (chars/4). Only with --better.
  #[arg(long)]
  pub budget: Option<usize>,

  /// Pass remaining args to `git diff`
  #[arg(allow_hyphen_values = true, trailing_var_arg = true)]
  pub passthrough: Vec<String>,
}

/// Arguments for `gb log`.
#[derive(Debug, clap::Args)]
pub struct LogArgs {
  /// One-line "branch story" (commits, type histogram, files, PR)
  #[arg(long)]
  pub story: bool,

  /// Truncate the commit payload to ~N tokens (chars/4). Only with --better.
  #[arg(long)]
  pub budget: Option<usize>,

  /// Pass remaining args to `git log`
  #[arg(allow_hyphen_values = true, trailing_var_arg = true)]
  pub passthrough: Vec<String>,
}

/// Arguments for `gb show`.
#[derive(Debug, clap::Args)]
pub struct ShowArgs {
  /// Show full diff with syntax-highlighted hunks instead of the lean stat
  #[arg(long)]
  pub full: bool,

  /// Truncate the patch payload to ~N tokens (chars/4). Only with --better.
  #[arg(long)]
  pub budget: Option<usize>,

  /// Specific commit/ref to show. Default: HEAD.
  pub target: Option<String>,

  /// Pass remaining args to `git show`
  #[arg(allow_hyphen_values = true, trailing_var_arg = true)]
  pub passthrough: Vec<String>,
}

/// Arguments for `gb branch`.
#[derive(Debug, clap::Args)]
pub struct BranchArgs {
  /// Pass remaining args to `git branch`
  #[arg(allow_hyphen_values = true, trailing_var_arg = true)]
  pub passthrough: Vec<String>,
}

/// Arguments for `gb reflog`.
#[derive(Debug, clap::Args)]
pub struct ReflogArgs {
  /// Number of entries to show (default 50)
  #[arg(short = 'n', long)]
  pub n: Option<usize>,

  /// Pass remaining args to `git reflog`
  #[arg(allow_hyphen_values = true, trailing_var_arg = true)]
  pub passthrough: Vec<String>,
}

/// Arguments for `gb conventions`.
#[derive(Debug, clap::Args)]
pub struct ConventionsArgs {
  /// Print the raw profile JSON without the `--better` envelope
  #[arg(long)]
  pub json: bool,

  /// Recompute the profile, ignoring cache freshness
  #[arg(long)]
  pub refresh: bool,

  /// Allow one bounded `gh pr list` lookup for recent pull-request titles
  #[arg(long)]
  pub with_remote: bool,

  /// Persist distilled prose rules, read from STDIN, for a convention file
  #[arg(long, value_name = "FILE")]
  pub save_prose: Option<String>,
}

/// Arguments for `gb skill`.
#[derive(Debug, clap::Args)]
pub struct SkillArgs {
  #[command(subcommand)]
  pub cmd: SkillCmd,
}

/// Operations on the embedded protocol document.
#[derive(Debug, Subcommand)]
pub enum SkillCmd {
  /// Print the embedded protocol document to stdout
  Print,

  /// List the resolved target path for every agent
  Path,

  /// Write the protocol document into agent config paths
  Install(SkillInstallArgs),
}

/// Arguments for `gb skill install`.
#[derive(Debug, clap::Args)]
pub struct SkillInstallArgs {
  /// Install only this target; repeatable. Bypasses detection and creates parents
  #[arg(long, value_name = "TARGET")]
  pub target: Vec<String>,

  /// Install every target, creating parents
  #[arg(long)]
  pub all: bool,

  /// Report what would change without writing anything
  #[arg(long)]
  pub dry_run: bool,

  /// Rewrite even when unchanged, and overwrite foreign content
  #[arg(long)]
  pub force: bool,
}

/// Dispatches the parsed command line.
pub fn run(cli: Cli) -> anyhow::Result<()> {
  let mode = OutputMode::from_flags(cli.plain, cli.better);
  match cli.cmd {
    Cmd::Status(args) => status::run(args, mode),
    Cmd::Diff(args) => diff::run(args, mode),
    Cmd::Log(args) => log::run(args, mode),
    Cmd::Show(args) => show::run(args, mode),
    Cmd::Branch(args) => branch::run(args, mode),
    Cmd::Reflog(args) => reflog::run(args, mode),
    Cmd::Conventions(args) => conventions::run(args, mode),
    Cmd::Skill(args) => skill::run(args, mode),
    Cmd::Passthrough(args) => passthrough::run(&args),
  }
}

mod passthrough {
  use crate::git::proc;

  pub fn run(args: &[String]) -> anyhow::Result<()> {
    if args.is_empty() {
      return Err(anyhow::anyhow!("usage: gb <git-subcommand> [args...]"));
    }
    let out = proc::run_git(args)?;
    proc::write_to_stdout(&out);
    if !out.is_empty() && !out.ends_with('\n') {
      println!();
    }
    Ok(())
  }
}

/// `gb branch`.
pub mod branch;
/// `gb conventions`.
pub mod conventions;
/// `gb diff`.
pub mod diff;
/// `gb log`.
pub mod log;
/// `gb reflog`.
pub mod reflog;
/// `gb show`.
pub mod show;
/// `gb skill`.
pub mod skill;
/// `gb status`.
pub mod status;
