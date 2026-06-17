use terminal_size::{terminal_size, Width};
use unicode_width::UnicodeWidthStr;

pub fn term_width() -> usize {
    if let Some((Width(w), _)) = terminal_size() {
        w as usize
    } else {
        80
    }
}

pub fn pad_right(s: &str, width: usize) -> String {
    let w = UnicodeWidthStr::width(s);
    if w >= width {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(width - w))
    }
}

#[allow(dead_code)]
pub fn pad_left(s: &str, width: usize) -> String {
    let w = UnicodeWidthStr::width(s);
    if w >= width {
        s.to_string()
    } else {
        format!("{}{}", " ".repeat(width - w), s)
    }
}

pub fn horizontal_rule(width: usize) -> String {
    "─".repeat(width.min(term_width()))
}

#[allow(dead_code)]
pub fn bar(value: u64, max: u64, width: usize) -> String {
    if max == 0 {
        return " ".repeat(width);
    }
    let filled = (value as f64 / max as f64 * width as f64).round() as usize;
    let filled = filled.min(width);
    format!("{}{}", "▌".repeat(filled), " ".repeat(width - filled))
}
