use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Copy, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Flag {
  #[default]
  Unknown,
  No,
  Yes,
}

impl Flag {
  pub fn is_yes(self) -> bool {
    matches!(self, Flag::Yes)
  }
}
