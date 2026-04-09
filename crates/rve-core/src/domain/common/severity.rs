use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use thiserror::Error;

/// Operational severity level for fraud outcomes.
///
/// Levels are ordered from highest to lowest impact and mapped to the
/// numeric range `1..=10`.
///
/// Numeric buckets:
/// - `10` -> [`Severity::Catastrophic`]
/// - `8..=9` -> [`Severity::VeryHigh`]
/// - `6..=7` -> [`Severity::High`]
/// - `4..=5` -> [`Severity::Moderate`]
/// - `2..=3` -> [`Severity::Low`]
/// - `1` -> [`Severity::None`]
///
/// Level guide:
/// - `catastrophic`: critical incident, potential systemic/regulatory impact.
/// - `very_high`: major fraud exposure requiring immediate containment.
/// - `high`: significant impact with strong operational urgency.
/// - `moderate`: controlled impact, requires analyst review.
/// - `low`: minor issue, usually handled by routine controls.
/// - `none`: informational/no meaningful impact.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
  /// Critical impact and potential systemic risk.
  Catastrophic,
  /// Major impact requiring urgent mitigation.
  VeryHigh,
  /// Significant impact requiring rapid response.
  High,
  /// Manageable impact that still requires investigation.
  Moderate,
  /// Minor impact with limited operational risk.
  Low,
  /// Informational or negligible impact.
  None,
}

/// Errors returned when converting numeric values into [`Severity`].
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SeverityError {
  /// Provided value is outside the accepted `1..=10` domain.
  #[error("severity value out of range: {value} (expected 1..=10)")]
  OutOfRange { value: u8 },
}

impl Severity {
  /// Returns the representative numeric value for this severity.
  ///
  /// For bucketed levels, this method returns the upper bound of the bucket.
  ///
  /// # Examples
  ///
  /// ```
  /// use rve_core::domain::common::Severity;
  ///
  /// assert_eq!(Severity::High.value(), 7);
  /// assert_eq!(Severity::VeryHigh.value(), 9);
  /// ```
  #[must_use]
  pub const fn value(self) -> u8 {
    match self {
      Self::Catastrophic => 10,
      Self::VeryHigh => 9,
      Self::High => 7,
      Self::Moderate => 5,
      Self::Low => 3,
      Self::None => 1,
    }
  }

  /// Converts a numeric score (`1..=10`) into a severity bucket.
  ///
  /// # Errors
  ///
  /// Returns [`SeverityError::OutOfRange`] when `value` is not in `1..=10`.
  ///
  /// # Examples
  ///
  /// ```
  /// # use rve_core::domain::common::{Severity, SeverityError};
  /// # use std::error::Error;
  /// # fn severity_demo() -> Result<(), Box<dyn Error>> {
  /// assert_eq!(Severity::new(10)?, Severity::Catastrophic);
  /// assert!(matches!(Severity::new(0), Err(SeverityError::OutOfRange { value: 0 })));
  /// # Ok(())
  /// # }
  /// # severity_demo().unwrap();
  /// ```
  pub const fn new(value: u8) -> Result<Self, SeverityError> {
    match value {
      10 => Ok(Self::Catastrophic),
      9 | 8 => Ok(Self::VeryHigh),
      7 | 6 => Ok(Self::High),
      5 | 4 => Ok(Self::Moderate),
      3 | 2 => Ok(Self::Low),
      1 => Ok(Self::None),
      _ => Err(SeverityError::OutOfRange { value }),
    }
  }

  /// Converts a numeric score (`1..=10`) into a severity bucket.
  ///
  /// This is a convenience wrapper over [`Severity::new`] for call sites that
  /// prefer `Option`.
  #[must_use]
  pub const fn from_u8(value: u8) -> Option<Self> {
    match Self::new(value) {
      Ok(severity) => Some(severity),
      Err(_) => None,
    }
  }

