use crate::git::commit::CommitRecord;
use crate::git::diff::{DiffFile, DiffLine, DiffSummary, FileStat};
use crate::git::reflog::ReflogEntry;
use crate::output::theme::Theme;
use crate::output::{OutputMode, icons, layout};

fn diff_stat_header(theme: &Theme, summary: &DiffSummary, pretty: bool) -> String {
  if pretty {
    format!(
      "{} file{} · {}{}{} {}{}",
      summary.files_changed,
      if summary.files_changed == 1 { "" } else { "s" },
      theme.insertion(&format!("+{}", summary.added)),
      theme.dim(" / "),
      theme.deletion(&format!("-{}", summary.removed)),
      theme.dim("·"),
      if summary.files_changed == 1 {
        "file"
      } else {
        "files"
      }
    )
  } else {
    format!(
      "{} file(s) · +{} / -{}",
      summary.files_changed, summary.added, summary.removed
    )
  }
}

fn print_diff_stat_row(
  theme: &Theme,
  file: &FileStat,
  max_changes: u64,
  widths: (usize, usize),
  pretty: bool,
) {
  let (bar_width, path_width) = widths;
  let bar = layout::bar(file.added + file.removed, max_changes, bar_width);
  let bar_colored = if pretty {
    if file.added > file.removed {
      theme.insertion(&bar)
    } else if file.removed > file.added {
      theme.deletion(&bar)
    } else {
      theme.warn(&bar)
    }
  } else {
    bar
  };
  let path = layout::pad_right(&file.path, path_width);
  let added = if pretty {
    theme.insertion(&format!("+{}", file.added))
  } else {
    format!("+{}", file.added)
  };
  let removed = if pretty {
    theme.deletion(&format!("-{}", file.removed))
  } else {
    format!("-{}", file.removed)
  };
  println!(
    "  {bar_colored} {path} {added} {removed}",
    bar_colored = bar_colored,
    path = path,
    added = layout::pad_left(&added, 6),
    removed = layout::pad_left(&removed, 6)
  );
}

fn print_diff_stat_excludes(theme: &Theme, ic: &icons::Icons, pretty: bool) {
  if pretty {
    println!(
      "  {} {} {}",
      ic.lock,
      theme.dim("excluded:"),
      crate::git::proc::LOCKFILE_EXCLUDES.join(", ")
    );
  } else {
    println!(
      "  [lock] excluded: {}",
      crate::git::proc::LOCKFILE_EXCLUDES.join(", ")
    );
  }
}

pub fn print_diff_stat(files: &[FileStat], summary: &DiffSummary, mode: OutputMode) {
  let pretty = mode.is_pretty();
  let theme = Theme::detect_with(!pretty);
  let ic = icons::detect(!pretty);
  let term = layout::term_width();
  let rule = layout::horizontal_rule(term);
  let max_changes = files
    .iter()
    .map(|f| f.added + f.removed)
    .max()
    .unwrap_or(0)
    .max(1);
  let bar_width: usize = 20;
  let meta_width: usize = 30;
  let path_width = term.saturating_sub(bar_width + meta_width + 4);

  println!(
    "{rule}\n{} {} {}\n{}",
    theme.dim("diff ·"),
    diff_stat_header(&theme, summary, pretty),
    theme.dim("·"),
    rule
  );

  for file in files {
    print_diff_stat_row(&theme, file, max_changes, (bar_width, path_width), pretty);
  }

  println!("{rule}");
  print_diff_stat_excludes(&theme, &ic, pretty);
}

fn print_diff_full_file(file: &DiffFile, pretty: bool) {
  let syntax_path = if file.new_path == "/dev/null" {
    &file.old_path
  } else {
    &file.new_path
  };
  let syntax = crate::output::highlight::syntax_for_filename(syntax_path);
  let mut highlighter = crate::output::highlight::make_highlighter(syntax);
  print!(
    "{}",
    crate::output::highlight::render_file_header(&file.old_path, &file.new_path, pretty)
  );
  for hunk in &file.hunks {
    print!(
      "{}",
      crate::output::highlight::render_hunk_header(&hunk.header, pretty)
    );
    for line in &hunk.lines {
      let bg = match line {
        DiffLine::Context(_) => crate::output::highlight::LineBg::Context,
        DiffLine::Insertion(_) => crate::output::highlight::LineBg::Insertion,
        DiffLine::Deletion(_) => crate::output::highlight::LineBg::Deletion,
      };
      print!(
        "{}",
        crate::output::highlight::render_diff_line(
          highlighter.as_mut(),
          line.content(),
          bg,
          pretty
        )
      );
    }
  }
}

