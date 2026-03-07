use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::common::TimestampMs;

/// Errors that can occur when validating [`RuleSchedule`].
#[derive(Debug, Clone, Serialize, Deserialize, Error, PartialEq, Eq)]
pub enum RuleScheduleError {
  /// `active_until_ms` is less than or equal to `active_from_ms`.
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

/// Optional time window for rule execution.
///
/// The window is half-open: `[active_from_ms, active_until_ms)`.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RuleSchedule {
  /// The inclusive start timestamp establishing execution eligibility.
  pub active_from_ms: Option<TimestampMs>,

  /// The exclusive end timestamp ceasing execution eligibility.
  pub active_until_ms: Option<TimestampMs>,
}

impl RuleSchedule {
  /// Returns `true` if `now_ms` is inside the configured window.
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

  /// Alias for [`Self::is_within_window`].
  pub fn allows(&self, now_ms: u64) -> bool {
    self.is_within_window(now_ms)
  }

  /// Validates schedule boundaries.
  ///
  /// # Errors
  ///
  /// Returns [`RuleScheduleError::InvalidWindow`] if both boundaries are set and
  /// `active_until_ms <= active_from_ms`.
  pub fn validate(&self) -> Result<(), RuleScheduleError> {
    if let (Some(from), Some(until)) = (self.active_from_ms, self.active_until_ms)
      && until.as_u64() <= from.as_u64()
    {
      return Err(RuleScheduleError::InvalidWindow { from: from.as_u64(), until: until.as_u64() });
    }
    Ok(())
  }

  /// Creates a new schedule and validates it.
  ///
  /// # Errors
  ///
  /// Returns [`RuleScheduleError`] if boundaries are invalid.
  pub fn new(
    active_from_ms: Option<TimestampMs>,
    active_until_ms: Option<TimestampMs>,
  ) -> Result<Self, RuleScheduleError> {
    let schedule = Self { active_from_ms, active_until_ms };
    schedule.validate()?;
    Ok(schedule)
  }
}
