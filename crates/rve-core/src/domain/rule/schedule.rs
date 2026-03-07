use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::common::TimestampMs;

/// The error type returned when a rule's temporal boundaries are invalid.
#[derive(Debug, Clone, Serialize, Deserialize, Error, PartialEq, Eq)]
pub enum RuleScheduleError {
  /// The end boundary chronologically precedes or equals the start boundary.
  #[error(
    "invalid schedule window: active_until_ms ({until}) must be greater than active_from_ms ({from})"
  )]
  InvalidWindow {
    /// The invalid inclusive start timestamp.
    from: u64,
    /// The invalid exclusive end timestamp.
    until: u64,
  },
}

/// The temporal boundaries defining a rule's active operational window.
///
/// `RuleSchedule` establishes when a rule is eligible for engine evaluation.
/// Bounded intervals are evaluated as half-open ranges `[from, until)`.
/// Unbounded bounds (`None`) are treated as positive or negative infinity.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RuleSchedule {
  /// The inclusive start timestamp establishing execution eligibility.
  pub active_from_ms: Option<TimestampMs>,

  /// The exclusive end timestamp ceasing execution eligibility.
  pub active_until_ms: Option<TimestampMs>,
}

impl RuleSchedule {
  /// Evaluates if the provided timestamp falls within the schedule's boundaries.
  ///
  /// Returns `true` if `now_ms` is greater than or equal to `active_from_ms`
  /// and strictly less than `active_until_ms`.
  /// Missing bounds are open (always `true` on that side).
  pub fn is_within_window(&self, now_ms: u64) -> bool {
    if let Some(from) = self.active_from_ms {
      if now_ms < from.as_u64() {
        return false;
      }
    }

    if let Some(until) = self.active_until_ms {
      if now_ms >= until.as_u64() {
        return false;
      }
    }

    true
  }

  /// Alias for [`RuleSchedule::is_within_window`].
  pub fn allows(&self, now_ms: u64) -> bool {
    self.is_within_window(now_ms)
  }

  /// Validates the chronological integrity of the schedule's boundaries.
  ///
  /// Returns a [`RuleScheduleError::InvalidWindow`] if both boundaries are present
  /// and `active_until_ms` is less than or equal to `active_from_ms`.
  pub fn validate(&self) -> Result<(), RuleScheduleError> {
    if let (Some(from), Some(until)) = (self.active_from_ms, self.active_until_ms)
      && until.as_u64() <= from.as_u64()
    {
      return Err(RuleScheduleError::InvalidWindow { from: from.as_u64(), until: until.as_u64() });
    }
    Ok(())
  }

  /// Creates a new schedule after validating window consistency.
  ///
  /// Returns a [`RuleScheduleError`] if the defined window is chronologically invalid.
  pub fn new(
    active_from_ms: Option<TimestampMs>,
    active_until_ms: Option<TimestampMs>,
  ) -> Result<Self, RuleScheduleError> {
    let schedule = Self { active_from_ms, active_until_ms };
    schedule.validate()?;
    Ok(schedule)
  }
}