pub fn print_diff_full(files: &[DiffFile], mode: OutputMode) {
  let pretty = mode.is_pretty();
  let theme = Theme::detect_with(!pretty);
  let term = layout::term_width();
  let rule = layout::horizontal_rule(term);

  if files.is_empty() {
    println!("{rule}");
    println!("{}", theme.dim("no changes"));
    println!("{rule}");
    return;
  }

  println!("{rule}");
  println!("{}", theme.dim("diff · full"));
  println!("{rule}");

  for file in files {
    print_diff_full_file(file, pretty);
  }
  println!("{rule}");
}

fn type_tag_colored(theme: &Theme, ic: &icons::Icons, ctype: Option<&str>, pretty: bool) -> String {
  let type_str = match ctype {
    Some(t) => icons::type_tag(ic, t).to_string(),
    None => ic.type_other.to_string(),
  };
  if pretty {
    match ctype {
      Some("feat") | Some("perf") => theme.accent(&type_str),
      Some("fix") | Some("docs") => theme.warn(&type_str),
      _ => theme.dim(&type_str),
    }
  } else {
    type_str
  }
}

fn print_log_row(theme: &Theme, ic: &icons::Icons, r: &CommitRecord, pretty: bool) {
  let time_w = 12;
  let sha_w = 9;
  let type_colored = type_tag_colored(theme, ic, r.conventional_type.as_deref(), pretty);
  let sha_colored = if pretty {
    theme.dim(&r.short_sha)
  } else {
    r.short_sha.clone()
  };
  let time_colored = if pretty {
    theme.dim(&layout::pad_left(&r.time_relative, time_w))
  } else {
    layout::pad_left(&r.time_relative, time_w)
  };
  let pr_str = r
    .pr_number
    .map(|n| {
      if pretty {
        theme.accent(&format!(" #{n}"))
      } else {
        format!(" #{n}")
      }
    })
    .unwrap_or_default();
  println!(
    "  {type_colored} {sha_colored} {time_colored}   {subject}{pr}",
    type_colored = layout::pad_right(&type_colored, 16),
    sha_colored = layout::pad_right(&sha_colored, sha_w),
    time_colored = time_colored,
    subject = r.subject,
    pr = pr_str
  );
}

pub fn print_log(records: &[CommitRecord], mode: OutputMode) {
  let pretty = mode.is_pretty();
  let theme = Theme::detect_with(!pretty);
  let ic = icons::detect(!pretty);
  let term = layout::term_width();
  let rule = layout::horizontal_rule(term);

  if records.is_empty() {
    println!("{rule}");
    println!("{}", theme.dim("no commits"));
    println!("{rule}");
    return;
  }

  println!("{rule}");
  println!("{}", theme.dim("recent commits"));
  println!("{rule}");

  for r in records {
    print_log_row(&theme, &ic, r, pretty);
  }

  println!("{rule}");
}

pub struct LogStory<'a> {
  pub branch: &'a str,
  pub base: &'a str,
  pub total: u64,
  pub by_type: &'a std::collections::BTreeMap<String, u64>,
  pub files_changed: u64,
  pub net_added: u64,
  pub net_removed: u64,
  pub first_subject: &'a str,
  pub pr: Option<u64>,
}

fn log_story_type_summary(by_type: &std::collections::BTreeMap<String, u64>) -> String {
  let mut type_summary = String::new();
  for (t, n) in by_type {
    if !type_summary.is_empty() {
      type_summary.push_str(", ");
    }
    type_summary.push_str(&format!("{}:{n}", t));
  }
  if type_summary.is_empty() {
    type_summary.push_str("none");
  }
  type_summary
}

