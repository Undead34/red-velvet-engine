use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{RolloutPolicy, RuleMode, RuleSchedule, RuleState};

/// The error type returned when a rule's operational policy is invalid or violates constraints.
#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize, Deserialize)]
pub enum RulePolicyError {
  /// Operational state is invalid or a requested transition is forbidden.
  #[error("invalid rule state: {0}")]
  State(#[from] super::state::RuleStateError),

  /// Temporal execution windows are invalid (`active_until_ms <= active_from_ms`).
  #[error("invalid rule schedule: {0}")]
  Schedule(#[from] super::schedule::RuleScheduleError),

  /// Rollout percentage is outside the valid `[0, 100]` range.
  #[error("invalid rule rollout: {0}")]
  Rollout(#[from] super::rollout::RuleRolloutError),
}

/// Execution constraints and lifecycle configuration for a rule.
///
/// `RulePolicy` is the policy boundary for `Rule` execution.
/// It is the single source of truth for:
/// - lifecycle state (`state`)
/// - temporal window (`schedule`)
/// - traffic gating (`rollout`).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RulePolicy {
  /// The mutable operational state, including its current mode and audit history.
  pub state: RuleState,

  /// The temporal boundaries dictating when the rule is actively processing events.
  pub schedule: RuleSchedule,

  /// The percentage-based traffic allocation for gradual deployment.
  pub rollout: RolloutPolicy,
}

impl RulePolicy {
  /// Constructs a new `RulePolicy` and validates its components.
  ///
  /// Returns a [`RulePolicyError`] if the state, schedule, or rollout constraints are invalid.
  pub fn new(
    state: RuleState,
    schedule: RuleSchedule,
    rollout: RolloutPolicy,
  ) -> Result<Self, RulePolicyError> {
    let policy = Self { state, schedule, rollout };
    policy.validate()?;
    Ok(policy)
  }

  /// Validates the internal state, schedule, and rollout configurations.
  pub fn validate(&self) -> Result<(), RulePolicyError> {
    self.state.validate().map_err(RulePolicyError::State)?;
    self.schedule.validate().map_err(RulePolicyError::Schedule)?;
    self.rollout.validate().map_err(RulePolicyError::Rollout)?;
    Ok(())
  }

  /// Returns a reference to the rule's operational state.
  pub fn state(&self) -> &RuleState {
    &self.state
  }

  /// Returns a reference to the rule's execution schedule.
  pub fn schedule(&self) -> &RuleSchedule {
    &self.schedule
  }

  /// Returns a reference to the rule's rollout policy.
  pub fn rollout(&self) -> &RolloutPolicy {
    &self.rollout
  }

  /// Determines if the rule is strictly eligible for evaluation.
  ///
  /// A rule is executable only if its state permits execution, the `now_ms` timestamp falls
  /// within its scheduled window, and the `bucket_0_99` value satisfies the rollout threshold.
  pub fn can_execute(&self, now_ms: u64, bucket_0_99: u8) -> bool {
    self.state.is_executable() && self.schedule.allows(now_ms) && self.rollout.allows(bucket_0_99)
  }

  /// Mutates the rule's operational mode.
  ///
  /// Returns a [`RulePolicyError`] if the state transition is invalid.
  pub fn transition_to(&mut self, mode: RuleMode) -> Result<(), RulePolicyError> {
    self.state.transition_to(mode).map_err(RulePolicyError::from)
  }

  /// Returns `true` if the rule is currently in the active mode.
  pub fn is_active_mode(&self) -> bool {
    self.state.mode == RuleMode::Active
  }
}
