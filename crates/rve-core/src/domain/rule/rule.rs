use serde::{Deserialize, Serialize};

use crate::domain::{DomainError, common::RuleId};

use super::{RuleDecision, RuleDefinition, RuleIdentity, RuleMode, RulePolicy};

/// A fraud detection rule.
///
/// `Rule` acts as the coordinator for the engine's core components. It ties
/// together business metadata, execution constraints, evaluation logic, and
/// the resulting system actions into a single, cohesive unit.
/// The rule's constructor enforces validation boundaries on policy and
/// definition so invalid rules cannot be instantiated.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rule {
  /// System-level unique identifier for storage and referencing.
  pub id: RuleId,

  /// Human-readable identity, tracking codes, and organizational tags.
  identity: RuleIdentity,

  /// Deployment controls dictating execution eligibility (state, rollout, schedule).
  policy: RulePolicy,

  /// The logical expression evaluated against the incoming event payload.
  definition: RuleDefinition,

  /// The actions and risk scoring applied upon a positive evaluation.
  outcome: RuleDecision,
}

impl Rule {
  /// Creates a validated rule aggregate.
  ///
  /// This is the rule-level constructor and the last line of defence for
  /// invalid policy or definition payloads.
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

  /// Returns `true` when the policy allows execution at `now_ms` and for bucket.
  ///
  /// This method is the aggregate guard used by the engine before evaluating
  /// the rule definition.
  pub fn is_executable(&self, now_ms: u64, bucket_0_99: u8) -> bool {
    self.policy.can_execute(now_ms, bucket_0_99)
  }

  /// Moves the rule lifecycle mode forward/backward according to `RuleMode` rules.
  ///
  /// Domain errors are returned as `DomainError`; in practice this is backed by
  /// `RulePolicyError` when transition constraints fail.
  pub fn transition_to(&mut self, mode: RuleMode) -> Result<(), DomainError> {
    self.policy.transition_to(mode)?;
    Ok(())
  }

  /// Replaces the current policy after validation.
  ///
  /// This method is intentionally explicit so policy changes are always
  /// validated at the aggregate boundary.
  pub fn set_policy(&mut self, policy: RulePolicy) -> Result<(), DomainError> {
    policy.validate()?;
    self.policy = policy;
    Ok(())
  }

  pub fn identity(&self) -> &RuleIdentity {
    &self.identity
  }

  pub fn policy(&self) -> &RulePolicy {
    &self.policy
  }

  pub fn definition(&self) -> &RuleDefinition {
    &self.definition
  }

  pub fn outcome(&self) -> &RuleDecision {
    &self.outcome
  }

  pub fn state(&self) -> &super::RuleState {
    &self.policy.state
  }

  pub fn schedule(&self) -> &super::RuleSchedule {
    &self.policy.schedule
  }

  pub fn rollout(&self) -> &super::RolloutPolicy {
    &self.policy.rollout
  }

  pub fn evaluation(&self) -> &crate::domain::rule::RuleEvaluation {
    &self.definition.evaluation
  }

  pub fn enforcement(&self) -> &crate::domain::rule::RuleEnforcement {
    &self.outcome.enforcement
  }

  pub fn is_active_mode(&self) -> bool {
    self.policy.is_active_mode()
  }
}
