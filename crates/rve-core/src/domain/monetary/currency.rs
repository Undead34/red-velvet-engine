use std::fmt;
use std::str::FromStr;

use iso4217_catalog::{CurrencyCode, CurrencyMeta};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::domain::{DomainError, DomainResult};

use super::crypto::{CryptoAsset, find_crypto_asset};
pub use iso4217_catalog::{CATALOG_VERSION, CurrencyStatus};

/// Currency/asset representation supporting both ISO-4217 fiat codes and a curated set of cryptoassets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Currency {
  Fiat(CurrencyCode),
  Crypto(&'static CryptoAsset),
}

impl Currency {
  pub fn new(value: impl Into<String>) -> DomainResult<Self> {
    let value = value.into();
    let upper = value.to_ascii_uppercase();
    if let Ok(code) = CurrencyCode::from_str(&upper) {
      return Ok(Currency::Fiat(code));
    }
    if let Some(asset) = find_crypto_asset(&upper) {
      return Ok(Currency::Crypto(asset));
    }
    Err(DomainError::InvalidCurrencyCode(value))
  }

  #[must_use]
  pub fn from_code(code: CurrencyCode) -> Self {
    Currency::Fiat(code)
  }

  pub fn from_numeric(value: u16) -> Option<Self> {
    CurrencyCode::try_from(value).ok().map(Currency::Fiat)
  }

  #[must_use]
  pub fn as_str(&self) -> &'static str {
    match self {
      Currency::Fiat(code) => code.alpha(),
      Currency::Crypto(asset) => asset.code,
    }
  }

  #[must_use]
  pub fn exponent(&self) -> u8 {
    match self {
      Currency::Fiat(code) => code.digit().unwrap_or(0),
      Currency::Crypto(asset) => asset.exponent,
    }
  }

  #[must_use]
  pub fn numeric_code(&self) -> u16 {
    match self {
      Currency::Fiat(code) => code.num(),
      Currency::Crypto(_) => 0,
    }
  }

  #[must_use]
  pub fn display_name(&self) -> &'static str {
    match self {
      Currency::Fiat(code) => code.name(),
      Currency::Crypto(asset) => asset.name,
    }
  }

  #[must_use]
  pub fn symbol(&self) -> &'static str {
    match self {
      Currency::Fiat(code) => code.alpha(),
      Currency::Crypto(asset) => asset.symbol,
    }
  }

  #[must_use]
  pub fn status(&self) -> CurrencyStatus {
    match self {
      Currency::Fiat(code) => code.status(),
      Currency::Crypto(_) => CurrencyStatus::Active,
    }
  }

  #[must_use]
  pub fn is_crypto(&self) -> bool {
    matches!(self, Currency::Crypto(_))
  }

  pub fn crypto_metadata(&self) -> Option<&'static CryptoAsset> {
    match self {
      Currency::Fiat(_) => None,
      Currency::Crypto(asset) => Some(asset),
    }
  }

  pub fn fiat_meta(&self) -> Option<CurrencyMeta> {
    match self {
      Currency::Fiat(code) => Some(code.meta()),
      Currency::Crypto(_) => None,
    }
  }
}

impl Serialize for Currency {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(self.as_str())
  }
}

impl<'de> Deserialize<'de> for Currency {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let value = String::deserialize(deserializer)?;
    Currency::new(value).map_err(serde::de::Error::custom)
  }
}

impl TryFrom<String> for Currency {
  type Error = DomainError;

  fn try_from(value: String) -> DomainResult<Self> {
    Currency::new(value)
  }
}

impl From<Currency> for String {
  fn from(value: Currency) -> Self {
    value.as_str().to_owned()
  }
}

impl fmt::Display for Currency {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.as_str())
  }
}

#[cfg(test)]
mod tests {
  use super::{CATALOG_VERSION, Currency, CurrencyStatus};

  #[test]
  fn known_fiat_currency_has_metadata() {
    let jpy = Currency::new("JPY").unwrap();
    assert_eq!(jpy.numeric_code(), 392);
    assert_eq!(jpy.exponent(), 0);
    assert_eq!(jpy.display_name(), "Yen");
    assert_eq!(jpy.status(), CurrencyStatus::Active);
    assert!(!jpy.is_crypto());
  }

  #[test]
  fn crypto_currency_exposes_metadata() {
    let btc = Currency::new("btc").unwrap();
    assert!(btc.is_crypto());
    assert_eq!(btc.exponent(), 8);
    assert_eq!(btc.display_name(), "Bitcoin");
    assert_eq!(btc.symbol(), "₿");
  }

  #[test]
  fn rejects_unknown_currency() {
    assert!(Currency::new("ABCDEF").is_err());
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
