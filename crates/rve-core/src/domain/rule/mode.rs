use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleMode {
  Staged,
  Active,
  Suspended,
  Deactivated,
}

impl Default for RuleMode {
  fn default() -> Self {
    RuleMode::Staged
  }
}
