use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

use crate::domain::common::Severity;

/// Risk score in the closed range `1.0..=10.0`.
///
/// The type stores values as fixed-point hundredths to keep deterministic
/// arithmetic and stable serialization.
///
/// # Invariant
///
/// - `1.0 <= score <= 10.0`
///
/// # Serde
///
/// `Score` serializes as `f32` and deserializes through validation.
///
/// # Examples
///
/// ```
/// # use rve_core::domain::common::Score;
/// # use std::error::Error;
/// # fn demo() -> Result<(), Box<dyn Error>> {
/// let score = Score::new(6.5)?;
/// assert_eq!(score.as_f32(), 6.5);
/// # Ok(())
/// # }
/// # demo().unwrap();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "f32", into = "f32")]
pub struct Score(u16);

/// Errors returned when constructing or converting a [`Score`].
#[derive(Debug, Error, Clone, PartialEq)]
pub enum ScoreError {
  /// Input is NaN or infinite.
  #[error("score must be finite")]
  NonFinite,
  /// Input is outside `1.0..=10.0`.
  #[error("score out of range: {value} (expected 1.0..=10.0)")]
  OutOfRange { value: f32 },
}

impl Score {
  const SCALE: f32 = 100.0;
  const MIN: f32 = 1.0;
  const MAX: f32 = 10.0;

  /// Creates a validated score.
  ///
  /// # Errors
  ///
  /// - [`ScoreError::NonFinite`] for `NaN` and infinities.
  /// - [`ScoreError::OutOfRange`] when outside `1.0..=10.0`.
  ///
  /// # Examples
  ///
  /// ```
  /// # use rve_core::domain::common::{Score, ScoreError};
  /// # use std::error::Error;
  /// # fn score_new() -> Result<(), Box<dyn Error>> {
  /// assert_eq!(Score::new(7.25)?.as_f32(), 7.25);
  /// assert!(matches!(Score::new(0.9), Err(ScoreError::OutOfRange { .. })));
  /// # Ok(())
  /// # }
  /// # score_new().unwrap();
  /// ```
  pub fn new(value: f32) -> Result<Self, ScoreError> {
    if !value.is_finite() {
      return Err(ScoreError::NonFinite);
    }
    if !(Self::MIN..=Self::MAX).contains(&value) {
      return Err(ScoreError::OutOfRange { value });
    }

    let scaled = (value * Self::SCALE).round() as u16;
    Ok(Self(scaled))
  }

  /// Returns this score as a floating-point number.
  #[must_use]
  pub fn as_f32(self) -> f32 {
    self.0 as f32 / Self::SCALE
  }
}

impl TryFrom<f32> for Score {
  type Error = ScoreError;

  fn try_from(value: f32) -> Result<Self, Self::Error> {
    Self::new(value)
  }
}

impl From<Score> for f32 {
  fn from(value: Score) -> Self {
    value.as_f32()
  }
}

impl From<Severity> for Score {
  fn from(severity: Severity) -> Self {
    Self::new(severity.value() as f32).expect("severity representative value is always valid")
  }
}

impl From<Score> for Severity {
  /// Maps a score to severity using integer truncation of fixed-point storage.
  ///
  /// This conversion intentionally avoids floating-point arithmetic and works
  /// directly on the scaled integer representation.
  fn from(score: Score) -> Self {
    let integer = (score.0 / 100) as u8;
    Severity::new(integer).expect("score invariant guarantees integer bucket in 1..=10")
  }
}

impl fmt::Display for Score {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{:.2}", self.as_f32())
  }
}

#[cfg(test)]
mod tests {
  use crate::domain::common::Severity;

  use super::{Score, ScoreError};

  #[test]
  fn rejects_non_finite_values() {
    assert!(matches!(Score::new(f32::NAN), Err(ScoreError::NonFinite)));
    assert!(matches!(Score::new(f32::INFINITY), Err(ScoreError::NonFinite)));
  }

  #[test]
  fn rejects_out_of_range_values() {
    assert!(matches!(Score::new(0.99), Err(ScoreError::OutOfRange { .. })));
    assert!(matches!(Score::new(10.01), Err(ScoreError::OutOfRange { .. })));
  }

  #[test]
  fn serializes_as_decimal_number() {
    let score = Score::new(6.5).unwrap();
    let json = serde_json::to_string(&score).unwrap();
    assert_eq!(json, "6.5");
  }

  #[test]
  fn deserializes_with_validation() {
    let score: Score = serde_json::from_str("6.5").unwrap();
    assert_eq!(score.as_f32(), 6.5);

    let invalid = serde_json::from_str::<Score>("0");
    assert!(invalid.is_err());
  }

  #[test]
  fn maps_to_severity_without_float_rounding_risk() {
    assert_eq!(Severity::from(Score::new(1.0).unwrap()), Severity::None);
    assert_eq!(Severity::from(Score::new(1.99).unwrap()), Severity::None);
    assert_eq!(Severity::from(Score::new(2.0).unwrap()), Severity::Low);
    assert_eq!(Severity::from(Score::new(9.99).unwrap()), Severity::VeryHigh);
    assert_eq!(Severity::from(Score::new(10.0).unwrap()), Severity::Catastrophic);
  }
}
