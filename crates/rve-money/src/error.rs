use thiserror::Error;

use crate::currency::Currency;

/// Comprehensive error type for all monetary and mathematical operations.
#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum MoneyError {
  /// Indicates that a mathematical operation resulted in an overflow or underflow.
  #[error("mathematical operation caused an overflow")]
  Overflow,

  /// Indicates an attempt to divide an amount by zero.
  #[error("attempted to divide by zero")]
  DivideByZero,

  /// Indicates an attempt to operate on or compare two different currencies.
  /// This is a critical business logic safeguard.
  #[error("currency mismatch: expected {expected}, found {found}")]
  CurrencyMismatch { expected: Currency, found: Currency },

  /// Indicates that the provided string could not be parsed into a monetary value.
  #[error("invalid monetary format: '{0}'")]
  InvalidFormat(String),

  /// Indicates that the string contains more decimal places than the currency allows.
  #[error(
    "invalid scale: provided decimal places exceed the currency allowed exponent of {allowed}"
  )]
  InvalidScale { allowed: u8 },
}
