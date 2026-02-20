use serde::{Deserialize, Serialize};

/// Lifecycle mode of a rule in the decision engine.
///
/// This controls whether the rule can be evaluated and how operators should treat it.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleMode {
  /// Rule exists as a draft and is not evaluated in production traffic.
  ///
  /// Use this state while authoring or reviewing a rule before rollout.
  Staged,

  /// Rule is fully enabled and evaluated for eligible events.
  ///
  /// This is the normal operating mode for live fraud controls.
  Active,

  /// Rule is temporarily paused and skipped by the engine.
  ///
  /// Keep the rule definition intact while stopping its impact during investigation.
  Suspended,

  /// Rule is permanently retired and should not be reactivated.
  ///
  /// Prefer this over deletion when you need auditability of historical rules.
  Deactivated,
}

impl Default for RuleMode {
  /// New rules start in `Staged` by default to prevent accidental live activation.
  fn default() -> Self {
    RuleMode::Staged
  }
}