  /// Returns a short human-readable description of the impact level.
  #[must_use]
  pub const fn description(self) -> &'static str {
    match self {
      Self::Catastrophic => "Systemic or critical business impact",
      Self::VeryHigh => "Major financial or operational impact",
      Self::High => "Significant and time-sensitive impact",
      Self::Moderate => "Controlled impact requiring investigation",
      Self::Low => "Minor impact with low urgency",
      Self::None => "No meaningful impact",
    }
  }
}

impl TryFrom<u8> for Severity {
  type Error = SeverityError;

  fn try_from(value: u8) -> Result<Self, Self::Error> {
    Self::new(value)
  }
}

impl PartialOrd for Severity {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for Severity {
  fn cmp(&self, other: &Self) -> Ordering {
    self.value().cmp(&other.value())
  }
}

#[cfg(test)]
mod tests {
  use super::{Severity, SeverityError};

  #[test]
  fn value_uses_upper_bound_for_bucketed_levels() {
    assert_eq!(Severity::Catastrophic.value(), 10);
    assert_eq!(Severity::VeryHigh.value(), 9);
    assert_eq!(Severity::High.value(), 7);
    assert_eq!(Severity::Moderate.value(), 5);
    assert_eq!(Severity::Low.value(), 3);
    assert_eq!(Severity::None.value(), 1);
  }

  #[test]
  fn new_maps_all_buckets() {
    assert_eq!(Severity::new(10).unwrap(), Severity::Catastrophic);
    assert_eq!(Severity::new(9).unwrap(), Severity::VeryHigh);
    assert_eq!(Severity::new(8).unwrap(), Severity::VeryHigh);
    assert_eq!(Severity::new(7).unwrap(), Severity::High);
    assert_eq!(Severity::new(6).unwrap(), Severity::High);
    assert_eq!(Severity::new(5).unwrap(), Severity::Moderate);
    assert_eq!(Severity::new(4).unwrap(), Severity::Moderate);
    assert_eq!(Severity::new(3).unwrap(), Severity::Low);
    assert_eq!(Severity::new(2).unwrap(), Severity::Low);
    assert_eq!(Severity::new(1).unwrap(), Severity::None);
  }

  #[test]
  fn new_rejects_out_of_range_values() {
    assert!(matches!(Severity::new(0), Err(SeverityError::OutOfRange { value: 0 })));
    assert!(matches!(Severity::new(11), Err(SeverityError::OutOfRange { value: 11 })));
  }

  #[test]
  fn from_u8_remains_convenience_option_api() {
    assert_eq!(Severity::from_u8(10), Some(Severity::Catastrophic));
    assert_eq!(Severity::from_u8(0), None);
  }

  #[test]
  fn description_is_stable_and_non_empty() {
    for severity in [
      Severity::Catastrophic,
      Severity::VeryHigh,
      Severity::High,
      Severity::Moderate,
      Severity::Low,
      Severity::None,
    ] {
      assert!(!severity.description().is_empty());
    }
  }

  #[test]
  fn derived_order_follows_impact_descending() {
    assert!(Severity::Catastrophic > Severity::VeryHigh);
    assert!(Severity::VeryHigh > Severity::High);
    assert!(Severity::High > Severity::Moderate);
    assert!(Severity::Moderate > Severity::Low);
    assert!(Severity::Low > Severity::None);
  }

  #[test]
  fn derived_order_and_numeric_value_are_aligned() {
    let ordered = [
      Severity::None,
      Severity::Low,
      Severity::Moderate,
      Severity::High,
      Severity::VeryHigh,
      Severity::Catastrophic,
    ];

    for pair in ordered.windows(2) {
      let left = pair[0];
      let right = pair[1];

      assert!(left < right);
      assert!(left.value() < right.value());
    }
  }

  #[test]
  fn max_returns_worst_severity() {
    let severities = vec![Severity::Low, Severity::Catastrophic, Severity::Moderate];
    let worst = severities.into_iter().max();

    assert_eq!(worst, Some(Severity::Catastrophic));
  }
}
