use std::collections::BTreeSet;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::LazyLock;
use std::time::{Duration, Instant};

use regex::Regex;
use serde::Deserialize;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::conventions::hash::fnv1a_hex;
use crate::conventions::{
  BranchNaming, CommitFormat, Issues, Profile, PullRequest, Release, SCHEMA_VERSION,
};
use crate::error::GbError;
use crate::git::proc;

/// Number of recent commit subjects the convention majority is decided over.
///
/// Deliberately narrow: the profile must track the repository's *current*
/// practice, and a wider window dilutes a recently adopted convention.
const SUBJECT_WINDOW: usize = 50;

const REMOTE_TIMEOUT: Duration = Duration::from_secs(5);
const REMOTE_POLL: Duration = Duration::from_millis(50);

static CONVENTIONAL_RE: LazyLock<Option<Regex>> = LazyLock::new(|| {
  Regex::new(
    r"^(?P<type>feat|fix|chore|docs|test|refactor|perf|build|ci|style|revert)(?:\((?P<scope>[^)]+)\))?!?: ",
  )
  .ok()
});

static PR_SUFFIX_RE: LazyLock<Option<Regex>> = LazyLock::new(|| Regex::new(r" \(#\d+\)$").ok());

static RELEASE_SUBJECT_RE: LazyLock<Option<Regex>> =
  LazyLock::new(|| Regex::new(r"^chore\(release\): v").ok());

static HEADING_RE: LazyLock<Option<Regex>> = LazyLock::new(|| Regex::new(r"^#{1,3} +(.+)$").ok());

const DECLARED_FILES: &[&str] = &[
  ".github/PULL_REQUEST_TEMPLATE.md",
  ".github/pull_request_template.md",
  "CONTRIBUTING.md",
  ".czrc",
  "cz.json",
  ".releaserc",
  "release-please-config.json",
  ".release-please-manifest.json",
  ".github/release.yml",
  ".gitmessage",
  "CODEOWNERS",
  ".github/CODEOWNERS",
];

const DECLARED_PREFIXES: &[&str] = &[".releaserc", "commitlint"];

const PR_TEMPLATES: &[&str] = &[
  ".github/PULL_REQUEST_TEMPLATE.md",
  ".github/pull_request_template.md",
];

#[derive(Debug, Deserialize)]
struct RemoteTitle {
  title: String,
}

/// Returns the absolute top level of the repository containing the current directory.
pub fn repo_root() -> Result<PathBuf, GbError> {
  let out = proc::run_git(&["rev-parse".to_string(), "--show-toplevel".to_string()])?;
  let trimmed = out.trim();
  if trimmed.is_empty() {
    return Err(GbError::NotARepository);
  }
  Ok(PathBuf::from(trimmed))
}

/// Returns the convention files whose contents invalidate a cached profile, sorted.
pub fn declared_files(root: &Path) -> Vec<PathBuf> {
  let mut found = BTreeSet::new();
  for rel in DECLARED_FILES {
    let path = root.join(rel);
    if path.is_file() {
      found.insert(path);
    }
  }
  for dir in [root.to_path_buf(), root.join(".github")] {
    collect_prefixed(&dir, DECLARED_PREFIXES, &mut found);
  }
  for dir in [
    root.join(".github/PULL_REQUEST_TEMPLATE"),
    root.join(".github/ISSUE_TEMPLATE"),
  ] {
    collect_all(&dir, &mut found);
  }
  found.into_iter().collect()
}

/// Returns the digest of every declared convention file's contents, in path order.
pub fn source_hash(root: &Path) -> String {
  let mut buf = Vec::new();
  for path in declared_files(root) {
    match std::fs::read(&path) {
      Ok(bytes) => buf.extend_from_slice(&bytes),
      Err(_) => continue,
    }
  }
  fnv1a_hex(&buf)
}

/// Returns true when `subject` matches the conventional-commit prefix grammar.
pub fn is_conventional(subject: &str) -> bool {
  conventional_type(subject).is_some()
}

/// Returns the conventional-commit type of `subject`, when it has one.
pub fn conventional_type(subject: &str) -> Option<String> {
  let re = CONVENTIONAL_RE.as_ref()?;
  let caps = re.captures(subject)?;
  caps.name("type").map(|m| m.as_str().to_string())
}

/// Returns true when `subject` carries a conventional-commit scope.
pub fn has_scope(subject: &str) -> bool {
  CONVENTIONAL_RE
    .as_ref()
    .and_then(|re| re.captures(subject))
    .and_then(|caps| caps.name("scope").map(|_| ()))
    .is_some()
}

