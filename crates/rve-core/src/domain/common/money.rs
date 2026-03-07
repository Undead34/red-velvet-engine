use serde::{Deserialize, Serialize};

use super::codes::Currency;
use crate::domain::DomainError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Money {
  pub value: f64,
  pub ccy: Currency,
}

impl Money {
  pub fn new(value: f64, ccy: Currency) -> Result<Self, DomainError> {
    if !value.is_finite() {
      return Err(DomainError::InvalidMoneyAmount("value must be finite".to_owned()));
    }
    Ok(Self { value, ccy })
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoneyMinor {
  /// Monetary units in the smallest fraction of the currency.
  pub minor_units: i64,
  pub ccy: Currency,
}

impl MoneyMinor {
  pub fn new(minor_units: i64, ccy: Currency) -> Result<Self, DomainError> {
    if minor_units < 0 {
      return Err(DomainError::InvalidMoneyAmount("minor_units must be non-negative".to_owned()));
    }
    Ok(Self { minor_units, ccy })
  }

  pub fn to_money_f64(&self) -> f64 {
    self.minor_units as f64 / 100.0
  }
}
