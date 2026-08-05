use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Reads, validates, and refreshes cached convention profiles.
pub mod cache;
/// Infers a convention profile from git history and declared files.
pub mod detect;
/// Stable, dependency-free digest used for cache keys.
pub mod hash;

/// Version of the convention-profile shape, on disk and in `--better` output.
pub const SCHEMA_VERSION: u32 = 1;

/// Everything `gb` infers about a repository's commit, branch, PR, and release style.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
  /// Shape version of this profile.
  pub schema_version: u32,
  /// Absolute path of the repository the profile describes.
  pub repo_root: String,
  /// RFC 3339 UTC timestamp of the last recomputation.
  pub generated_at: String,
  /// Digest of the declared convention files, used for cache invalidation.
  pub source_hash: String,
  /// Commit-message conventions inferred from recent subjects.
  pub commit_format: CommitFormat,
  /// Branch-naming conventions inferred from local and remote branches.
  pub branch_naming: BranchNaming,
  /// Pull-request template and title conventions.
  pub pr: PullRequest,
  /// Release tooling detected in the repository.
  pub release: Release,
  /// Issue-template conventions.
  pub issues: Issues,
  /// Prose convention files still awaiting a one-time distillation.
  pub prose_pending: Vec<String>,
  /// Distilled prose rules, keyed by repo-relative file path.
  pub prose_distilled: BTreeMap<String, ProseEntry>,
  /// Whether the `gh` binary is on `PATH`.
  pub gh_available: bool,
  /// Whether a remote lookup actually ran for this profile.
  pub remote_consulted: bool,
}

/// Commit-message conventions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitFormat {
  /// `conventional-commits` when a majority of recent subjects match, else `unknown`.
  pub convention: String,
  /// Conventional-commit types seen in the window, sorted and deduplicated.
  pub types: Vec<String>,
  /// `used` when subjects carry `(scope)`, else `none`.
  pub scope: String,
  /// `(#N)` when subjects end with a pull-request number.
  pub pr_suffix: Option<String>,
  /// Up to three recent subjects, as evidence.
  pub samples: Vec<String>,
}

/// Branch-naming conventions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchNaming {
  /// `type/kebab` when a majority of branches are prefixed, else `unknown`.
  pub pattern: String,
  /// Branch prefixes seen, sorted and deduplicated.
  pub prefixes: Vec<String>,
  /// Up to two prefixed branch names, as evidence.
  pub examples: Vec<String>,
}

/// Pull-request conventions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
  /// Repo-relative path of the pull-request template, when present.
  pub template_path: Option<String>,
  /// Title format, mirroring the commit convention.
  pub title_format: String,
  /// Heading titles found in the pull-request template.
  pub body_sections: Vec<String>,
  /// Recent pull-request titles, populated only with `--with-remote`.
  pub recent_titles: Vec<String>,
}

/// Release conventions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
  /// Release tooling detected, such as `release-please` or `semantic-release`.
  pub tooling: Vec<String>,
  /// Release-commit subject pattern, when the history shows one.
  pub version_commit: Option<String>,
  /// Changelog file, when present.
  pub changelog: Option<String>,
}

/// Issue-template conventions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issues {
  /// Repo-relative path of the bug-report template, when present.
  pub bug_template_path: Option<String>,
  /// Fields the bug template marks required.
  pub required_fields: Vec<String>,
}

/// Distilled rules for one prose convention file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProseEntry {
  /// Digest of the file contents the rules were distilled from.
  pub hash: String,
  /// The distilled rules themselves.
  pub rules: String,
}
