use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The error type returned when a rollout percentage is out of the valid range.
#[derive(Debug, Clone, Serialize, Deserialize, Error, PartialEq, Eq)]
pub enum RuleRolloutError {
  /// The provided percentage exceeds the maximum allowed value (100).
  #[error("invalid rollout percent {percent}; expected 0..=100")]
  InvalidPercent {
    /// The invalid percentage value.
    percent: u8,
  },
}

/// Percentage-based traffic allocation for incremental rule deployment.
///
/// `RolloutPolicy` gatekeeps rule execution by comparing a pre-calculated
/// traffic bucket against a fixed percentage threshold.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RolloutPolicy {
  /// The percentage of traffic (0..=100) subjected to this rule.
  pub percent: u8,
}

impl RolloutPolicy {
  /// This method performs a simple range check: `bucket < percent`.
  ///
  /// Returns `true` if the bucket is inside the enabled percentage.
  pub fn is_allowed(&self, bucket_0_99: u8) -> bool {
    bucket_0_99 < self.percent.min(100)
  }

  /// An alias for [`RolloutPolicy::is_allowed`].
  pub fn allows(&self, bucket_0_99: u8) -> bool {
    self.is_allowed(bucket_0_99)
  }

  /// Creates a new [`RolloutPolicy`] and validates its constraints.
  ///
  /// # Errors
  ///
  /// Returns [`RuleRolloutError::InvalidPercent`] if `percent` exceeds 100.
  pub fn new(percent: u8) -> Result<Self, RuleRolloutError> {
    if percent > 100 {
      return Err(RuleRolloutError::InvalidPercent { percent });
    }
    Ok(Self { percent })
  }

  /// Validates the current policy configuration.
  pub fn validate(&self) -> Result<(), RuleRolloutError> {
    RolloutPolicy::new(self.percent).map(|_| ())
  }
}
