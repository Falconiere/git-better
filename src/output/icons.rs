use std::env;

#[derive(Debug, Clone, Copy)]
pub struct Icons {
  pub staged: &'static str,
  pub unstaged: &'static str,
  pub untracked: &'static str,
  pub ahead: &'static str,
  pub behind: &'static str,
  pub push: &'static str,
  pub pull: &'static str,
  pub type_feat: &'static str,
  pub type_fix: &'static str,
  pub type_docs: &'static str,
  pub type_refactor: &'static str,
  pub type_perf: &'static str,
  pub type_test: &'static str,
  pub type_chore: &'static str,
  pub type_build: &'static str,
  pub type_ci: &'static str,
  pub type_other: &'static str,
  pub bullet: &'static str,
  pub lock: &'static str,
  pub current: &'static str,
}

pub fn detect(use_ascii: bool) -> Icons {
  if use_ascii || env::var_os("GB_ASCII").is_some() {
    ASCII
  } else {
    UNICODE
  }
}

#[allow(dead_code)]
pub fn use_ascii_env() -> bool {
  env::var_os("GB_ASCII").is_some()
}

const UNICODE: Icons = Icons {
  staged: "●",
  unstaged: "◐",
  untracked: "?",
  ahead: "⇡",
  behind: "⇣",
  push: "↑",
  pull: "↓",
  type_feat: "✨ feat",
  type_fix: "🐛 fix",
  type_docs: "📝 docs",
  type_refactor: "🔨 refactor",
  type_perf: "⚡ perf",
  type_test: "✅ test",
  type_chore: "🔧 chore",
  type_build: "📦 build",
  type_ci: "🚀 ci",
  type_other: "•",
  bullet: "•",
  lock: "🔒",
  current: "⎇",
};

const ASCII: Icons = Icons {
  staged: "[M]",
  unstaged: "[M]",
  untracked: "[?]",
  ahead: "^",
  behind: "v",
  push: "^",
  pull: "v",
  type_feat: "feat:",
  type_fix: "fix:",
  type_docs: "docs:",
  type_refactor: "refactor:",
  type_perf: "perf:",
  type_test: "test:",
  type_chore: "chore:",
  type_build: "build:",
  type_ci: "ci:",
  type_other: "*",
  bullet: "*",
  lock: "[lock]",
  current: "@",
};

pub fn type_tag(icons: &Icons, kind: &str) -> &'static str {
  match kind {
    "feat" => icons.type_feat,
    "fix" => icons.type_fix,
    "docs" => icons.type_docs,
    "refactor" => icons.type_refactor,
    "perf" => icons.type_perf,
    "test" => icons.type_test,
    "chore" => icons.type_chore,
    "build" => icons.type_build,
    "ci" => icons.type_ci,
    _ => icons.type_other,
  }
}
