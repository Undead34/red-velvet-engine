use serde::{Deserialize, Serialize};

use super::RuleEvaluation;
use crate::domain::DomainError;

/// Logical definition of a rule.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleDefinition {
  /// Expressions used for evaluation.
  pub evaluation: RuleEvaluation,
}

impl RuleDefinition {
  /// Creates a definition and validates it.
  ///
  /// # Errors
  ///
  /// Returns [`DomainError`] if evaluation is invalid.
  pub fn new(evaluation: RuleEvaluation) -> Result<Self, DomainError> {
    evaluation.validate()?;
    Ok(Self { evaluation })
  }

  /// Validates the contained evaluation.
  ///
  /// # Errors
  ///
  /// Returns [`DomainError`] if evaluation is invalid.
  pub fn validate(&self) -> Result<(), DomainError> {
    self.evaluation.validate()
  }

  /// Returns the inner evaluation.
  pub fn evaluation(&self) -> &RuleEvaluation {
    &self.evaluation
  }
}
