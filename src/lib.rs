/// Command-line surface: clap definitions and per-subcommand dispatch.
pub mod cli;
/// Repository convention profile: detection, caching, and prose distillation.
pub mod conventions;
/// Error type shared by the git and output layers.
pub mod error;
/// Git shell-out helpers and porcelain parsers.
pub mod git;
/// Human and JSON rendering.
pub mod output;
