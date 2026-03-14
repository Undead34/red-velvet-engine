use serde::{Deserialize, Serialize};
use std::{
  num::NonZeroU64,
  time::{SystemTime, UNIX_EPOCH},
};
use thiserror::Error;

/// Milliseconds since Unix epoch.
///
/// `TimestampMs` is a small validated wrapper used across the domain model.
/// It enforces a single invariant: the timestamp must be strictly greater than
/// `0`.
///
/// # Invariant
///
/// - `timestamp_ms > 0`
///
/// # Serde
///
/// This type serializes as a plain `u64` and deserializes through validation.
///
/// # Examples
///
/// ```
/// use rve_core::domain::common::TimestampMs;
///
/// let ts = TimestampMs::new(1_730_000_000_000).unwrap();
/// assert_eq!(ts.as_u64(), 1_730_000_000_000);
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(try_from = "u64", into = "u64")]
pub struct TimestampMs(NonZeroU64);

/// Construction and conversion errors for [`TimestampMs`].
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TimestampMsError {
  /// The provided millisecond value is `0`, which is rejected by domain policy.
  #[error("timestamp in milliseconds must be greater than zero, got {value}")]
  NonPositive { value: u64 },
  /// The provided [`SystemTime`] is before [`UNIX_EPOCH`].
  #[error("system time is earlier than unix epoch")]
  BeforeUnixEpoch,
  /// The millisecond representation does not fit into `u64`.
  #[error("system time in milliseconds does not fit in u64")]
  Overflow,
}

impl TimestampMs {
  /// Creates a validated timestamp from epoch milliseconds.
  ///
  /// # Errors
  ///
  /// Returns [`TimestampMsError::NonPositive`] when `value == 0`.
  ///
  /// # Examples
  ///
  /// ```
  /// use rve_core::domain::common::{TimestampMs, TimestampMsError};
  ///
  /// assert!(matches!(
  ///   TimestampMs::new(0),
  ///   Err(TimestampMsError::NonPositive { value: 0 })
  /// ));
  /// ```
  pub fn new(value: u64) -> Result<Self, TimestampMsError> {
    let non_zero = NonZeroU64::new(value).ok_or(TimestampMsError::NonPositive { value })?;
    Ok(Self(non_zero))
  }

  /// Returns the underlying epoch-millis value.
  #[must_use]
  pub fn as_u64(self) -> u64 {
    self.0.get()
  }
}

impl TryFrom<u64> for TimestampMs {
  type Error = TimestampMsError;

  fn try_from(value: u64) -> Result<Self, Self::Error> {
    Self::new(value)
  }
}

impl TryFrom<SystemTime> for TimestampMs {
  type Error = TimestampMsError;

  /// Converts a [`SystemTime`] into a validated [`TimestampMs`].
  ///
  /// # Errors
  ///
  /// - [`TimestampMsError::BeforeUnixEpoch`] if `value < UNIX_EPOCH`.
  /// - [`TimestampMsError::Overflow`] if milliseconds do not fit in `u64`.
  /// - [`TimestampMsError::NonPositive`] for `UNIX_EPOCH` exactly (`0 ms`).
  ///
  /// # Examples
  ///
  /// ```
  /// use rve_core::domain::common::TimestampMs;
  /// use std::time::{Duration, UNIX_EPOCH};
  ///
  /// let ts = TimestampMs::try_from(UNIX_EPOCH + Duration::from_millis(123)).unwrap();
  /// assert_eq!(ts.as_u64(), 123);
  /// ```
  fn try_from(value: SystemTime) -> Result<Self, Self::Error> {
    let millis =
      value.duration_since(UNIX_EPOCH).map_err(|_| TimestampMsError::BeforeUnixEpoch)?.as_millis();

    let millis = u64::try_from(millis).map_err(|_| TimestampMsError::Overflow)?;
    Self::new(millis)
  }
}

impl From<TimestampMs> for u64 {
  fn from(value: TimestampMs) -> Self {
    value.0.get()
  }
}

#[cfg(test)]
mod tests {
  use super::TimestampMs;
  use std::time::{Duration, UNIX_EPOCH};

  #[test]
  fn rejects_zero() {
    assert!(TimestampMs::new(0).is_err());
  }

  #[test]
  fn converts_from_system_time() {
    let ts = TimestampMs::try_from(UNIX_EPOCH + Duration::from_millis(123)).expect("valid");
    assert_eq!(ts.as_u64(), 123);
  }

  #[test]
  fn rejects_unix_epoch_exact_zero() {
    assert!(TimestampMs::try_from(UNIX_EPOCH).is_err());
  }
}
