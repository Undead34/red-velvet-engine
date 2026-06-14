use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during `Amount` mathematical operations.
#[derive(Error, Debug, PartialEq, Eq, Clone, Copy)]
pub enum AmountError {
  /// Indicates that an operation would result in an arithmetic overflow or underflow.
  #[error("mathematical operation caused an overflow")]
  Overflow,

  /// Indicates an attempt to divide by zero.
  #[error("attempted to divide by zero")]
  DivideByZero,
}

/// `Amount` is a pure mathematical abstraction of minor units.
/// It has no concept of currency, decimals, or exponents.
/// It safely wraps an `i128` and provides secure mathematical operations.
#[derive(
  Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct Amount {
  units: i128,
}

impl Amount {
  /// Creates a new amount from minor units (e.g., cents).
  #[inline]
  #[must_use]
  pub const fn new(units: i128) -> Self {
    Self { units }
  }

  /// Creates an amount with a value of strictly zero.
  #[inline]
  #[must_use]
  pub const fn zero() -> Self {
    Self { units: 0 }
  }

  /// Returns the underlying internal units.
  #[inline]
  #[must_use]
  pub const fn units(&self) -> i128 {
    self.units
  }

  /// Checks if the amount is exactly zero.
  #[inline]
  #[must_use]
  pub const fn is_zero(&self) -> bool {
    self.units == 0
  }

  /// Checks if the amount is strictly negative.
  #[inline]
  #[must_use]
  pub const fn is_negative(&self) -> bool {
    self.units < 0
  }

  /// Checks if the amount is strictly positive (greater than zero).
  #[inline]
  #[must_use]
  pub const fn is_positive(&self) -> bool {
    self.units > 0
  }

  /// Returns the absolute value of the amount.
  /// Fails with an overflow error if the value is `i128::MIN`.
  pub fn abs(self) -> Result<Self, AmountError> {
    self.units.checked_abs().map(Self::new).ok_or(AmountError::Overflow)
  }

  /// Performs secure addition preventing overflow.
  pub fn checked_add(self, other: Self) -> Result<Self, AmountError> {
    self.units.checked_add(other.units).map(Self::new).ok_or(AmountError::Overflow)
  }

  /// Performs secure subtraction preventing underflow/overflow.
  pub fn checked_sub(self, other: Self) -> Result<Self, AmountError> {
    self.units.checked_sub(other.units).map(Self::new).ok_or(AmountError::Overflow)
  }

  /// Performs secure multiplication by a scalar (standard integer).
  /// Useful for operations like "triple this amount".
  pub fn checked_mul(self, scalar: i128) -> Result<Self, AmountError> {
    self.units.checked_mul(scalar).map(Self::new).ok_or(AmountError::Overflow)
  }

  /// Performs secure integer division.
  /// Fails if attempting to divide by zero or on overflow (e.g., `i128::MIN / -1`).
  pub fn checked_div(self, scalar: i128) -> Result<Self, AmountError> {
    if scalar == 0 {
      return Err(AmountError::DivideByZero);
    }
    self.units.checked_div(scalar).map(Self::new).ok_or(AmountError::Overflow)
  }

  /// Computes the remainder of a secure division.
  /// Highly useful in financial software for handling leftover fractions (pennies).
  pub fn checked_rem(self, scalar: i128) -> Result<Self, AmountError> {
    if scalar == 0 {
      return Err(AmountError::DivideByZero);
    }
    self.units.checked_rem(scalar).map(Self::new).ok_or(AmountError::Overflow)
  }
}

// A clean macro to safely implement lossless conversions from smaller integers.
macro_rules! impl_from_int {
    ($($type:ty),*) => {
        $(
            impl From<$type> for Amount {
                #[inline]
                fn from(value: $type) -> Self {
                    Self::new(i128::from(value))
                }
            }
        )*
    };
}

impl_from_int!(i8, i16, i32, i64);

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_basic_operations() {
    let a = Amount::new(100);
    let b = Amount::new(50);

    assert_eq!(a.checked_add(b).unwrap(), Amount::new(150));
    assert_eq!(a.checked_sub(b).unwrap(), Amount::new(50));
    assert_eq!(b.checked_mul(3).unwrap(), Amount::new(150));
    assert_eq!(a.checked_div(2).unwrap(), Amount::new(50));
  }

  #[test]
  fn test_division_by_zero() {
    let a = Amount::new(100);
    assert_eq!(a.checked_div(0), Err(AmountError::DivideByZero));
    assert_eq!(a.checked_rem(0), Err(AmountError::DivideByZero));
  }

  #[test]
  fn test_overflow_and_underflow() {
    let max = Amount::new(i128::MAX);
    let min = Amount::new(i128::MIN);
    let one = Amount::new(1);

    assert_eq!(max.checked_add(one), Err(AmountError::Overflow));
    assert_eq!(min.checked_sub(one), Err(AmountError::Overflow));

    // i128::MIN absolute value causes overflow because max positive is MIN absolute - 1
    assert_eq!(min.abs(), Err(AmountError::Overflow));

    // i128::MIN / -1 causes overflow
    assert_eq!(min.checked_div(-1), Err(AmountError::Overflow));
  }

  #[test]
  fn test_traits_and_defaults() {
    assert_eq!(Amount::default(), Amount::zero());
    assert_eq!(Amount::from(42_i32), Amount::new(42));
    assert_eq!(Amount::from(-100_i64), Amount::new(-100));
  }
}
