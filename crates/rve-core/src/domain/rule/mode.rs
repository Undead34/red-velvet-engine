use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleMode {
  #[serde(alias = "draft")]
  Staged,
  #[serde(alias = "enabled")]
  Active,
  #[serde(alias = "paused")]
  Suspended,
  #[serde(alias = "disabled")]
  Deactivated,
}

impl Default for RuleMode {
  fn default() -> Self {
    RuleMode::Staged
  }
}
