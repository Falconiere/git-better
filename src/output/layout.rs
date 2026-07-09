use terminal_size::{Width, terminal_size};
use unicode_width::UnicodeWidthChar;

pub fn term_width() -> usize {
  if let Some((Width(w), _)) = terminal_size() {
    w as usize
  } else {
    80
  }
}

fn visible_width(s: &str) -> usize {
  let mut width = 0usize;
  let mut chars = s.chars().peekable();
  while let Some(c) = chars.next() {
    if c == '\u{1b}' {
      if matches!(chars.peek(), Some('[')) {
        chars.next();
        for esc in chars.by_ref() {
          if matches!(esc, '@'..='~') {
            break;
          }
        }
        continue;
      }
      continue;
    }
    width += UnicodeWidthChar::width(c).unwrap_or(0);
  }
  width
}

pub fn pad_right(s: &str, width: usize) -> String {
  let w = visible_width(s);
  if w >= width {
    s.to_string()
  } else {
    format!("{}{}", s, " ".repeat(width - w))
  }
}

pub fn pad_left(s: &str, width: usize) -> String {
  let w = visible_width(s);
  if w >= width {
    s.to_string()
  } else {
    format!("{}{}", " ".repeat(width - w), s)
  }
}

pub fn horizontal_rule(width: usize) -> String {
  "─".repeat(width.min(term_width()))
}

pub fn bar(value: u64, max: u64, width: usize) -> String {
  if max == 0 {
    return " ".repeat(width);
  }
  let filled = (value as f64 / max as f64 * width as f64).round() as usize;
  let filled = filled.min(width);
  format!("{}{}", "▌".repeat(filled), " ".repeat(width - filled))
}
