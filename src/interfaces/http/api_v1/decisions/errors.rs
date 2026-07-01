use thiserror::Error;

#[derive(Debug, Error)]
pub enum DecisionPayloadError {
  #[error("{0}")]
  Invalid(String),
}
