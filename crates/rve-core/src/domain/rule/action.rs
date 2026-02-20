use serde::{Deserialize, Serialize};

/// Action suggested when a rule is triggered.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleAction {
  Allow,
  Review,
  Block,
  TagOnly,
}