/// Returns true when `hits` is at least half of a non-zero `total`.
pub fn is_majority(hits: usize, total: usize) -> bool {
  total > 0 && hits * 2 >= total
}

/// Builds a fresh convention profile for `root`.
///
/// Local git and filesystem only, unless `with_remote` allows one bounded `gh`
/// lookup. Missing history, missing branches, and unreadable files degrade to
/// empty findings rather than failing.
pub fn build_profile(root: &Path, with_remote: bool) -> Result<Profile, GbError> {
  let subjects = recent_subjects(root);
  let commit_format = commit_format(&subjects);
  let branch_naming = branch_naming(root);
  let (template_path, body_sections) = pr_template(root);
  let remote_titles = if with_remote {
    recent_pr_titles(root)
  } else {
    Vec::new()
  };

  Ok(Profile {
    schema_version: SCHEMA_VERSION,
    repo_root: root.display().to_string(),
    generated_at: now_rfc3339()?,
    source_hash: source_hash(root),
    pr: PullRequest {
      template_path,
      title_format: commit_format.convention.clone(),
      body_sections,
      recent_titles: remote_titles,
    },
    commit_format,
    branch_naming,
    release: release(root, &subjects),
    issues: Issues {
      bug_template_path: bug_template(root),
      required_fields: Vec::new(),
    },
    prose_pending: Vec::new(),
    prose_distilled: Default::default(),
    gh_available: gh_on_path(),
    remote_consulted: with_remote,
  })
}

fn collect_prefixed(dir: &Path, prefixes: &[&str], out: &mut BTreeSet<PathBuf>) {
  let entries = match std::fs::read_dir(dir) {
    Ok(entries) => entries,
    Err(_) => return,
  };
  for entry in entries.flatten() {
    let path = entry.path();
    if !path.is_file() {
      continue;
    }
    let name = entry.file_name();
    let name = name.to_string_lossy();
    if prefixes.iter().any(|p| name.starts_with(p)) {
      out.insert(path);
    }
  }
}

fn collect_all(dir: &Path, out: &mut BTreeSet<PathBuf>) {
  let entries = match std::fs::read_dir(dir) {
    Ok(entries) => entries,
    Err(_) => return,
  };
  for entry in entries.flatten() {
    let path = entry.path();
    if path.is_file() {
      out.insert(path);
    }
  }
}

fn recent_subjects(root: &Path) -> Vec<String> {
  let args = vec![
    "-C".to_string(),
    root.display().to_string(),
    "log".to_string(),
    format!("-n{SUBJECT_WINDOW}"),
    "--format=%s".to_string(),
  ];
  proc::run_git(&args)
    .unwrap_or_default()
    .lines()
    .map(str::trim)
    .filter(|line| !line.is_empty())
    .map(String::from)
    .collect()
}

fn commit_format(subjects: &[String]) -> CommitFormat {
  let hits = subjects.iter().filter(|s| is_conventional(s)).count();
  let types: BTreeSet<String> = subjects
    .iter()
    .filter_map(|s| conventional_type(s))
    .collect();
  let pr_suffix = PR_SUFFIX_RE
    .as_ref()
    .filter(|re| subjects.iter().any(|s| re.is_match(s)))
    .map(|_| "(#N)".to_string());

  CommitFormat {
    convention: if is_majority(hits, subjects.len()) {
      "conventional-commits".to_string()
    } else {
      "unknown".to_string()
    },
    types: types.into_iter().collect(),
    scope: if subjects.iter().any(|s| has_scope(s)) {
      "used".to_string()
    } else {
      "none".to_string()
    },
    pr_suffix,
    samples: subjects.iter().take(3).cloned().collect(),
  }
}

fn branch_naming(root: &Path) -> BranchNaming {
  let args = vec![
    "-C".to_string(),
    root.display().to_string(),
    "branch".to_string(),
    "-a".to_string(),
    "--format=%(refname:short)".to_string(),
  ];
  let raw = proc::run_git(&args).unwrap_or_default();
  let names: BTreeSet<String> = raw
    .lines()
    .map(str::trim)
    .filter(|line| !line.is_empty() && *line != "HEAD")
    .map(|line| line.strip_prefix("origin/").unwrap_or(line).to_string())
    .filter(|line| line != "HEAD")
    .collect();

  let prefixed: Vec<&String> = names.iter().filter(|n| is_prefixed(n)).collect();
  let prefixes: BTreeSet<String> = prefixed
    .iter()
    .filter_map(|n| n.split_once('/').map(|(p, _)| p.to_string()))
    .collect();

  BranchNaming {
    pattern: if is_majority(prefixed.len(), names.len()) {
      "type/kebab".to_string()
    } else {
      "unknown".to_string()
    },
    prefixes: prefixes.into_iter().collect(),
    examples: prefixed.iter().take(2).map(|n| n.to_string()).collect(),
  }
}

