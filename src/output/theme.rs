use std::io::IsTerminal;

use owo_colors::OwoColorize;

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub enabled: bool,
}

impl Theme {
    pub fn detect() -> Self {
        Self {
            enabled: !should_disable_color(),
        }
    }

    pub fn branch(&self, s: &str) -> String {
        if self.enabled {
            s.bright_cyan().to_string()
        } else {
            s.to_string()
        }
    }

    pub fn insertion(&self, s: &str) -> String {
        if self.enabled {
            s.green().to_string()
        } else {
            s.to_string()
        }
    }

    pub fn deletion(&self, s: &str) -> String {
        if self.enabled {
            s.red().to_string()
        } else {
            s.to_string()
        }
    }

    pub fn dim(&self, s: &str) -> String {
        if self.enabled {
            s.dimmed().to_string()
        } else {
            s.to_string()
        }
    }

    pub fn warn(&self, s: &str) -> String {
        if self.enabled {
            s.yellow().to_string()
        } else {
            s.to_string()
        }
    }

    pub fn accent(&self, s: &str) -> String {
        if self.enabled {
            s.bright_magenta().to_string()
        } else {
            s.to_string()
        }
    }

    pub fn bold(&self, s: &str) -> String {
        if self.enabled {
            s.bold().to_string()
        } else {
            s.to_string()
        }
    }
}

fn should_disable_color() -> bool {
    if std::env::var_os("NO_COLOR").is_some() {
        return true;
    }
    if std::env::var_os("GB_ASCII").is_some() {
        return true;
    }
    if !std::io::stdout().is_terminal() {
        return true;
    }
    false
}
