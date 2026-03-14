use std::str::FromStr;

use iso4217_catalog::{CurrencyCode, CurrencyMeta};
use serde::{Deserialize, Serialize};

use crate::domain::DomainError;

pub use iso4217_catalog::{CATALOG_VERSION, CurrencyStatus};
pub type CurrencySpec = CurrencyMeta;

/// ISO-4217 currency code validated against the generated catalog.
///
/// `Currency` is a lightweight domain wrapper around [`CurrencyCode`] with
/// serde support and domain-oriented constructors.
///
/// Notes:
/// - `ccy` in domain payloads refers to this type.
/// - Exponent/minor-unit precision comes from the catalog generated at
///   compile-time from SIX `list-one.xml`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(try_from = "String", into = "String")]
pub struct Currency(CurrencyCode);

impl Currency {
  /// Creates a currency from a 3-letter ISO code (for example `"USD"`).
  ///
  /// # Errors
  ///
  /// Returns [`DomainError::InvalidCurrencyCode`] when code is unknown.
  pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
    let value = value.into();
    let code = CurrencyCode::from_str(&value)
      .map_err(|_| DomainError::InvalidCurrencyCode(value.clone()))?;
    Ok(Self(code))
  }

  #[must_use]
  pub fn from_code(code: CurrencyCode) -> Self {
    Self(code)
  }

  /// Returns currency from numeric ISO code (for example `840` -> `USD`).
  pub fn from_numeric(value: u16) -> Option<Self> {
    CurrencyCode::try_from(value).ok().map(Self)
  }

  #[must_use]
  pub fn as_code(self) -> CurrencyCode {
    self.0
  }

  /// Returns the alpha code (e.g. `"USD"`, `"JPY"`).
  #[must_use]
  pub fn as_str(&self) -> &'static str {
    self.0.alpha()
  }

  #[must_use]
  pub fn spec(&self) -> CurrencyMeta {
    self.0.meta()
  }

  /// Returns the ISO minor-unit exponent (digits after decimal point).
  ///
  /// Examples:
  /// - USD -> `2`
  /// - JPY -> `0`
  /// - KWD -> `3`
  #[must_use]
  pub fn exponent(&self) -> u8 {
    self.0.digit().unwrap_or(0)
  }

  #[must_use]
  pub fn numeric_code(&self) -> u16 {
    self.0.num()
  }

  #[must_use]
  pub fn display_name(&self) -> &'static str {
    self.0.name()
  }

  #[must_use]
  pub fn status(&self) -> CurrencyStatus {
    self.0.status()
  }
}

impl TryFrom<String> for Currency {
  type Error = DomainError;

  fn try_from(value: String) -> Result<Self, Self::Error> {
    Self::new(value)
  }
}

impl From<Currency> for String {
  fn from(value: Currency) -> Self {
    value.0.alpha().to_owned()
  }
}

#[cfg(test)]
mod tests {
  use super::{CATALOG_VERSION, Currency, CurrencyStatus};

  #[test]
  fn known_currency_has_metadata() {
    let jpy = Currency::new("JPY").unwrap();
    assert_eq!(jpy.numeric_code(), 392);
    assert_eq!(jpy.exponent(), 0);
    assert_eq!(jpy.display_name(), "Yen");
    assert_eq!(jpy.status(), CurrencyStatus::Active);
  }

  #[test]
  fn supports_funds_and_metals_from_list_one() {
    let xau = Currency::new("XAU").unwrap();
    assert_eq!(xau.status(), CurrencyStatus::Metal);
    assert_eq!(xau.exponent(), 0);

    let xts = Currency::new("XTS").unwrap();
    assert_eq!(xts.status(), CurrencyStatus::Testing);
  }

  #[test]
  fn rejects_unknown_currency() {
    assert!(Currency::new("ABC").is_err());
  }

  #[test]
  fn numeric_lookup_works() {
    let jpy = Currency::from_numeric(392).unwrap();
    assert_eq!(jpy.as_str(), "JPY");
  }

  #[test]
  fn catalog_version_is_non_empty() {
    assert!(!CATALOG_VERSION.is_empty());
  }
}
