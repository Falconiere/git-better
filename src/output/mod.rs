use std::io::IsTerminal;

pub mod better;
pub mod highlight;
pub mod human;
pub mod icons;
pub mod layout;
pub mod theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
