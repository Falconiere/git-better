use clap::Parser;
use git_better::error::GbError;

fn main() {
  let cli = git_better::cli::Cli::parse();
  if let Err(e) = git_better::cli::run(cli) {
    eprintln!("{e:#}");
    let code = e
      .downcast_ref::<GbError>()
      .and_then(|g| match g {
        GbError::GitFailed { code, .. } => Some(*code),
        _ => None,
      })
      .unwrap_or(1);
    std::process::exit(code);
  }
}
