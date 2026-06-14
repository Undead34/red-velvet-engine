use thiserror::Error;

/// Result alias for `rve-crypto` operations.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Top-level error covering all crypto operations.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum Error {
  #[error(transparent)]
  SettlementNetwork(#[from] SettlementNetworkError),
  #[error(transparent)]
  CryptoAsset(#[from] CryptoAssetError),
}

/// Error parsing an unknown settlement network string.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("unknown settlement network: {0}")]
pub struct SettlementNetworkError(pub String);

/// Error resolving a crypto asset by code or network.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CryptoAssetError {
  /// No crypto asset matches the given code.
  #[error("unknown crypto asset code: {code}")]
  UnknownCode {
    code: String,
  },
  /// The code is valid but the network suffix does not match any entry.
  #[error("unknown crypto asset network `{network}` for {code}")]
  UnknownNetwork {
    code: String,
    network: String,
  },
}
