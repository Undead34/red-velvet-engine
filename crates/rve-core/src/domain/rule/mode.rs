use serde::{Deserialize, Serialize};

/// The operational lifecycle mode of a rule.
///
/// `RuleMode` dictates a rule's eligibility for evaluation by the engine and
/// governs its allowable state transitions within the system.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleMode {
  /// A draft state that strictly bypasses production evaluation.
  Staged,

  /// An operational state that actively evaluates eligible traffic.
  Active,

  /// A paused state temporarily bypassed by the engine.
  Suspended,

  /// A terminal, retired state retained exclusively for historical auditing.
  Deactivated,
}

impl Default for RuleMode {
  /// Returns the default mode ([`RuleMode::Staged`]) to prevent accidental live activation.
  fn default() -> Self {
    RuleMode::Staged
  }
}

impl RuleMode {
  /// Returns `true` if the mode permits active rule evaluation.
  pub fn is_executable(&self) -> bool {
    matches!(self, Self::Active)
  }

  /// Returns `true` if the state machine permits transitioning to the `next` mode.
  ///
  /// The lifecycle enforces the following directed graph:
  ///
  /// * **`Staged`**: Can transition to any state.
  /// * **`Active`**: Can transition to `Suspended` (pause) or `Deactivated` (retire).
  ///   Cannot revert to `Staged`.
  /// * **`Suspended`**: Can return to `Active` (resume), revert to `Staged` (re-draft),
  ///   or transition to `Deactivated`.
  /// * **`Deactivated`**: A terminal state. Cannot transition to any other mode.
  ///
  /// Self-transitions (e.g., `Active` to `Active`) are always permitted.
  pub fn can_transition_to(&self, next: Self) -> bool {
    match (self, next) {
      (Self::Staged, Self::Staged)
      | (Self::Staged, Self::Active)
      | (Self::Staged, Self::Suspended)
      | (Self::Staged, Self::Deactivated)
      | (Self::Active, Self::Active)
      | (Self::Active, Self::Suspended)
      | (Self::Active, Self::Deactivated)
      | (Self::Suspended, Self::Staged)
      | (Self::Suspended, Self::Active)
      | (Self::Suspended, Self::Suspended)
      | (Self::Suspended, Self::Deactivated)
      | (Self::Deactivated, Self::Deactivated) => true,
      _ => false,
    }
  }

  /// Returns `true` if the mode represents a final, irreversible state.
  pub fn is_terminal(&self) -> bool {
    matches!(self, Self::Deactivated)
  }

  /// Returns `true` if the mode represents a stable resting state.
  pub fn is_stable(&self) -> bool {
    matches!(self, Self::Active | Self::Staged | Self::Suspended)
  }

  /// Returns `true` if the mode represents an active or suspended operational state.
  pub fn is_mutating(&self) -> bool {
    matches!(self, Self::Active | Self::Suspended)
  }
}
