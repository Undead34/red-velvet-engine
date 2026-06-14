use crate::error::MoneyError;
use crate::{amount::Amount, currency::Currency};

mod amount;
mod currency;
mod error;

use std::cmp::Ordering;
use std::fmt;

/// `Money` represents a precise monetary value in a specific currency.
///
/// It safely binds an `Amount` (pure math) with a `Currency` (context),
/// ensuring that invalid cross-currency operations cannot happen.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Money {
  amount: Amount,
  currency: Currency,
}

impl Money {
  /// Creates a new `Money` instance from an existing amount and currency.
  #[inline]
  #[must_use]
  pub const fn new(amount: Amount, currency: Currency) -> Self {
    Self { amount, currency }
  }

  /// Creates a strictly zero `Money` instance for the given currency.
  #[inline]
  #[must_use]
  pub const fn zero(currency: Currency) -> Self {
    Self { amount: Amount::zero(), currency }
  }

  /// Returns a reference to the internal mathematical amount.
  #[inline]
  #[must_use]
  pub const fn amount(&self) -> Amount {
    self.amount
  }

  /// Returns the associated currency.
  #[inline]
  #[must_use]
  pub const fn currency(&self) -> Currency {
    self.currency
  }

  /// Adds two money values. Fails on overflow or if currencies do not match.
  pub fn checked_add(self, other: Self) -> Result<Self, MoneyError> {
    if self.currency != other.currency {
      return Err(MoneyError::CurrencyMismatch { expected: self.currency, found: other.currency });
    }

    let new_amount = self.amount.checked_add(other.amount).map_err(|_| MoneyError::Overflow)?;
    Ok(Self::new(new_amount, self.currency))
  }

  /// Subtracts two money values. Fails on underflow/overflow or currency mismatch.
  pub fn checked_sub(self, other: Self) -> Result<Self, MoneyError> {
    if self.currency != other.currency {
      return Err(MoneyError::CurrencyMismatch { expected: self.currency, found: other.currency });
    }

    let new_amount = self.amount.checked_sub(other.amount).map_err(|_| MoneyError::Overflow)?;
    Ok(Self::new(new_amount, self.currency))
  }

  /// Multiplies the money value by a scalar integer (e.g., billing 3 identical items).
  pub fn checked_mul(self, scalar: i128) -> Result<Self, MoneyError> {
    let new_amount = self.amount.checked_mul(scalar).map_err(|_| MoneyError::Overflow)?;
    Ok(Self::new(new_amount, self.currency))
  }

  /// Divides the money value evenly. Useful for splitting bills.
  pub fn checked_div(self, scalar: i128) -> Result<Self, MoneyError> {
    let new_amount = self.amount.checked_div(scalar).map_err(|_| MoneyError::DivideByZero)?; // or Overflow inside
    Ok(Self::new(new_amount, self.currency))
  }

  /// Safely parses a decimal string (e.g., "-150.50") into a `Money` instance,
  /// enforcing the exact scale limitations of the target currency.
  pub fn parse(input: &str, currency: Currency) -> Result<Self, MoneyError> {
    let trimmed = input.trim();

    // Block trivial invalid inputs early.
    if trimmed.is_empty() || trimmed == "." || trimmed == "-." || trimmed == "+." {
      return Err(MoneyError::InvalidFormat(trimmed.into()));
    }

    // 1. Extract the sign safely without heap allocations.
    let (sign, rest) = match trimmed.as_bytes().first() {
      Some(b'-') => (-1i128, &trimmed[1..]),
      Some(b'+') => (1i128, &trimmed[1..]),
      _ => (1i128, trimmed),
    };

    // Edge case: string was just "-" or "+"
    if rest.is_empty() {
      return Err(MoneyError::InvalidFormat(trimmed.into()));
    }

    // 2. Separate whole and fractional parts.
    let (whole_str, frac_str) = rest.split_once('.').unwrap_or((rest, ""));

    // 3. Strict character validation (must be ASCII digits only).
    if (!whole_str.is_empty() && !whole_str.chars().all(|c| c.is_ascii_digit()))
      || (!frac_str.is_empty() && !frac_str.chars().all(|c| c.is_ascii_digit()))
    {
      return Err(MoneyError::InvalidFormat(trimmed.to_owned()));
    }

    // 4. Parse the whole number portion safely.
    let whole: i128 = if whole_str.is_empty() {
      0
    } else {
      whole_str.parse().map_err(|_| MoneyError::InvalidFormat(trimmed.to_owned()))?
    };

    // 5. Validate that the user isn't supplying more precision than the currency allows.
    let exponent = currency.exponent();
    let provided_scale = frac_str.len() as u8;

    if provided_scale > exponent {
      return Err(MoneyError::InvalidScale { allowed: exponent });
    }

    // 6. Execute overflow-safe mathematical scaling.
    let factor = 10i128.checked_pow(exponent as u32).ok_or(MoneyError::Overflow)?;
    let mut units = whole.checked_mul(factor).ok_or(MoneyError::Overflow)?;

    if !frac_str.is_empty() {
      let frac_val: i128 =
        frac_str.parse().map_err(|_| MoneyError::InvalidFormat(trimmed.to_owned()))?;
      let frac_factor =
        10i128.checked_pow((exponent - provided_scale) as u32).ok_or(MoneyError::Overflow)?;
      let scaled_frac = frac_val.checked_mul(frac_factor).ok_or(MoneyError::Overflow)?;
      units = units.checked_add(scaled_frac).ok_or(MoneyError::Overflow)?;
    }

    // Apply the sign correctly.
    units = units.checked_mul(sign).ok_or(MoneyError::Overflow)?;

    Ok(Self::new(Amount::new(units), currency))
  }

