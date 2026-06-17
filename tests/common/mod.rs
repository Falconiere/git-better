use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

#[allow(dead_code)]
pub fn init_repo() -> TempDir {
  let dir = tempfile::tempdir().unwrap();
  let init = Command::new("git")
    .args(["init", "-q", "-b", "main"])
    .current_dir(dir.path())
    .status()
    .unwrap();
  assert!(init.success(), "git init failed");
  for kv in [
    ("user.email", "test@example.com"),
    ("user.name", "Test"),
    ("commit.gpgsign", "false"),
    ("tag.gpgsign", "false"),
  ] {
    let _ = Command::new("git")
      .args(["config", kv.0, kv.1])
      .current_dir(dir.path())
      .status()
      .unwrap();
  }
  std::fs::write(dir.path().join("README.md"), "# test\n").unwrap();
  let _ = Command::new("git")
    .args(["add", "README.md"])
    .current_dir(dir.path())
    .status()
    .unwrap();
  let _ = Command::new("git")
    .args(["commit", "-q", "-m", "init"])
    .current_dir(dir.path())
    .status()
    .unwrap();
  std::fs::write(dir.path().join("dirty.txt"), "uncommitted\n").unwrap();
  dir
}

#[allow(dead_code)]
pub fn init_repo_with_tracked_change() -> TempDir {
  let dir = init_repo();
  let p = dir.path();
  std::fs::write(p.join("dirty.txt"), "original\n").unwrap();
  run_git(p, &["add", "dirty.txt"]);
  run_git(p, &["commit", "-q", "-m", "add dirty"]);
  std::fs::write(p.join("dirty.txt"), "modified\n").unwrap();
  dir
}

#[allow(dead_code)]
pub fn git_stdout(args: &[&str], dir: &Path) -> String {
  let mut full = vec!["-c", "color.ui=false"];
  full.extend_from_slice(args);
  let out = Command::new("git")
    .args(&full)
    .current_dir(dir)
    .output()
    .unwrap();
  assert!(out.status.success(), "git invocation failed");
  String::from_utf8(out.stdout).unwrap()
}

#[allow(dead_code)]
pub fn run_git(dir: &Path, args: &[&str]) {
  let _ = Command::new("git")
    .args(args)
    .current_dir(dir)
    .status()
    .unwrap();
}

#[allow(dead_code)]
pub fn make_branch(dir: &Path, name: &str) {
  run_git(dir, &["checkout", "-q", "-b", name]);
}

#[allow(dead_code)]
pub fn make_commit(dir: &Path, msg: &str) {
  run_git(dir, &["commit", "--allow-empty", "-q", "-m", msg]);
}

#[allow(dead_code)]
pub fn init_with_pr_trail() -> TempDir {
  let dir = init_repo();
  let p = dir.path();
  make_branch(p, "feat/oauth");
  make_commit(p, "feat(auth): add OAuth2 PKCE flow (#142)");
  make_commit(p, "fix(auth): handle expired verifier (#142)");
  make_commit(p, "docs(readme): document PKCE flow (#142)");
  make_commit(p, "chore: bump deps");
  dir
}

#[allow(dead_code)]
pub fn init_lockfile_heavy() -> TempDir {
  let dir = init_repo();
  let p = dir.path();
  std::fs::write(
    p.join("Cargo.lock"),
    "[package]\nname = \"x\"\nversion = \"0.1.0\"\n",
  )
  .unwrap();
  std::fs::write(p.join("package-lock.json"), "{}\n").unwrap();
  std::fs::write(p.join("bun.lock"), "{\"lockfileVersion\":1}\n").unwrap();
  std::fs::write(p.join("main.rs"), "fn main() {}\n").unwrap();
  run_git(p, &["add", "."]);
  run_git(p, &["commit", "-q", "-m", "chore: initial lockfiles"]);
  std::fs::write(
    p.join("Cargo.lock"),
    "[package]\nname = \"x\"\nversion = \"0.2.0\"\n",
  )
  .unwrap();
  std::fs::write(p.join("package-lock.json"), "{ \"version\": 2 }\n").unwrap();
  std::fs::write(p.join("main.rs"), "fn main() {\n    println!(\"hi\");\n}\n").unwrap();
  dir
}

#[allow(dead_code)]
pub fn init_syntax_heavy() -> TempDir {
  let dir = init_repo();
  let p = dir.path();
  std::fs::write(
    p.join("app.rs"),
    r#"fn greet(name: &str) -> String {
    format!("Hello, {name}!")
}

fn main() {
    let msg = greet("world");
    println!("{msg}");
}
"#,
  )
  .unwrap();
  run_git(p, &["add", "app.rs"]);
  run_git(p, &["commit", "-q", "-m", "feat: add greet function"]);
  std::fs::write(
    p.join("app.rs"),
    r#"fn greet(name: &str) -> String {
    let suffix = if name.is_empty() { "stranger" } else { name };
    format!("Hello, {suffix}!")
}

fn farewell(name: &str) -> String {
    format!("Goodbye, {name}!")
}

fn main() {
    let msg = greet("world");
    let bye = farewell("world");
    println!("{msg}");
    println!("{bye}");
}
"#,
  )
  .unwrap();
  dir
}
