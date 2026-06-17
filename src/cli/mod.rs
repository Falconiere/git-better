use clap::{Parser, Subcommand};

use crate::output::OutputMode;

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

    /// Pass through to `git` (anything not handled above)
    #[command(external_subcommand)]
    Passthrough(Vec<String>),
}

#[derive(Debug, clap::Args)]
pub struct StatusArgs {
    /// Pass remaining args to `git status`
    #[arg(allow_hyphen_values = true, trailing_var_arg = true)]
    pub passthrough: Vec<String>,
}

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

#[derive(Debug, clap::Args)]
pub struct BranchArgs {
    /// Pass remaining args to `git branch`
    #[arg(allow_hyphen_values = true, trailing_var_arg = true)]
    pub passthrough: Vec<String>,
}

#[derive(Debug, clap::Args)]
pub struct ReflogArgs {
    /// Number of entries to show (default 50)
    #[arg(short = 'n', long)]
    pub n: Option<usize>,

    /// Pass remaining args to `git reflog`
    #[arg(allow_hyphen_values = true, trailing_var_arg = true)]
    pub passthrough: Vec<String>,
}

pub fn run(cli: Cli) -> anyhow::Result<()> {
    let mode = OutputMode::from_flags(cli.plain, cli.better);
    match cli.cmd {
        Cmd::Status(args) => status::run(args, mode),
        Cmd::Diff(args) => diff::run(args, mode),
        Cmd::Log(args) => log::run(args, mode),
        Cmd::Show(args) => show::run(args, mode),
        Cmd::Branch(args) => branch::run(args, mode),
        Cmd::Reflog(args) => reflog::run(args, mode),
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

pub mod branch;
pub mod diff;
pub mod log;
pub mod reflog;
pub mod show;
pub mod status;