  /// Formats only the numerical value as a string strictly following the currency's exponent.
  pub fn format_value(&self) -> String {
    let units = self.amount.units();
    let exponent = self.currency.exponent();

    if exponent == 0 {
      return format!("{}", units);
    }

    let negative = units < 0;
    // unsigned_abs() is critical: prevents panic on i128::MIN
    let abs_units = units.unsigned_abs();
    let factor = 10u128.pow(exponent as u32);

    let whole = abs_units / factor;
    let frac = abs_units % factor;

    let sign_str = if negative { "-" } else { "" };

    if frac == 0 {
      format!("{}{}", sign_str, whole)
    } else {
      // Pad fractional part with leading zeros based on the currency's exponent width.
      let frac_str = format!("{:0width$}", frac, width = exponent as usize);
      // Mathematically canonical representation: remove trailing redundant zeros.
      let trimmed_frac = frac_str.trim_end_matches('0');

      format!("{}{}.{}", sign_str, whole, trimmed_frac)
    }
  }
}

/// Implementing `PartialOrd` manually is a security feature.
/// It prevents comparing Money of different currencies (returning `None`).
impl PartialOrd for Money {
  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    if self.currency != other.currency {
      None // Crucial: Cross-currency comparisons are mathematically undefined without FX rates.
    } else {
      self.amount.partial_cmp(&other.amount)
    }
  }
}

/// Formats the money instance in a standard readable format (e.g., "150.5 USD").
impl fmt::Display for Money {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{} {}", self.format_value(), self.currency.alpha())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  // Mock Helper for testing
  fn usd() -> Currency {
    Currency::new("USD").unwrap()
  } // exponent 2
  fn jpy() -> Currency {
    Currency::new("JPY").unwrap()
  } // exponent 0

  #[test]
  fn test_cross_currency_protection() {
    let a = Money::new(Amount::new(100), usd());
    let b = Money::new(Amount::new(100), jpy());

    // Arithmetic Protection
    assert!(matches!(a.checked_add(b), Err(MoneyError::CurrencyMismatch { .. })));

    // Comparison Protection
    assert_eq!(a.partial_cmp(&b), None);
  }

  #[test]
  fn test_parsing_logic() {
    let m = Money::parse("10.50", usd()).unwrap();
    assert_eq!(m.amount().units(), 1050);

    let m_neg = Money::parse("-0.5", usd()).unwrap();
    assert_eq!(m_neg.amount().units(), -50);

    // Rejects excessive scale
    let err = Money::parse("10.123", usd());
    assert!(matches!(err, Err(MoneyError::InvalidScale { allowed: 2 })));

    // Robust edge case handling
    assert!(Money::parse("-", usd()).is_err());
    assert!(Money::parse(".", usd()).is_err());
    assert!(Money::parse("-.", usd()).is_err());
  }

  #[test]
  fn test_formatting() {
    let m1 = Money::new(Amount::new(1050), usd());
    assert_eq!(m1.to_string(), "10.5 USD");

    let m2 = Money::new(Amount::new(-5), usd());
    assert_eq!(m2.to_string(), "-0.05 USD");

    let m3 = Money::new(Amount::new(1000), jpy());
    assert_eq!(m3.to_string(), "1000 JPY"); // JPY has 0 exponent
  }
}
