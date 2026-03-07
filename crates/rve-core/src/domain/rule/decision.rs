use serde::{Deserialize, Serialize};

use super::RuleEnforcement;

/// Per-rule outcome used when a rule matches.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleDecision {
  /// Enforcement configuration for this rule.
  pub enforcement: RuleEnforcement,
}

impl RuleDecision {
  /// Creates a decision from enforcement settings.
  pub fn new(enforcement: RuleEnforcement) -> Self {
    Self { enforcement }
  }

  /// Returns the underlying enforcement settings.
  pub fn enforcement(&self) -> &RuleEnforcement {
    &self.enforcement
  }
}
