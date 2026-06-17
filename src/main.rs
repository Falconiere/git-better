use clap::Parser;

fn main() -> anyhow::Result<()> {
  let cli = git_better::cli::Cli::parse();
  git_better::cli::run(cli)
}
