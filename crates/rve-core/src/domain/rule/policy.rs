use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{RolloutPolicy, RuleMode, RuleSchedule, RuleState};

/// Errors that can occur when validating or mutating [`RulePolicy`].
#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize, Deserialize)]
pub enum RulePolicyError {
  /// State is invalid or a requested transition is forbidden.
  #[error("invalid rule state: {0}")]
  State(#[from] super::state::RuleStateError),

  /// Schedule boundaries are invalid.
  #[error("invalid rule schedule: {0}")]
  Schedule(#[from] super::schedule::RuleScheduleError),

  /// Rollout percentage is outside `0..=100`.
  #[error("invalid rule rollout: {0}")]
  Rollout(#[from] super::rollout::RuleRolloutError),
}

/// Execution controls for a rule.
///
/// A policy combines lifecycle state, schedule window, and rollout percentage.
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
  /// Creates a policy and validates it.
  ///
  /// # Errors
  ///
  /// Returns [`RulePolicyError`] if any component is invalid.
  pub fn new(
    state: RuleState,
    schedule: RuleSchedule,
    rollout: RolloutPolicy,
  ) -> Result<Self, RulePolicyError> {
    let policy = Self { state, schedule, rollout };
    policy.validate()?;
    Ok(policy)
  }

  /// Validates all policy components.
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

  /// Returns `true` if the rule can execute at `now_ms` for `bucket_0_99`.
  pub fn can_execute(&self, now_ms: u64, bucket_0_99: u8) -> bool {
    self.state.is_executable() && self.schedule.allows(now_ms) && self.rollout.allows(bucket_0_99)
  }

  /// Transitions the policy mode.
  ///
  /// # Errors
  ///
  /// Returns [`RulePolicyError::State`] if the transition is invalid.
  pub fn transition_to(&mut self, mode: RuleMode) -> Result<(), RulePolicyError> {
    self.state.transition_to(mode).map_err(RulePolicyError::from)
  }

  /// Returns `true` if the rule is currently in the active mode.
  pub fn is_active_mode(&self) -> bool {
    self.state.mode == RuleMode::Active
  }
}
