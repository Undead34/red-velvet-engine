use serde::{Deserialize, Serialize};

use super::expression::RuleExpression;
use crate::domain::DomainResult;

/// Pair of expressions used to evaluate a rule.
///
/// `condition` and `logic` are both validated expressions.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleEvaluation {
  /// Guard expression.
  pub condition: RuleExpression,

  /// Main logic expression.
  pub logic: RuleExpression,
}

impl RuleEvaluation {
  /// Creates an evaluation and validates variable roots.
  ///
  /// # Errors
  ///
  /// Returns [`DomainError`] if either expression has a disallowed `var` root.
  pub fn new(condition: RuleExpression, logic: RuleExpression) -> DomainResult<Self> {
    let this = Self { condition, logic };
    this.validate()?;
    Ok(this)
  }

  /// Validates variable roots for both expressions.
  ///
  /// # Errors
  ///
  /// Returns [`DomainError`] if either expression has a disallowed `var` root.
  pub fn validate(&self) -> DomainResult<()> {
    self.condition.validate_vars()?;
    self.logic.validate_vars()?;
    Ok(())
  }

  /// Consumes the value and returns `(condition, logic)`.
  pub fn into_parts(self) -> (RuleExpression, RuleExpression) {
    (self.condition, self.logic)
  }
}
