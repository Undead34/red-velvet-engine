use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{RuleAudit, mode::RuleMode};

/// The error type returned when rule state operations or constraints fail.
#[derive(Debug, Clone, Serialize, Deserialize, Error, PartialEq, Eq)]
pub enum RuleStateError {
  /// The underlying audit trail failed validation.
  #[error("invalid rule audit: {0}")]
  Audit(#[from] super::audit::RuleAuditError),

  /// The requested lifecycle mode transition violates the state machine's rules.
  #[error("invalid rule state transition: cannot transition from {:?} to {:?}", from, to)]
  InvalidTransition {
    /// The current operational mode.
    from: RuleMode,
    /// The rejected target mode.
    to: RuleMode,
  },
}

/// The operational state and modification history of a rule.
///
/// `RuleState` encapsulates the current execution mode of the rule alongside the
/// audit trail of its lifecycle changes. It governs valid state transitions and
/// exposes the rule's fundamental eligibility for evaluation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleState {
  /// The current lifecycle mode dictating execution behavior.
  pub mode: RuleMode,
  /// The chronological record of creation and modification events.
  pub audit: RuleAudit,
}

impl RuleState {
  /// Creates a new `RuleState` and validates its components.
  ///
  /// Returns [`RuleStateError`] when the embedded audit trail is invalid.
  pub fn new(mode: RuleMode, audit: RuleAudit) -> Result<Self, RuleStateError> {
    let state = Self { mode, audit };
    state.validate()?;
    Ok(state)
  }

  /// Validates the internal audit trail.
  pub fn validate(&self) -> Result<(), RuleStateError> {
    self.audit.validate().map_err(RuleStateError::Audit)
  }

  /// Returns `true` when the operational mode allows evaluation.
  ///
  /// Current implementation delegates to [`RuleMode::is_executable`].
  pub fn is_executable(&self) -> bool {
    self.mode.is_executable()
  }

  /// Returns `true` when [`RuleMode::can_transition_to`] admits `next`.
  pub fn can_transition_to(&self, next: RuleMode) -> bool {
    self.mode.can_transition_to(next)
  }

  /// Returns `true` when the current mode is terminal.
  /// Terminal mode implies the rule cannot be reactivated in normal operation.
  pub fn is_terminal(&self) -> bool {
    self.mode.is_terminal()
  }

  /// Returns `true` when the current mode is stable (`active`, `staged`, `suspended`).
  pub fn is_stable(&self) -> bool {
    self.mode.is_stable()
  }

  /// Applies a state transition atomically.
  ///
  /// Returns a [`RuleStateError::InvalidTransition`] if the requested transition
  /// violates the defined state machine rules in [`RuleMode::can_transition_to`].
  pub fn transition_to(&mut self, next: RuleMode) -> Result<(), RuleStateError> {
    if !self.can_transition_to(next) {
      return Err(RuleStateError::InvalidTransition { from: self.mode, to: next });
    }

    self.mode = next;
    Ok(())
  }
}
