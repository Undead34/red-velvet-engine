use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DomainError {
  #[error("invalid country code: {0}")]
  InvalidCountryCode(String),
  #[error("invalid currency code: {0}")]
  InvalidCurrencyCode(String),
  #[error("invalid account id: {0}")]
  InvalidAccountId(String),
  #[error("invalid bank reference: {0}")]
  InvalidBankRef(String),
  #[error("invalid event source: {0}")]
  InvalidEventSource(String),
  #[error("invalid event id: {0}")]
  InvalidEventId(String),
  #[error("invalid instrument: {0}")]
  InvalidInstrument(String),
  #[error("invalid channel: {0}")]
  InvalidChannel(String),
  #[error("invalid device id: {0}")]
  InvalidDeviceId(String),
  #[error("invalid session id: {0}")]
  InvalidSessionId(String),
  #[error("invalid kyc level: {0}")]
  InvalidKycLevel(String),
  #[error("invalid locale tag: {0}")]
  InvalidLocaleTag(String),
  #[error("invalid timezone name: {0}")]
  InvalidTimezoneName(String),
  #[error("invalid user agent: {0}")]
  InvalidUserAgent(String),
  #[error("invalid rule id: {0}")]
  InvalidRuleId(String),
  #[error("invalid timestamp (ms): {0}")]
  InvalidTimestampMs(u64),
}
