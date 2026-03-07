use serde::{Deserialize, Serialize};

use super::{RuleDecision, RuleDefinition, RuleIdentity, RuleMode, RulePolicy};
use crate::domain::{DomainError, common::RuleId};

/// The central domain aggregate representing a fraud detection rule.
///
/// `Rule` acts as the primary coordinator for the engine's core components. It
/// integrates business metadata, execution eligibility, evaluation logic, and
/// deterministic outcomes into a single, cohesive unit.
///
/// This structure enforces strict validation boundaries; it is impossible to
/// instantiate or mutate a `Rule` into an invalid operational state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rule {
  /// System-level unique identifier for storage and referencing.
  pub id: RuleId,

  /// Descriptive identity, tracking codes, and organizational metadata.
  identity: RuleIdentity,

  /// Operational controls governing execution eligibility (lifecycle, rollout, schedule).
  policy: RulePolicy,

  /// The logical criteria evaluated against incoming event payloads.
  definition: RuleDefinition,

  /// The prescribed actions and risk impact yielded upon a positive match.
  outcome: RuleDecision,
}

impl Rule {
  /// Creates a new, validated `Rule` aggregate.
  ///
  /// This constructor serves as the final validation boundary for the rule's
  /// constituent components.
  ///
  /// # Errors
  ///
  /// Returns a [`DomainError`] if the provided [`RulePolicy`] or [`RuleDefinition`]
  /// violate engine constraints.
  pub fn new(
    id: RuleId,
    identity: RuleIdentity,
    policy: RulePolicy,
    definition: RuleDefinition,
    outcome: RuleDecision,
  ) -> Result<Self, DomainError> {
    policy.validate()?;
    definition.validate()?;

    Ok(Self { id, identity, policy, definition, outcome })
  }

  /// Evaluates whether the rule is eligible for execution.
  ///
  /// This is a high-level guard that checks the internal [`RulePolicy`] against
  /// the current system time and the event's deterministic traffic bucket.
  pub fn is_executable(&self, now_ms: u64, bucket_0_99: u8) -> bool {
    self.policy.can_execute(now_ms, bucket_0_99)
  }

  /// Transitions the rule's operational mode.
  ///
  /// # Errors
  ///
  /// Returns a [`DomainError`] if the requested transition violates the
  /// state machine rules defined in [`RuleMode`].
  pub fn transition_to(&mut self, mode: RuleMode) -> Result<(), DomainError> {
    self.policy.transition_to(mode)?;
    Ok(())
  }

  /// Updates the rule's policy after performing validation.
  ///
  /// This method ensures that policy updates (e.g., changes in rollout or schedule)
  /// satisfy all domain invariants before being applied.
  pub fn set_policy(&mut self, policy: RulePolicy) -> Result<(), DomainError> {
    policy.validate()?;
    self.policy = policy;
    Ok(())
  }

  /// Returns a reference to the rule's identity metadata.
  pub fn identity(&self) -> &RuleIdentity {
    &self.identity
  }

  /// Returns a reference to the rule's operational policy.
  pub fn policy(&self) -> &RulePolicy {
    &self.policy
  }

  /// Returns a reference to the rule's logical definition.
  pub fn definition(&self) -> &RuleDefinition {
    &self.definition
  }

  /// Returns a reference to the rule's decision output.
  pub fn outcome(&self) -> &RuleDecision {
    &self.outcome
  }

  /// Returns a reference to the underlying operational state.
  pub fn state(&self) -> &super::RuleState {
    &self.policy.state
  }

  /// Returns a reference to the rule's temporal schedule.
  pub fn schedule(&self) -> &super::RuleSchedule {
    &self.policy.schedule
  }

  /// Returns a reference to the rule's rollout configuration.
  pub fn rollout(&self) -> &super::RolloutPolicy {
    &self.policy.rollout
  }

  /// Returns a reference to the bipartite evaluation logic.
  pub fn evaluation(&self) -> &crate::domain::rule::RuleEvaluation {
    &self.definition.evaluation
  }

  /// Returns a reference to the prescribed enforcement actions.
  pub fn enforcement(&self) -> &crate::domain::rule::RuleEnforcement {
    &self.outcome.enforcement
  }

  /// Returns `true` if the rule is currently in the [`RuleMode::Active`] state.
  pub fn is_active_mode(&self) -> bool {
    self.policy.is_active_mode()
  }
}
