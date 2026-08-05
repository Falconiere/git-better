use git_better::conventions::detect;
use git_better::conventions::hash::fnv1a_hex;

mod common;

#[test]
fn majority_is_inclusive_at_half() {
  assert!(detect::is_majority(1, 2));
  assert!(detect::is_majority(25, 50));
  assert!(!detect::is_majority(24, 50));
  assert!(!detect::is_majority(1, 3));
}

#[test]
fn majority_of_nothing_is_never_a_majority() {
  assert!(!detect::is_majority(0, 0));
}

#[test]
fn conventional_type_reads_the_prefix() {
  assert_eq!(
    detect::conventional_type("feat(auth): add OAuth2 PKCE flow (#142)"),
    Some("feat".to_string())
  );
  assert_eq!(
    detect::conventional_type("style: apply rustfmt"),
    Some("style".to_string())
  );
  assert_eq!(
    detect::conventional_type("fix!: drop the broken flag"),
    Some("fix".to_string())
  );
}

#[test]
fn non_conventional_subjects_have_no_type() {
  assert_eq!(detect::conventional_type("wip on the parser"), None);
  assert_eq!(detect::conventional_type("Merge pull request #1"), None);
  assert_eq!(detect::conventional_type("feat:missing space"), None);
  assert!(!detect::is_conventional("update readme"));
}

#[test]
fn scope_is_detected_only_when_present() {
  assert!(detect::has_scope("fix(auth): handle expired verifier"));
  assert!(!detect::has_scope("fix: handle expired verifier"));
}

#[test]
fn fnv1a_digest_is_stable() {
  assert_eq!(fnv1a_hex(b""), "cbf29ce484222325");
  assert_eq!(fnv1a_hex(b"git-better"), "26d901f0b1314ac4");
  assert_eq!(
    fnv1a_hex(b"feat(auth): add OAuth2 PKCE flow (#142)"),
    "c93f88cb46ac67ac"
  );
}

#[test]
fn declared_files_and_hash_track_convention_files() {
  let repo = common::init_repo();
  let root = repo.path();

  let empty_hash = detect::source_hash(root);
  assert!(detect::declared_files(root).is_empty());

  std::fs::write(root.join("CONTRIBUTING.md"), "# Contributing\n\nBe kind.\n").unwrap();
  let declared = detect::declared_files(root);
  assert_eq!(declared.len(), 1);
  assert!(declared[0].ends_with("CONTRIBUTING.md"));
  assert_ne!(detect::source_hash(root), empty_hash);
}

#[test]
fn profile_of_a_conventional_repo_reports_scope_and_pr_suffix() {
  let repo = common::init_with_pr_trail();
  let profile = detect::build_profile(repo.path(), false).unwrap();

  assert_eq!(profile.commit_format.convention, "conventional-commits");
  assert_eq!(profile.commit_format.scope, "used");
  assert_eq!(profile.commit_format.pr_suffix.as_deref(), Some("(#N)"));
  assert!(profile.commit_format.types.contains(&"feat".to_string()));
  assert!(profile.commit_format.types.contains(&"docs".to_string()));
  assert_eq!(profile.branch_naming.pattern, "type/kebab");
  assert_eq!(profile.branch_naming.prefixes, vec!["feat".to_string()]);
  assert!(!profile.remote_consulted);
  assert_eq!(profile.pr.recent_titles.len(), 0);
}

#[test]
fn profile_of_a_repo_without_commits_stays_empty_and_succeeds() {
  let dir = tempfile::tempdir().unwrap();
  common::run_git(dir.path(), &["init", "-q", "-b", "main"]);

  let profile = detect::build_profile(dir.path(), false).unwrap();

  assert_eq!(profile.commit_format.convention, "unknown");
  assert!(profile.commit_format.types.is_empty());
  assert!(profile.commit_format.samples.is_empty());
  assert_eq!(profile.commit_format.pr_suffix, None);
  assert_eq!(profile.branch_naming.pattern, "unknown");
}
