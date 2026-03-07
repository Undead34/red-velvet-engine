use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{RuleAudit, mode::RuleMode};

/// Errors that can occur when creating or mutating [`RuleState`].
#[derive(Debug, Clone, Serialize, Deserialize, Error, PartialEq, Eq)]
pub enum RuleStateError {
  /// The underlying audit trail failed validation.
  #[error("invalid rule audit: {0}")]
  Audit(#[from] super::audit::RuleAuditError),

  /// The requested lifecycle transition is not allowed.
  #[error("invalid rule state transition: cannot transition from {:?} to {:?}", from, to)]
  InvalidTransition {
    /// The current operational mode.
    from: RuleMode,
    /// The rejected target mode.
    to: RuleMode,
  },
}

/// Lifecycle state for a rule.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleState {
  /// The current lifecycle mode dictating execution behavior.
  pub mode: RuleMode,
  /// The chronological record of creation and modification events.
  pub audit: RuleAudit,
}

impl RuleState {
  /// Creates a new state and validates its audit metadata.
  ///
  /// # Errors
  ///
  /// Returns [`RuleStateError::Audit`] if `audit` is invalid.
  pub fn new(mode: RuleMode, audit: RuleAudit) -> Result<Self, RuleStateError> {
    let state = Self { mode, audit };
    state.validate()?;
    Ok(state)
  }

  /// Validates the state.
  ///
  /// # Errors
  ///
  /// Returns [`RuleStateError::Audit`] if the embedded audit is invalid.
  pub fn validate(&self) -> Result<(), RuleStateError> {
    self.audit.validate().map_err(RuleStateError::Audit)
  }

  /// Returns `true` if the current mode allows evaluation.
  pub fn is_executable(&self) -> bool {
    self.mode.is_executable()
  }

  /// Returns `true` when [`RuleMode::can_transition_to`] admits `next`.
  pub fn can_transition_to(&self, next: RuleMode) -> bool {
    self.mode.can_transition_to(next)
  }

  /// Returns `true` if the current mode is terminal.
  pub fn is_terminal(&self) -> bool {
    self.mode.is_terminal()
  }

  /// Returns `true` if the current mode is stable.
  pub fn is_stable(&self) -> bool {
    self.mode.is_stable()
  }

  /// Transitions to `next`.
  ///
  /// # Errors
  ///
  /// Returns [`RuleStateError::InvalidTransition`] if `next` is not allowed.
  pub fn transition_to(&mut self, next: RuleMode) -> Result<(), RuleStateError> {
    if !self.can_transition_to(next) {
      return Err(RuleStateError::InvalidTransition { from: self.mode, to: next });
    }

    self.mode = next;
    Ok(())
  }
}
