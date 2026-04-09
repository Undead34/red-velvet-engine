use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use thiserror::Error;

use super::Currency;

/// Monetary amount represented as minor units and currency.
///
/// `Money` is a domain value object with exact arithmetic semantics:
/// amount and currency are inseparable.
///
/// Internally, values are stored in **minor units** (for example cents for USD)
/// according to the ISO-4217 exponent of the currency.
///
/// Terminology:
/// - **major units**: human-facing amount (`"12.34"` USD means 12 dollars + 34 cents)
/// - **minor units**: smallest integer unit (for USD, `1234`)
/// - **ccy**: currency code (ISO-4217), e.g. `USD`, `JPY`, `KWD`
///
/// # Examples
///
/// ```
/// # use rve_core::domain::common::{Currency, Money};
/// # use std::error::Error;
/// # fn run() -> Result<(), Box<dyn Error>> {
/// let usd = Currency::new("USD")?;
///
/// let from_major = Money::from_major_str("12.34", usd.clone())?;
/// assert_eq!(from_major.minor_units(), 1_234);
///
/// let from_minor = Money::from_minor(1_234, usd)?;
/// assert_eq!(from_minor.value(), 12.34);
/// # Ok(())
/// # }
/// # run().unwrap();
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Money {
  minor_units: u64,
  ccy: Currency,
}

/// Errors returned by money construction and operations.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum MoneyError {
  #[error("money amount must be non-negative")]
  NegativeAmount,
  #[error("invalid decimal amount format: {0}")]
  InvalidFormat(String),
  #[error("too many fractional digits for {currency}: got {provided}, max {allowed}")]
  ScaleMismatch { currency: String, provided: u8, allowed: u8 },
  #[error("money amount overflow")]
  Overflow,
  #[error("currency mismatch: left={left}, right={right}")]
  CurrencyMismatch { left: String, right: String },
}

impl Money {
  /// Creates money from exact minor units.
  ///
  /// Use this constructor when data is already normalized to integer minor
  /// units (for example persisted values, ledger rows, or internal messages).
  ///
  /// # Examples
  ///
  /// ```
  /// # use rve_core::domain::common::{Currency, Money};
  /// # use std::error::Error;
  /// # fn example() -> Result<(), Box<dyn Error>> {
  /// let money = Money::from_minor(1_050, Currency::new("USD")?)?;
  /// assert_eq!(money.value(), 10.5);
  /// # Ok(())
  /// # }
  /// # example().unwrap();
  /// ```
  pub fn from_minor(minor_units: u64, ccy: Currency) -> Result<Self, MoneyError> {
    Ok(Self { minor_units, ccy })
  }

  /// Creates money from a decimal major-unit string.
  ///
  /// This parser accepts canonical decimal format with `.` as separator.
  /// Locale-specific input (such as `"123,45"`) is rejected and should be
  /// normalized at the API/UI boundary.
  ///
  /// # Errors
  ///
  /// - [`MoneyError::InvalidFormat`] for malformed decimal strings.
  /// - [`MoneyError::NegativeAmount`] for values below zero.
  /// - [`MoneyError::ScaleMismatch`] when fractional digits exceed currency precision.
  /// - [`MoneyError::Overflow`] when numeric conversion overflows.
  ///
  /// # Examples
  ///
  /// ```
  /// # use rve_core::domain::common::{Currency, Money};
  /// # use std::error::Error;
  /// # fn parse_demo() -> Result<(), Box<dyn Error>> {
  /// let usd = Currency::new("USD")?;
  /// let jpy = Currency::new("JPY")?;
  ///
  /// let usd_money = Money::from_major_str("123.45", usd)?;
  /// assert_eq!(usd_money.minor_units(), 12_345);
  ///
  /// assert!(Money::from_major_str("123.45", jpy).is_err());
  /// # Ok(())
  /// # }
  /// # parse_demo().unwrap();
  /// ```
  pub fn from_major_str(value: &str, ccy: Currency) -> Result<Self, MoneyError> {
    let value = value.trim();
    if value.is_empty() {
      return Err(MoneyError::InvalidFormat("empty amount".to_owned()));
    }
    if value.starts_with('-') {
      return Err(MoneyError::NegativeAmount);
    }

    let exponent = ccy.exponent();
    let parts: Vec<&str> = value.split('.').collect();
    if parts.len() > 2 {
      return Err(MoneyError::InvalidFormat(value.to_owned()));
    }

    let whole_str = if parts[0].is_empty() { "0" } else { parts[0] };
    let whole =
      whole_str.parse::<u64>().map_err(|_| MoneyError::InvalidFormat(value.to_owned()))?;

    let frac = if parts.len() == 2 { parts[1] } else { "" };
    if !frac.chars().all(|c| c.is_ascii_digit()) {
      return Err(MoneyError::InvalidFormat(value.to_owned()));
    }

    let provided = frac.len() as u8;
    if provided > exponent {
      return Err(MoneyError::ScaleMismatch {
        currency: ccy.as_str().to_owned(),
        provided,
        allowed: exponent,
      });
    }

    let factor = ten_pow_u64(exponent)?;
    let whole_minor = whole.checked_mul(factor).ok_or(MoneyError::Overflow)?;

    let frac_value = if frac.is_empty() {
      0
    } else {
      frac.parse::<u64>().map_err(|_| MoneyError::InvalidFormat(value.to_owned()))?
    };

    let frac_factor = ten_pow_u64(exponent - provided)?;
    let frac_minor = frac_value.checked_mul(frac_factor).ok_or(MoneyError::Overflow)?;
    let minor_units = whole_minor.checked_add(frac_minor).ok_or(MoneyError::Overflow)?;

    Self::from_minor(minor_units, ccy)
  }

