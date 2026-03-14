use std::str::FromStr;

use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

use super::validation::{is_valid_kyc_level, is_valid_locale_tag, is_valid_user_agent};
use crate::domain::DomainError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(try_from = "String", into = "String")]
pub struct CountryCode(String);

impl CountryCode {
  pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
    let value = value.into();
    if value.len() == 2 && value.chars().all(|c| c.is_ascii_uppercase()) {
      Ok(Self(value))
    } else {
      Err(DomainError::InvalidCountryCode(value))
    }
  }

  pub fn as_str(&self) -> &str {
    &self.0
  }
}

impl TryFrom<String> for CountryCode {
  type Error = DomainError;
  fn try_from(value: String) -> Result<Self, Self::Error> {
    Self::new(value)
  }
}

impl From<CountryCode> for String {
  fn from(value: CountryCode) -> Self {
    value.0
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(try_from = "String", into = "String")]
pub struct KycLevel(String);

impl KycLevel {
  pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
    let value = value.into();
    if is_valid_kyc_level(&value) {
      Ok(Self(value))
    } else {
      Err(DomainError::InvalidKycLevel(value))
    }
  }

  pub fn as_str(&self) -> &str {
    &self.0
  }
}

impl TryFrom<String> for KycLevel {
  type Error = DomainError;
  fn try_from(value: String) -> Result<Self, Self::Error> {
    Self::new(value)
  }
}

impl From<KycLevel> for String {
  fn from(value: KycLevel) -> Self {
    value.0
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(try_from = "String", into = "String")]
pub struct LocaleTag(String);

impl LocaleTag {
  pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
    let value = value.into();
    if is_valid_locale_tag(&value) {
      Ok(Self(value))
    } else {
      Err(DomainError::InvalidLocaleTag(value))
    }
  }
}

impl TryFrom<String> for LocaleTag {
  type Error = DomainError;
  fn try_from(value: String) -> Result<Self, Self::Error> {
    Self::new(value)
  }
}

impl From<LocaleTag> for String {
  fn from(value: LocaleTag) -> Self {
    value.0
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(try_from = "String", into = "String")]
pub struct TimezoneName(String);

impl TimezoneName {
  pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
    let value = value.into();
    if Tz::from_str(&value).is_ok() {
      Ok(Self(value))
    } else {
      Err(DomainError::InvalidTimezoneName(value))
    }
  }
}

impl TryFrom<String> for TimezoneName {
  type Error = DomainError;
  fn try_from(value: String) -> Result<Self, Self::Error> {
    Self::new(value)
  }
}

impl From<TimezoneName> for String {
  fn from(value: TimezoneName) -> Self {
    value.0
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(try_from = "String", into = "String")]
pub struct UserAgent(String);

impl UserAgent {
  pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
    let value = value.into();
    if is_valid_user_agent(&value) {
      Ok(Self(value))
    } else {
      Err(DomainError::InvalidUserAgent(value))
    }
  }
}

impl TryFrom<String> for UserAgent {
  type Error = DomainError;
  fn try_from(value: String) -> Result<Self, Self::Error> {
    Self::new(value)
  }
}

impl From<UserAgent> for String {
  fn from(value: UserAgent) -> Self {
    value.0
  }
}
