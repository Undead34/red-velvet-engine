use iso4217_catalog::{CURRENCY_CODES, CurrencyCode, CurrencyStatus};

use rve_crypto::{CryptoToken, SettlementNetwork, TokenStatus};
use rve_money::Currency;

/// Kind of asset (fiat currency vs crypto token).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AssetKind {
  Fiat,
  Crypto,
}

/// Operational status of an asset.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Status {
  Active,
  Deprecated,
  Disabled,
  Experimental,
}

impl From<CurrencyStatus> for Status {
  fn from(value: CurrencyStatus) -> Self {
    match value {
      CurrencyStatus::Active => Status::Active,
      CurrencyStatus::Testing => Status::Experimental,
      CurrencyStatus::Metal => Status::Active,
      CurrencyStatus::NoCurrency => Status::Disabled,
    }
  }
}

impl From<TokenStatus> for Status {
  fn from(value: TokenStatus) -> Self {
    match value {
      TokenStatus::Active => Status::Active,
      TokenStatus::Deprecated => Status::Deprecated,
      TokenStatus::Disabled => Status::Disabled,
      TokenStatus::Experimental => Status::Experimental,
    }
  }
}

/// Error type for fiat currency code validation.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid fiat currency code: {0}")]
pub struct FiatCurrencyError(pub String);

/// Errors that can occur when working with [`AssetId`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AssetIdError {
  /// Invalid or unknown fiat currency code.
  #[error("invalid fiat currency code: {0}")]
  Fiat(FiatCurrencyError),
  /// Unknown crypto token code.
  #[error("unknown crypto token: {code}")]
  UnknownCrypto { code: String },
  /// Unknown or mismatched settlement network for a crypto token.
  #[error("unknown network for token `{code}`: {network}")]
  UnknownNetwork { code: String, network: String },
}

/// Unified asset identifier — either a fiat currency or a crypto token.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AssetId {
  Fiat(Currency),
  Crypto(CryptoToken),
}

impl AssetId {
  /// What kind of asset this is.
  #[must_use]
  pub fn kind(&self) -> AssetKind {
    match self {
      AssetId::Fiat(_) => AssetKind::Fiat,
      AssetId::Crypto(_) => AssetKind::Crypto,
    }
  }

  /// Normalized code (e.g. `"USD"`, `"BTC"`).
  #[must_use]
  pub fn code(&self) -> &'static str {
    match self {
      AssetId::Fiat(currency) => currency.alpha(),
      AssetId::Crypto(token) => token.code(),
    }
  }

  /// Minor-unit exponent (e.g. `2` for USD, `8` for BTC).
  #[must_use]
  pub fn exponent(&self) -> u8 {
    match self {
      AssetId::Fiat(currency) => currency.exponent(),
      AssetId::Crypto(token) => token.exponent(),
    }
  }

  /// Full unified metadata for this asset.
  #[must_use]
  pub fn metadata(&self) -> AssetMetadata {
    match self {
      AssetId::Fiat(currency) => AssetMetadata {
        code: currency.alpha(),
        kind: AssetKind::Fiat,
        name: currency.name(),
        symbol: Some(currency.alpha()),
        exponent: currency.exponent(),
        status: currency.status().into(),
        network: None,
      },
      AssetId::Crypto(token) => {
        let meta = token.metadata();
        AssetMetadata {
          code: meta.code,
          kind: AssetKind::Crypto,
          name: meta.name,
          symbol: Some(meta.symbol),
          exponent: meta.exponent,
          status: meta.status.into(),
          network: meta.network,
        }
      }
    }
  }
}

impl From<Currency> for AssetId {
  fn from(currency: Currency) -> Self {
    AssetId::Fiat(currency)
  }
}

impl From<CryptoToken> for AssetId {
  fn from(token: CryptoToken) -> Self {
    AssetId::Crypto(token)
  }
}

/// Unified metadata describing any asset.
#[derive(Clone, Copy, Debug)]
pub struct AssetMetadata {
  pub code: &'static str,
  pub kind: AssetKind,
  pub name: &'static str,
  pub symbol: Option<&'static str>,
  pub exponent: u8,
  pub status: Status,
  pub network: Option<SettlementNetwork>,
}

/// All known ISO 4217 currency codes.
#[must_use]
pub fn supported_fiat_codes() -> &'static [CurrencyCode] {
  CURRENCY_CODES
}
