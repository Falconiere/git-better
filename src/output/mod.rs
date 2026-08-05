use std::io::IsTerminal;

/// JSON envelope serialization for `--better`.
pub mod better;
/// Convention-profile summary printer.
pub mod conventions_view;
/// Syntax highlighting via a shared syntect set.
pub mod highlight;
/// Pretty printers for the read commands.
pub mod human;
/// Unicode icons with an ASCII fallback.
pub mod icons;
/// Column padding, rules, and bar charts.
pub mod layout;
/// Color palette and color-enable detection.
pub mod theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// How a command should render: human text (pretty or flat) or a JSON envelope.
pub enum OutputMode {
  Human { pretty: bool },
  Better { budget: Option<usize> },
}

impl OutputMode {
  pub fn from_flags(plain: bool, better: bool) -> Self {
    if better {
      OutputMode::Better { budget: None }
    } else if plain || !std::io::stdout().is_terminal() {
      OutputMode::Human { pretty: false }
    } else {
      OutputMode::Human { pretty: true }
    }
  }

  pub fn is_better(&self) -> bool {
    matches!(self, OutputMode::Better { .. })
  }

  pub fn is_pretty(&self) -> bool {
    matches!(self, OutputMode::Human { pretty: true })
  }
}
