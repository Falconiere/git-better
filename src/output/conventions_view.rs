use crate::conventions::Profile;
use crate::output::OutputMode;
use crate::output::layout::pad_right;
use crate::output::theme::Theme;

const LABEL_WIDTH: usize = 9;

/// Prints the five-line convention summary: commit, branch, PR, release, prose.
pub fn print_summary(profile: &Profile, mode: OutputMode) {
  let theme = Theme::detect_with(!mode.is_pretty());

  let convention = if profile.commit_format.convention == "conventional-commits" {
    theme.accent(&profile.commit_format.convention)
  } else {
    theme.warn(&profile.commit_format.convention)
  };
  emit(
    &theme,
    "commit:",
    format!(
      "{convention} | scope {} | suffix {}",
      profile.commit_format.scope,
      profile.commit_format.pr_suffix.as_deref().unwrap_or("none")
    ),
  );
  emit(
    &theme,
    "branch:",
    format!(
      "{} {}",
      theme.branch(&profile.branch_naming.pattern),
      list(&profile.branch_naming.prefixes)
    ),
  );
  emit(
    &theme,
    "pr:",
    format!(
      "template {} | sections {}",
      profile.pr.template_path.as_deref().unwrap_or("none"),
      list(&profile.pr.body_sections)
    ),
  );
  emit(
    &theme,
    "release:",
    format!(
      "{} | {}",
      list(&profile.release.tooling),
      profile.release.version_commit.as_deref().unwrap_or("none")
    ),
  );
  emit(
    &theme,
    "prose:",
    format!("pending {}", list(&profile.prose_pending)),
  );
}

fn emit(theme: &Theme, label: &str, value: String) {
  println!("{}{value}", theme.dim(&pad_right(label, LABEL_WIDTH)));
}

fn list(items: &[String]) -> String {
  format!("[{}]", items.join(", "))
}
