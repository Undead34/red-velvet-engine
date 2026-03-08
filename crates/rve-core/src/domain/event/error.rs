use thiserror::Error;

/// Errors produced by event domain validation.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EventError {
  #[error(transparent)]
  Geo(#[from] EventGeoError),
  #[error(transparent)]
  Party(#[from] EventPartyError),
  #[error(transparent)]
  Features(#[from] EventFeaturesError),
}

/// Errors for geographic snapshot validation.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EventGeoError {
  #[error("invalid geo latitude: {value}; expected -90..=90")]
  InvalidLatitude { value: String },
  #[error("invalid geo longitude: {value}; expected -180..=180")]
  InvalidLongitude { value: String },
}

/// Errors for party validation.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EventPartyError {
  #[error("invalid sanctions_score: {value}; expected 0.0..=1.0")]
  InvalidSanctionsScore { value: String },
}

/// Errors for feature-set validation.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EventFeaturesError {
  #[error("invalid feature chronology: first_seen_at ({first_seen_at}) must be <= last_seen_at ({last_seen_at})")]
  InvalidSeenChronology { first_seen_at: u64, last_seen_at: u64 },
  #[error("invalid feature chronology: last_declined_at ({last_declined_at}) must be >= first_seen_at ({first_seen_at})")]
  InvalidLastDeclinedChronology { first_seen_at: u64, last_declined_at: u64 },
}
