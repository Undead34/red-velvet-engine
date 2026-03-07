use serde::{Deserialize, Serialize};

use super::RuleEnforcement;

/// The terminal output of a successful rule evaluation.
///
/// `RuleDecision` represents the final artifact produced by the engine when a rule's
/// criteria are met. It encapsulates the specific enforcement actions, risk scores,
/// and metadata that the consuming system must apply to the transaction.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleDecision {
  /// The prescribed enforcement parameters and risk impact.
  pub enforcement: RuleEnforcement,
}

impl RuleDecision {
  /// Creates a new `RuleDecision` with the specified enforcement.
  pub fn new(enforcement: RuleEnforcement) -> Self {
    Self { enforcement }
  }

  /// Returns a reference to the rule's [`RuleEnforcement`] parameters.
  ///
  /// This provides access to the concrete actions (e.g., block, allow) and
  /// scoring adjustments dictated by the rule.
  pub fn enforcement(&self) -> &RuleEnforcement {
    &self.enforcement
  }
}
