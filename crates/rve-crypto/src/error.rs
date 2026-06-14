use thiserror::Error;

/// Error parsing an unknown settlement network string.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("unknown settlement network: {0}")]
pub struct SettlementNetworkError(pub String);

/// Error resolving a crypto asset by code or network.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CryptoError {
  /// No crypto asset matches the given code.
  #[error("unknown crypto asset code: {code}")]
  UnknownCode { code: String },
  /// The code is valid but the network suffix does not match any entry.
  #[error("unknown crypto asset network `{network}` for {code}")]
  UnknownNetwork { code: String, network: String },
}
