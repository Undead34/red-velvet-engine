use serde::{Deserialize, Serialize};

use super::expression::RuleExpression;
use crate::domain::DomainError;

/// The evaluation logic consisting of a guard condition and a primary logic expression.
///
/// `RuleEvaluation` defines a two-step execution flow designed for optimization.
/// The engine first evaluates the `condition`; if it yields `false`, the execution
/// short-circuits to avoid the computational cost of the `logic` expression.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleEvaluation {
  /// A lightweight guard expression evaluated as a pre-requisite.
  pub condition: RuleExpression,

  /// The primary domain logic executed only when the `condition` evaluates to `true`.
  pub logic: RuleExpression,
}

impl RuleEvaluation {
  /// Creates a new `RuleEvaluation` and performs a static validation of its expressions.
  ///
  /// # Errors
  ///
  /// Returns a [`DomainError`] if either expression contains illegal variables or
  /// violates engine-specific syntax constraints.
  pub fn new(condition: RuleExpression, logic: RuleExpression) -> Result<Self, DomainError> {
    let this = Self { condition, logic };
    this.validate()?;
    Ok(this)
  }

  /// Validates the variable access patterns for both the guard and the primary logic.
  ///
  /// This ensures all referenced data paths are within the engine's allowed
  /// root schemas (e.g., `event`, `payload`, `context`).
  pub fn validate(&self) -> Result<(), DomainError> {
    self.condition.validate_vars()?;
    self.logic.validate_vars()?;
    Ok(())
  }

  /// Consumes the evaluation, returning its constituent expressions.
  ///
  /// Returns a tuple containing `(condition, logic)`.
  pub fn into_parts(self) -> (RuleExpression, RuleExpression) {
    (self.condition, self.logic)
  }
}
