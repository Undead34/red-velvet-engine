use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur when validating [`RolloutPolicy`].
#[derive(Debug, Clone, Serialize, Deserialize, Error, PartialEq, Eq)]
pub enum RuleRolloutError {
  /// The provided percentage exceeds the maximum allowed value (100).
  #[error("invalid rollout percent {percent}; expected 0..=100")]
  InvalidPercent {
    /// The invalid percentage value.
    percent: u8,
  },
}

/// Percentage-based traffic gating for a rule.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RolloutPolicy {
  /// The percentage of traffic (0..=100) subjected to this rule.
  pub percent: u8,
}

impl RolloutPolicy {
  /// Returns `true` if `bucket_0_99 < percent`.
  pub fn is_allowed(&self, bucket_0_99: u8) -> bool {
    bucket_0_99 < self.percent.min(100)
  }

  /// Alias for [`Self::is_allowed`].
  pub fn allows(&self, bucket_0_99: u8) -> bool {
    self.is_allowed(bucket_0_99)
  }

  /// Creates a rollout policy.
  ///
  /// # Errors
  ///
  /// Returns [`RuleRolloutError::InvalidPercent`] if `percent > 100`.
  pub fn new(percent: u8) -> Result<Self, RuleRolloutError> {
    if percent > 100 {
      return Err(RuleRolloutError::InvalidPercent { percent });
    }
    Ok(Self { percent })
  }

  /// Validates this policy.
  pub fn validate(&self) -> Result<(), RuleRolloutError> {
    RolloutPolicy::new(self.percent).map(|_| ())
  }
}