fn print_log_story_stats(theme: &Theme, ic: &icons::Icons, story: &LogStory<'_>, pretty: bool) {
  let type_summary = log_story_type_summary(story.by_type);
  let type_colored = if pretty {
    theme.accent(&type_summary)
  } else {
    type_summary
  };
  let pr_str = story
    .pr
    .map(|n| format!("PR #{n}"))
    .unwrap_or_else(|| "no PR".to_string());
  let pr_colored = if pretty { theme.warn(&pr_str) } else { pr_str };
  println!(
    "  {} {} commit{} · {} file{} · {}{} {}{}",
    ic.bullet,
    story.total,
    if story.total == 1 { "" } else { "s" },
    story.files_changed,
    if story.files_changed == 1 { "" } else { "s" },
    theme.insertion(&format!("+{}", story.net_added)),
    theme.deletion(&format!("-{}", story.net_removed)),
    type_colored,
    theme.dim("·")
  );
  println!(
    "  ↳ {pr_colored} {first_subject}",
    first_subject = story.first_subject
  );
}

pub fn print_log_story(story: &LogStory<'_>, mode: OutputMode) {
  let pretty = mode.is_pretty();
  let theme = Theme::detect_with(!pretty);
  let ic = icons::detect(!pretty);
  let term = layout::term_width();
  let rule = layout::horizontal_rule(term);

  println!("{rule}");
  println!(
    "  {} {} {} {}",
    ic.current,
    theme.branch(story.branch),
    theme.dim("→"),
    theme.branch(story.base)
  );
  print_log_story_stats(&theme, &ic, story, pretty);
  println!("{rule}");
}

fn print_show_header(theme: &Theme, ic: &icons::Icons, record: &CommitRecord, pretty: bool) {
  let type_colored = type_tag_colored(theme, ic, record.conventional_type.as_deref(), pretty);
  println!(
    "  {} {} {}",
    type_colored,
    theme.dim(&record.short_sha),
    theme.dim("·")
  );
  println!("  {}", theme.branch(&record.subject));
  println!(
    "  {}: {} <{}>     {}",
    theme.dim("Author"),
    record.author_name,
    theme.dim(&record.author_email),
    theme.dim(&record.time_relative)
  );
}

fn print_show_file_row(
  theme: &Theme,
  f: &FileStat,
  summary: &DiffSummary,
  term: usize,
  pretty: bool,
) {
  let bar = layout::bar(f.added + f.removed, summary.added + summary.removed, 16);
  let added = if pretty {
    theme.insertion(&format!("+{}", f.added))
  } else {
    format!("+{}", f.added)
  };
  let removed = if pretty {
    theme.deletion(&format!("-{}", f.removed))
  } else {
    format!("-{}", f.removed)
  };
  println!(
    "  {} {} {} {}",
    bar,
    layout::pad_right(&f.path, term.saturating_sub(50)),
    added,
    removed
  );
}

pub fn print_show(
  record: &CommitRecord,
  files: &[crate::git::diff::FileStat],
  summary: &DiffSummary,
  mode: OutputMode,
) {
  let pretty = mode.is_pretty();
  let theme = Theme::detect_with(!pretty);
  let ic = icons::detect(!pretty);
  let term = layout::term_width();
  let rule = layout::horizontal_rule(term);

  println!("{rule}");
  print_show_header(&theme, &ic, record, pretty);
  println!("{rule}");

  for f in files {
    print_show_file_row(&theme, f, summary, term, pretty);
  }
  println!("{rule}");
}

pub fn print_reflog(entries: &[ReflogEntry], mode: crate::output::OutputMode) {
  let pretty = mode.is_pretty();
  let theme = Theme::detect_with(!pretty);
  let term = layout::term_width();
  let rule = layout::horizontal_rule(term);

  if entries.is_empty() {
    println!("{rule}");
    println!("{}", theme.dim("reflog is empty"));
    println!("{rule}");
    return;
  }

  println!("{rule}");
  println!("{}", theme.dim("reflog"));
  println!("{rule}");

  let sha_w = 9;
  let ref_w = 9;
  let time_w = 14;

  for e in entries {
    let sha = if pretty {
      theme.dim(&e.sha)
    } else {
      e.sha.clone()
    };
    let refsel = if pretty {
      theme.accent(&e.ref_selector)
    } else {
      e.ref_selector.clone()
    };
    let time = if pretty {
      theme.dim(&e.time)
    } else {
      e.time.clone()
    };
    println!(
      "  {} {} {} {}",
      layout::pad_right(&sha, sha_w),
      layout::pad_right(&refsel, ref_w),
      layout::pad_left(&time, time_w),
      e.action
    );
  }
  println!("{rule}");
}
