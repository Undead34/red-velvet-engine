use serde::{Deserialize, Serialize};

use super::RuleEvaluation;
use crate::domain::DomainError;

/// The logical core and evaluation criteria of a fraud rule.
///
/// `RuleDefinition` encapsulates the "If" component of a rule's logic. It houses
/// the specific expressions and conditions that the engine evaluates against
/// incoming event payloads to determine if a rule should trigger.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleDefinition {
  /// The bipartite logic (condition and main expression) to be evaluated.
  pub evaluation: RuleEvaluation,
}

impl RuleDefinition {
  /// Creates a new `RuleDefinition` after validating its evaluation logic.
  ///
  /// # Errors
  ///
  /// Returns a [`DomainError`] if the underlying evaluation expressions are
  /// syntactically invalid or violate engine constraints (e.g., restricted variables).
  pub fn new(evaluation: RuleEvaluation) -> Result<Self, DomainError> {
    evaluation.validate()?;
    Ok(Self { evaluation })
  }

  /// Validates the integrity of the internal evaluation logic.
  ///
  /// This ensures that both the guard condition and the main logic are
  /// executable by the rule engine.
  pub fn validate(&self) -> Result<(), DomainError> {
    self.evaluation.validate()
  }

  /// Returns a reference to the internal [`RuleEvaluation`].
  pub fn evaluation(&self) -> &RuleEvaluation {
    &self.evaluation
  }
}