fn is_prefixed(name: &str) -> bool {
  match name.split_once('/') {
    Some((prefix, rest)) => {
      !prefix.is_empty() && !rest.is_empty() && prefix.chars().all(|c| c.is_ascii_lowercase())
    },
    None => false,
  }
}

fn pr_template(root: &Path) -> (Option<String>, Vec<String>) {
  for rel in PR_TEMPLATES {
    let path = root.join(rel);
    let text = match std::fs::read_to_string(&path) {
      Ok(text) => text,
      Err(_) => continue,
    };
    let sections = match HEADING_RE.as_ref() {
      Some(re) => text
        .lines()
        .filter_map(|line| re.captures(line))
        .filter_map(|caps| caps.get(1).map(|m| m.as_str().trim().to_string()))
        .collect(),
      None => Vec::new(),
    };
    return (Some((*rel).to_string()), sections);
  }
  (None, Vec::new())
}

fn release(root: &Path, subjects: &[String]) -> Release {
  let mut tooling = Vec::new();
  if root.join(".github/workflows/release.yml").is_file() {
    tooling.push("release.yml-workflow".to_string());
  }
  if root.join("release-please-config.json").is_file()
    || root.join(".release-please-manifest.json").is_file()
  {
    tooling.push("release-please".to_string());
  }
  if has_semantic_release(root) {
    tooling.push("semantic-release".to_string());
  }

  let version_commit = RELEASE_SUBJECT_RE
    .as_ref()
    .filter(|re| subjects.iter().any(|s| re.is_match(s)))
    .map(|_| "chore(release): vX".to_string());

  Release {
    tooling,
    version_commit,
    changelog: root
      .join("CHANGELOG.md")
      .is_file()
      .then(|| "CHANGELOG.md".to_string()),
  }
}

fn has_semantic_release(root: &Path) -> bool {
  let mut releaserc = BTreeSet::new();
  collect_prefixed(root, &[".releaserc"], &mut releaserc);
  if !releaserc.is_empty() {
    return true;
  }
  match std::fs::read_to_string(root.join("package.json")) {
    Ok(text) => text.contains("\"semantic-release\""),
    Err(_) => false,
  }
}

fn bug_template(root: &Path) -> Option<String> {
  let dir = root.join(".github/ISSUE_TEMPLATE");
  let entries = std::fs::read_dir(&dir).ok()?;
  let mut matches = BTreeSet::new();
  for entry in entries.flatten() {
    let name = entry.file_name().to_string_lossy().to_lowercase();
    if name.contains("bug") && entry.path().is_file() {
      matches.insert(format!(
        ".github/ISSUE_TEMPLATE/{}",
        entry.file_name().to_string_lossy()
      ));
    }
  }
  matches.into_iter().next()
}

fn gh_on_path() -> bool {
  let path = match std::env::var_os("PATH") {
    Some(path) => path,
    None => return false,
  };
  std::env::split_paths(&path).any(|dir| dir.join("gh").is_file())
}

fn recent_pr_titles(root: &Path) -> Vec<String> {
  match gh_pr_list(root) {
    Some(raw) => match serde_json::from_str::<Vec<RemoteTitle>>(&raw) {
      Ok(titles) => titles.into_iter().map(|t| t.title).collect(),
      Err(_) => Vec::new(),
    },
    None => Vec::new(),
  }
}

fn gh_pr_list(root: &Path) -> Option<String> {
  let mut child = Command::new("gh")
    .args(["pr", "list", "--limit", "10", "--json", "title"])
    .current_dir(root)
    .stdout(Stdio::piped())
    .stderr(Stdio::null())
    .spawn()
    .ok()?;

  let deadline = Instant::now() + REMOTE_TIMEOUT;
  loop {
    match child.try_wait() {
      Ok(Some(status)) if status.success() => break,
      Ok(Some(_)) => return None,
      Ok(None) => {
        if Instant::now() >= deadline {
          let _ = child.kill();
          let _ = child.wait();
          return None;
        }
        std::thread::sleep(REMOTE_POLL);
      },
      Err(_) => return None,
    }
  }

  let mut buf = String::new();
  let mut stdout = child.stdout.take()?;
  match stdout.read_to_string(&mut buf) {
    Ok(_) => Some(buf),
    Err(_) => None,
  }
}

fn now_rfc3339() -> Result<String, GbError> {
  OffsetDateTime::now_utc()
    .format(&Rfc3339)
    .map_err(|e| GbError::Other(anyhow::anyhow!("cannot format the current time: {e}")))
}
