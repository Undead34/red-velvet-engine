use serde::{Deserialize, Serialize};

use super::{RuleAudit, mode::RuleMode};

/// Mutable operational state of a rule.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleState {
  pub mode: RuleMode,
  pub audit: RuleAudit,
}
