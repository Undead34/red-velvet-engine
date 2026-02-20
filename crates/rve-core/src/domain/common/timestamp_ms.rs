use serde::{Deserialize, Serialize};

use crate::domain::DomainError;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(try_from = "u64", into = "u64")]
pub struct TimestampMs(u64);

impl TimestampMs {
  pub fn new(value: u64) -> Result<Self, DomainError> {
    if value == 0 { Err(DomainError::InvalidTimestampMs(value)) } else { Ok(Self(value)) }
  }

  pub fn as_u64(self) -> u64 {
    self.0
  }
}

impl TryFrom<u64> for TimestampMs {
  type Error = DomainError;

  fn try_from(value: u64) -> Result<Self, Self::Error> {
    Self::new(value)
  }
}

impl From<TimestampMs> for u64 {
  fn from(value: TimestampMs) -> Self {
    value.0
  }
}
