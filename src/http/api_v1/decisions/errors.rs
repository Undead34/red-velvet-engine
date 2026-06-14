use thiserror::Error;

use rve_core::domain::DomainError;

#[derive(Debug, Error)]
pub enum DecisionPayloadError {
  #[error("{0}")]
  Invalid(String),
  #[error(transparent)]
  Domain(#[from] DomainError),
}
