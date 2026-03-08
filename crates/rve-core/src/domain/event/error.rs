use thiserror::Error;

/// Errors produced by event domain validation.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EventError {
  /// Error in geographic snapshot validation.
  #[error(transparent)]
  Geo(#[from] EventGeoError),
  /// Error in party validation.
  #[error(transparent)]
  Party(#[from] EventPartyError),
  /// Error in feature validation.
  #[error(transparent)]
  Features(#[from] EventFeaturesError),
}

/// Errors for geographic snapshot validation.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EventGeoError {
  /// Latitude is outside `-90..=90` or non-finite.
  #[error("invalid geo latitude: {value}; expected -90..=90")]
  InvalidLatitude { value: String },
  /// Longitude is outside `-180..=180` or non-finite.
  #[error("invalid geo longitude: {value}; expected -180..=180")]
  InvalidLongitude { value: String },
}

/// Errors for party validation.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EventPartyError {
  /// Sanctions score is outside `0.0..=1.0` or non-finite.
  #[error("invalid sanctions_score: {value}; expected 0.0..=1.0")]
  InvalidSanctionsScore { value: String },
}

/// Errors for feature-set validation.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EventFeaturesError {
  /// `first_seen_at` is later than `last_seen_at`.
  #[error(
    "invalid feature chronology: first_seen_at ({first_seen_at}) must be <= last_seen_at ({last_seen_at})"
  )]
  InvalidSeenChronology { first_seen_at: u64, last_seen_at: u64 },
  /// `last_declined_at` is earlier than `first_seen_at`.
  #[error(
    "invalid feature chronology: last_declined_at ({last_declined_at}) must be >= first_seen_at ({first_seen_at})"
  )]
  InvalidLastDeclinedChronology { first_seen_at: u64, last_declined_at: u64 },
}
