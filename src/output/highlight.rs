use std::sync::LazyLock;

use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::{SyntaxReference, SyntaxSet};
use syntect::util::as_24_bit_terminal_escaped;

pub static SYNTAX_SET: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);
pub static THEME_SET: LazyLock<ThemeSet> = LazyLock::new(ThemeSet::load_defaults);
static FALLBACK_THEME: LazyLock<Theme> = LazyLock::new(Theme::default);

pub fn syntax_for_extension(extension: &str) -> Option<&'static SyntaxReference> {
  SYNTAX_SET
    .find_syntax_by_extension(extension)
    .or_else(|| SYNTAX_SET.find_syntax_by_name(extension))
}

pub fn syntax_for_filename(filename: &str) -> Option<&'static SyntaxReference> {
  let path = std::path::Path::new(filename);
  SYNTAX_SET
    .find_syntax_by_path(path.to_str().unwrap_or(""))
    .or_else(|| {
      path
        .extension()
        .and_then(|e| e.to_str())
        .and_then(syntax_for_extension)
    })
}

pub fn default_theme() -> &'static Theme {
  THEME_SET
    .themes
    .get("base16-eighties.dark")
    .or_else(|| THEME_SET.themes.values().next())
    .unwrap_or(&FALLBACK_THEME)
}

pub fn light_theme() -> Option<&'static Theme> {
  THEME_SET.themes.get("InspiredGitHub")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineBg {
  None,
  Context,
  Insertion,
  Deletion,
  Header,
}

pub fn make_highlighter(
  syntax: Option<&'static SyntaxReference>,
) -> Option<HighlightLines<'static>> {
  syntax.map(|s| HighlightLines::new(s, default_theme()))
}

fn highlight_body(highlighter: Option<&mut HighlightLines<'static>>, content: &str) -> String {
  match highlighter {
    Some(h) => match h.highlight_line(content, &SYNTAX_SET) {
      Ok(ranges) => as_24_bit_terminal_escaped(&ranges, false),
      Err(_) => content.to_string(),
    },
    None => content.to_string(),
  }
}

pub fn render_diff_line(
  highlighter: Option<&mut HighlightLines<'static>>,
  content: &str,
  bg: LineBg,
  theme_enabled: bool,
) -> String {
  let sign = match bg {
    LineBg::Insertion => '+',
    LineBg::Deletion => '-',
    LineBg::Context | LineBg::None => ' ',
    LineBg::Header => '@',
  };

  if !theme_enabled {
    return format!("{sign}{content}\n");
  }

  if matches!(bg, LineBg::Header) {
    return format!("{sign}\u{1b}[1;97m{content}\u{1b}[0m\n");
  }

  let bg_ansi = match bg {
    LineBg::Insertion => "\u{1b}[48;2;0;50;0m",
    LineBg::Deletion => "\u{1b}[48;2;50;0;0m",
    LineBg::Context => "\u{1b}[48;2;30;30;30m",
    LineBg::Header => "\u{1b}[48;2;30;30;60m",
    LineBg::None => "",
  };

  let body = highlight_body(highlighter, content);

  let mut out = String::with_capacity(content.len() + 32);
  out.push(sign);
  if !bg_ansi.is_empty() {
    out.push_str(bg_ansi);
  }
  out.push_str(&body);
  out.push_str("\u{1b}[0m");
  out.push('\n');
  out
}

pub fn render_hunk_header(header: &str, theme_enabled: bool) -> String {
  if !theme_enabled {
    return format!("{header}\n");
  }
  format!("\u{1b}[38;2;130;130;200m{header}\u{1b}[0m\n")
}

pub fn render_file_header(old_path: &str, new_path: &str, theme_enabled: bool) -> String {
  if !theme_enabled {
    return format!("--- a/{old_path}\n+++ b/{new_path}\n");
  }
  format!(
    "\u{1b}[1;96m--- a/{}\u{1b}[0m\n\u{1b}[1;96m+++ b/{}\u{1b}[0m\n",
    old_path, new_path
  )
}
