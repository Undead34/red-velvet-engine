use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::validation::is_valid_identifier;
use crate::domain::DomainError;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct RuleId(Uuid);

impl RuleId {
  pub fn new_v7() -> Self {
    Self(Uuid::now_v7())
  }

  pub fn as_uuid(&self) -> &Uuid {
    &self.0
  }
}

impl TryFrom<String> for RuleId {
  type Error = DomainError;

  fn try_from(value: String) -> Result<Self, Self::Error> {
    Uuid::parse_str(&value).map(Self).map_err(|_| DomainError::InvalidRuleId(value))
  }
}

impl From<RuleId> for String {
  fn from(value: RuleId) -> Self {
    value.0.to_string()
  }
}

impl fmt::Display for RuleId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(try_from = "String", into = "String")]
pub struct AccountId(String);

impl AccountId {
  pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
    let value = value.into();
    if is_valid_identifier(&value) {
      Ok(Self(value))
    } else {
      Err(DomainError::InvalidAccountId(value))
    }
  }

  pub fn as_str(&self) -> &str {
    &self.0
  }
}

impl TryFrom<String> for AccountId {
  type Error = DomainError;
  fn try_from(value: String) -> Result<Self, Self::Error> {
    Self::new(value)
  }
}

impl From<AccountId> for String {
  fn from(value: AccountId) -> Self {
    value.0
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(try_from = "String", into = "String")]
pub struct BankRef(String);

impl BankRef {
  pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
    let value = value.into();
    if is_valid_identifier(&value) {
      Ok(Self(value))
    } else {
      Err(DomainError::InvalidBankRef(value))
    }
  }

  pub fn as_str(&self) -> &str {
    &self.0
  }
}

impl TryFrom<String> for BankRef {
  type Error = DomainError;
  fn try_from(value: String) -> Result<Self, Self::Error> {
    Self::new(value)
  }
}

impl From<BankRef> for String {
  fn from(value: BankRef) -> Self {
    value.0
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(try_from = "String", into = "String")]
pub struct DeviceId(String);

impl DeviceId {
  pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
    let value = value.into();
    if is_valid_identifier(&value) {
      Ok(Self(value))
    } else {
      Err(DomainError::InvalidDeviceId(value))
    }
  }
}

impl TryFrom<String> for DeviceId {
  type Error = DomainError;
  fn try_from(value: String) -> Result<Self, Self::Error> {
    Self::new(value)
  }
}

impl From<DeviceId> for String {
  fn from(value: DeviceId) -> Self {
    value.0
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(try_from = "String", into = "String")]
pub struct SessionId(String);

impl SessionId {
  pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
    let value = value.into();
    if is_valid_identifier(&value) {
      Ok(Self(value))
    } else {
      Err(DomainError::InvalidSessionId(value))
    }
  }
}

impl TryFrom<String> for SessionId {
  type Error = DomainError;
  fn try_from(value: String) -> Result<Self, Self::Error> {
    Self::new(value)
  }
}

impl From<SessionId> for String {
  fn from(value: SessionId) -> Self {
    value.0
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(try_from = "String", into = "String")]
pub struct EventId(String);

impl EventId {
  pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
    let value = value.into();
    if is_valid_identifier(&value) {
      Ok(Self(value))
    } else {
      Err(DomainError::InvalidEventId(value))
    }
  }
}

impl TryFrom<String> for EventId {
  type Error = DomainError;
  fn try_from(value: String) -> Result<Self, Self::Error> {
    Self::new(value)
  }
}

impl From<EventId> for String {
  fn from(value: EventId) -> Self {
    value.0
  }
}
