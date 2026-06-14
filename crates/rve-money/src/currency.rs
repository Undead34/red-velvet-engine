use std::fmt;
use std::str::FromStr;

use iso4217_catalog::{CurrencyCode, CurrencyStatus};
use serde::{Deserialize, Serialize};

/// Error parsing or validating a fiat currency code from ISO 4217.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[error("invalid fiat currency code: {0}")]
pub struct CurrencyError(pub String);

/// ISO 4217 fiat currency code.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Currency(CurrencyCode);

impl Currency {
  /// Parse a currency code from a string. Case-insensitive.
  pub fn new(code: &str) -> Result<Self, CurrencyError> {
    code.parse()
  }

  /// Construct from a raw [`CurrencyCode`], skipping validation.
  #[must_use]
  pub const fn from_code(code: CurrencyCode) -> Self {
    Self(code)
  }

  /// Return the raw [`CurrencyCode`].
  #[must_use]
  pub fn code(&self) -> CurrencyCode {
    self.0
  }

  /// Three-letter ISO 4217 alphabetic code (e.g. `"USD"`).
  #[must_use]
  pub fn alpha(&self) -> &'static str {
    self.0.alpha()
  }

  /// Three-digit ISO 4217 numeric code (e.g. `840`).
  #[must_use]
  pub fn numeric(&self) -> u16 {
    self.0.num()
  }

  /// Minor-unit exponent (e.g. `2` for USD, `0` for JPY).
  #[must_use]
  pub fn exponent(&self) -> u8 {
    self.0.digit().unwrap_or(0)
  }

  /// Human-readable currency name (e.g. `"US Dollar"`).
  #[must_use]
  pub fn name(&self) -> &'static str {
    self.0.name()
  }

  /// ISO 4217 currency status (active, testing, etc.).
  #[must_use]
  pub fn status(&self) -> CurrencyStatus {
    self.0.status()
  }
}

/// `"USD".parse::<FiatCurrency>()`
impl FromStr for Currency {
  type Err = CurrencyError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let upper = s.to_ascii_uppercase();
    CurrencyCode::from_str(&upper).map(Self).map_err(|_| CurrencyError(s.to_owned()))
  }
}

/// `FiatCurrency::try_from("USD")`
impl TryFrom<&str> for Currency {
  type Error = CurrencyError;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    value.parse()
  }
}

/// `FiatCurrency::try_from("USD".to_string())`
impl TryFrom<String> for Currency {
  type Error = CurrencyError;

  fn try_from(value: String) -> Result<Self, Self::Error> {
    value.parse()
  }
}

impl fmt::Display for Currency {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(self.alpha())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_valid_parsing() {
    let curr1 = Currency::new("USD").expect("valid USD");
    let curr2: Currency = "EUR".parse().expect("valid EUR");
    let curr3 = Currency::try_from("JPY").expect("valid JPY");

    assert_eq!(curr1.alpha(), "USD");
    assert_eq!(curr2.alpha(), "EUR");
    assert_eq!(curr3.alpha(), "JPY");
  }

  #[test]
  fn test_case_insensitivity() {
    let curr1 = Currency::new("usd").unwrap();
    let curr2 = Currency::new("eUr").unwrap();

    assert_eq!(curr1.alpha(), "USD");
    assert_eq!(curr2.alpha(), "EUR");
  }

  #[test]
  fn test_invalid_parsing() {
    let err = Currency::new("XYZ").unwrap_err();
    assert_eq!(err, CurrencyError("XYZ".to_string()));

    let err2 = Currency::new("").unwrap_err();
    assert_eq!(err2, CurrencyError("".to_string()));
  }

  #[test]
  fn test_display_trait() {
    let curr = Currency::new("USD").unwrap();
    assert_eq!(curr.to_string(), "USD");
  }

  #[test]
  fn test_currency_properties() {
    let usd = Currency::new("USD").unwrap();
    assert_eq!(usd.numeric(), 840);
    assert_eq!(usd.exponent(), 2);
    assert_eq!(usd.name(), "US Dollar");
    assert_eq!(usd.status(), CurrencyStatus::Active);

    let jpy = Currency::new("JPY").unwrap();
    assert_eq!(jpy.exponent(), 0);
  }
}