  #[must_use]
  pub fn minor_units(&self) -> u64 {
    self.minor_units
  }

  /// Returns the ISO-4217 currency code associated with this amount.
  #[must_use]
  pub fn ccy(&self) -> &Currency {
    &self.ccy
  }

  /// Returns a floating-point representation in major units.
  ///
  /// This is provided for display/interoperability. Domain arithmetic should
  /// continue to use `minor_units` and checked operations.
  #[must_use]
  pub fn value(&self) -> f64 {
    self.minor_units as f64
      / ten_pow_u64(self.ccy.exponent()).expect("currency exponent must fit") as f64
  }

  /// Adds two amounts of the same currency.
  ///
  /// # Errors
  ///
  /// Returns [`MoneyError::CurrencyMismatch`] when currencies differ.
  /// Returns [`MoneyError::Overflow`] on integer overflow.
  pub fn checked_add(&self, other: &Self) -> Result<Self, MoneyError> {
    ensure_same_currency(self, other)?;
    let minor_units =
      self.minor_units.checked_add(other.minor_units).ok_or(MoneyError::Overflow)?;
    Self::from_minor(minor_units, self.ccy.clone())
  }

  /// Subtracts two amounts of the same currency.
  ///
  /// # Errors
  ///
  /// Returns [`MoneyError::CurrencyMismatch`] when currencies differ.
  /// Returns [`MoneyError::NegativeAmount`] if subtraction would go below zero.
  pub fn checked_sub(&self, other: &Self) -> Result<Self, MoneyError> {
    ensure_same_currency(self, other)?;
    let minor_units =
      self.minor_units.checked_sub(other.minor_units).ok_or(MoneyError::NegativeAmount)?;
    Self::from_minor(minor_units, self.ccy.clone())
  }
}

impl PartialOrd for Money {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    if self.ccy == other.ccy { Some(self.minor_units.cmp(&other.minor_units)) } else { None }
  }
}

fn ensure_same_currency(left: &Money, right: &Money) -> Result<(), MoneyError> {
  if left.ccy == right.ccy {
    Ok(())
  } else {
    Err(MoneyError::CurrencyMismatch {
      left: left.ccy.as_str().to_owned(),
      right: right.ccy.as_str().to_owned(),
    })
  }
}

fn ten_pow_u64(exp: u8) -> Result<u64, MoneyError> {
  10u64.checked_pow(exp as u32).ok_or(MoneyError::Overflow)
}

#[cfg(test)]
mod tests {
  use super::{Money, MoneyError};
  use crate::domain::common::Currency;

  #[test]
  fn parses_major_string_with_iso_exponent() {
    let usd = Currency::new("USD").unwrap();
    let money = Money::from_major_str("123.45", usd).unwrap();
    assert_eq!(money.minor_units(), 12_345);
  }

  #[test]
  fn rejects_fraction_scale_for_jpy() {
    let jpy = Currency::new("JPY").unwrap();
    let error = Money::from_major_str("10.50", jpy).expect_err("must reject fractional JPY");
    assert!(matches!(error, MoneyError::ScaleMismatch { .. }));
  }

  #[test]
  fn checked_add_requires_same_currency() {
    let usd = Currency::new("USD").unwrap();
    let eur = Currency::new("EUR").unwrap();
    let a = Money::from_major_str("1.00", usd).unwrap();
    let b = Money::from_major_str("1.00", eur).unwrap();
    let error = a.checked_add(&b).expect_err("must reject cross-currency add");
    assert!(matches!(error, MoneyError::CurrencyMismatch { .. }));
  }

  #[test]
  fn partial_order_works_for_same_currency() {
    let usd = Currency::new("USD").unwrap();
    let low = Money::from_major_str("10.00", usd.clone()).unwrap();
    let high = Money::from_major_str("20.00", usd).unwrap();

    assert!(high > low);
  }

  #[test]
  fn partial_order_is_none_for_different_currencies() {
    let usd = Currency::new("USD").unwrap();
    let eur = Currency::new("EUR").unwrap();
    let left = Money::from_major_str("10.00", usd).unwrap();
    let right = Money::from_major_str("20.00", eur).unwrap();

    assert_eq!(left.partial_cmp(&right), None);
    assert!(!(left > right));
  }

  #[test]
  fn serializes_with_minor_units_shape() {
    let usd = Currency::new("USD").unwrap();
    let money = Money::from_major_str("10.50", usd).unwrap();
    let json = serde_json::to_string(&money).unwrap();
    assert_eq!(json, r#"{"minor_units":1050,"ccy":"USD"}"#);
  }
}
