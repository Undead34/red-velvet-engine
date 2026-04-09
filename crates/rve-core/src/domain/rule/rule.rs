use serde::{Deserialize, Serialize};

use super::{RuleDecision, RuleDefinition, RuleIdentity, RuleMode, RulePolicy, RuleScope};
use crate::domain::{DomainResult, common::RuleId};

/// A fraud rule aggregate.
///
/// A rule combines identity metadata, execution policy, logic definition, and
/// per-rule outcome.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rule {
  /// System-level unique identifier for storage and referencing.
  pub id: RuleId,

  /// Descriptive identity, tracking codes, and organizational metadata.
  identity: RuleIdentity,

  /// Channel-based applicability scope.
  #[serde(default)]
  scope: RuleScope,

  /// Operational controls governing execution eligibility (lifecycle, rollout, schedule).
  policy: RulePolicy,

  /// The logical criteria evaluated against incoming event payloads.
  definition: RuleDefinition,

  /// The prescribed actions and risk impact yielded upon a positive match.
  outcome: RuleDecision,
}

impl Rule {
  /// Creates a rule and validates aggregate invariants.
  ///
  /// # Errors
  ///
  /// Returns [`DomainResult`]::Err if `policy` or `definition` is invalid.
  pub fn new(
    id: RuleId,
    identity: RuleIdentity,
    scope: RuleScope,
    policy: RulePolicy,
    definition: RuleDefinition,
    outcome: RuleDecision,
  ) -> DomainResult<Self> {
    scope.validate()?;
    policy.validate()?;
    definition.validate()?;

    Ok(Self { id, identity, scope, policy, definition, outcome })
  }

  /// Returns `true` if this rule is executable for the given time and bucket.
  pub fn is_executable(&self, now_ms: u64, bucket_0_99: u8) -> bool {
    self.policy.can_execute(now_ms, bucket_0_99)
  }

  /// Returns `true` when this rule can apply to the provided event channel.
  pub fn applies_to_channel(&self, channel: Option<&crate::domain::common::Channel>) -> bool {
    self.scope.applies_to(channel)
  }

  /// Returns `true` when both policy and channel scope admit execution.
  pub fn is_executable_for_channel(
    &self,
    now_ms: u64,
    bucket_0_99: u8,
    channel: Option<&crate::domain::common::Channel>,
  ) -> bool {
    self.is_executable(now_ms, bucket_0_99) && self.applies_to_channel(channel)
  }

  /// Transitions the rule mode.
  ///
  /// # Errors
  ///
  /// Returns [`DomainResult`]::Err if the transition is not allowed.
  pub fn transition_to(&mut self, mode: RuleMode) -> DomainResult<()> {
    self.policy.transition_to(mode)?;
    Ok(())
  }

  /// Replaces the policy after validation.
  ///
  /// # Errors
  ///
  /// Returns [`DomainResult`]::Err if `policy` is invalid.
  pub fn set_policy(&mut self, policy: RulePolicy) -> DomainResult<()> {
    policy.validate()?;
    self.policy = policy;
    Ok(())
  }

  /// Returns the identity metadata.
  pub fn identity(&self) -> &RuleIdentity {
    &self.identity
  }

  /// Returns the execution policy.
  pub fn policy(&self) -> &RulePolicy {
    &self.policy
  }

  /// Returns the applicability scope.
  pub fn scope(&self) -> &RuleScope {
    &self.scope
  }

  /// Returns the rule definition.
  pub fn definition(&self) -> &RuleDefinition {
    &self.definition
  }

  /// Returns the per-rule outcome.
  pub fn outcome(&self) -> &RuleDecision {
    &self.outcome
  }

  /// Returns the current state.
  pub fn state(&self) -> &super::RuleState {
    &self.policy.state
  }

  /// Returns the schedule window.
  pub fn schedule(&self) -> &super::RuleSchedule {
    &self.policy.schedule
  }

  /// Returns the rollout policy.
  pub fn rollout(&self) -> &super::RolloutPolicy {
    &self.policy.rollout
  }

  /// Returns the evaluation logic.
  pub fn evaluation(&self) -> &crate::domain::rule::RuleEvaluation {
    &self.definition.evaluation
  }

  /// Returns the enforcement settings.
  pub fn enforcement(&self) -> &crate::domain::rule::RuleEnforcement {
    &self.outcome.enforcement
  }

  /// Returns `true` if the rule is currently in the [`RuleMode::Active`] state.
  pub fn is_active_mode(&self) -> bool {
    self.policy.is_active_mode()
  }
}
