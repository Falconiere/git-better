use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::conventions::hash::fnv1a_hex;
use crate::conventions::{Profile, ProseEntry, SCHEMA_VERSION, detect};
use crate::error::GbError;

const MAX_AGE: Duration = Duration::from_secs(7 * 24 * 60 * 60);
const MAX_PROSE_BYTES: usize = 8 * 1024;
const PROSE_CANDIDATES: &[&str] = &["CONTRIBUTING.md"];

/// Distilled prose rules to persist for one convention file.
#[derive(Debug, Clone)]
pub struct ProseSave {
  /// Repo-relative path of the file the rules were distilled from.
  pub file: String,
  /// The distilled rules.
  pub rules: String,
}

/// A profile plus whether it was served from cache.
#[derive(Debug, Clone)]
pub struct Resolved {
  /// The convention profile.
  pub profile: Profile,
  /// True when the profile came from a fresh cache entry.
  pub cache_hit: bool,
}

/// Returns the directory holding cached profiles.
///
/// `GB_CACHE_DIR` wins, then `XDG_CACHE_HOME/git-better`, then
/// `$HOME/.cache/git-better`. `None` when no home is discoverable.
pub fn cache_dir() -> Option<PathBuf> {
  if let Some(dir) = non_empty_env("GB_CACHE_DIR") {
    return Some(dir);
  }
  if let Some(dir) = non_empty_env("XDG_CACHE_HOME") {
    return Some(dir.join("git-better"));
  }
  non_empty_env("HOME").map(|home| home.join(".cache").join("git-better"))
}

/// Returns the cache file for `root`.
pub fn cache_path(root: &Path) -> Option<PathBuf> {
  let key = fnv1a_hex(root.display().to_string().as_bytes());
  cache_dir().map(|dir| dir.join("conventions").join(format!("{key}.json")))
}

/// Returns a convention profile for `root`, recomputing it only when needed.
///
/// A cached profile is reused when it parses, matches the current schema,
/// repository, and declared-file digest, and is at most seven days old.
/// `refresh` and `save` always force a recomputation. Cache read and write
/// failures degrade to computing and printing; they never fail the command.
pub fn resolve(
  root: &Path,
  with_remote: bool,
  refresh: bool,
  save: Option<ProseSave>,
) -> Result<Resolved, GbError> {
  let current_hash = detect::source_hash(root);
  let path = cache_path(root);

  if !refresh && save.is_none() {
    if let Some(profile) = fresh_cached(path.as_deref(), root, &current_hash) {
      return Ok(Resolved {
        profile,
        cache_hit: true,
      });
    }
  }

  let distilled = merged_prose(path.as_deref(), root, save)?;
  let mut profile = detect::build_profile(root, with_remote)?;
  profile.prose_pending = pending_prose(root, &distilled);
  profile.prose_distilled = distilled;

  if let Some(path) = path.as_deref() {
    write_profile(path, &profile);
  }

  Ok(Resolved {
    profile,
    cache_hit: false,
  })
}

fn non_empty_env(key: &str) -> Option<PathBuf> {
  let value = std::env::var_os(key)?;
  if value.is_empty() {
    return None;
  }
  Some(PathBuf::from(value))
}

fn fresh_cached(path: Option<&Path>, root: &Path, current_hash: &str) -> Option<Profile> {
  let path = path?;
  let cached = read_profile(path)?;
  if !is_fresh(&cached, root, current_hash) || !is_within_max_age(path) {
    return None;
  }
  Some(cached)
}

fn merged_prose(
  path: Option<&Path>,
  root: &Path,
  save: Option<ProseSave>,
) -> Result<BTreeMap<String, ProseEntry>, GbError> {
  let mut distilled = path
    .and_then(read_profile)
    .map(|prior| prior.prose_distilled)
    .unwrap_or_default();
  if let Some(save) = save {
    let bytes = fs::read(root.join(&save.file))?;
    distilled.insert(
      save.file,
      ProseEntry {
        hash: fnv1a_hex(&bytes),
        rules: truncate_rules(&save.rules),
      },
    );
  }
  Ok(distilled)
}

fn read_profile(path: &Path) -> Option<Profile> {
  let text = fs::read_to_string(path).ok()?;
  serde_json::from_str::<Profile>(&text).ok()
}

fn is_fresh(cached: &Profile, root: &Path, current_hash: &str) -> bool {
  cached.schema_version == SCHEMA_VERSION
    && cached.repo_root == root.display().to_string()
    && cached.source_hash == current_hash
}

fn is_within_max_age(path: &Path) -> bool {
  let modified = match fs::metadata(path).and_then(|meta| meta.modified()) {
    Ok(modified) => modified,
    Err(_) => return false,
  };
  match modified.elapsed() {
    Ok(age) => age <= MAX_AGE,
    Err(_) => true,
  }
}

fn pending_prose(root: &Path, distilled: &BTreeMap<String, ProseEntry>) -> Vec<String> {
  let mut pending = Vec::new();
  for candidate in PROSE_CANDIDATES {
    let bytes = match fs::read(root.join(candidate)) {
      Ok(bytes) => bytes,
      Err(_) => continue,
    };
    let current = fnv1a_hex(&bytes);
    let known = distilled.get(*candidate).map(|entry| entry.hash.as_str());
    if known != Some(current.as_str()) {
      pending.push((*candidate).to_string());
    }
  }
  pending
}

fn truncate_rules(rules: &str) -> String {
  if rules.len() <= MAX_PROSE_BYTES {
    return rules.to_string();
  }
  let mut end = MAX_PROSE_BYTES;
  while end > 0 && !rules.is_char_boundary(end) {
    end -= 1;
  }
  rules[..end].to_string()
}

fn write_profile(path: &Path, profile: &Profile) {
  let Some(parent) = path.parent() else {
    return;
  };
  if fs::create_dir_all(parent).is_err() {
    return;
  }
  let Ok(text) = serde_json::to_string_pretty(profile) else {
    return;
  };
  let tmp = parent.join(format!(".{}.tmp", std::process::id()));
  if fs::write(&tmp, format!("{text}\n")).is_err() {
    let _ = fs::remove_file(&tmp);
    return;
  }
  if fs::rename(&tmp, path).is_err() {
    let _ = fs::remove_file(&tmp);
  }
}
