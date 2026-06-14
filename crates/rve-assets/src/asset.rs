use iso4217_catalog::{CURRENCY_CODES, CurrencyCode, CurrencyStatus};

use rve_crypto::crypto::{self, CryptoAssetMetadata};
use rve_money::AssetId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AssetKind {
  Fiat,
  Crypto,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AssetStatus {
  Active,
  Deprecated,
  Disabled,
  Experimental,
}

impl From<CurrencyStatus> for AssetStatus {
  fn from(value: CurrencyStatus) -> Self {
    match value {
      CurrencyStatus::Active => AssetStatus::Active,
      CurrencyStatus::Testing => AssetStatus::Experimental,
      CurrencyStatus::Metal => AssetStatus::Active,
      CurrencyStatus::NoCurrency => AssetStatus::Disabled,
    }
  }
}

impl From<rve_crypto::crypto::AssetStatus> for AssetStatus {
  fn from(value: rve_crypto::crypto::AssetStatus) -> Self {
    match value {
      rve_crypto::crypto::AssetStatus::Active => AssetStatus::Active,
      rve_crypto::crypto::AssetStatus::Deprecated => AssetStatus::Deprecated,
      rve_crypto::crypto::AssetStatus::Disabled => AssetStatus::Disabled,
      rve_crypto::crypto::AssetStatus::Experimental => AssetStatus::Experimental,
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AssetMetadata {
  pub code: &'static str,
  pub kind: AssetKind,
  pub name: &'static str,
  pub symbol: Option<&'static str>,
  pub exponent: u8,
  pub status: AssetStatus,
  pub network: Option<crypto::SettlementNetwork>,
}

pub trait AssetIdExt {
  fn kind(&self) -> AssetKind;
  fn metadata(&self) -> AssetMetadata;
}

impl AssetIdExt for AssetId {
  fn kind(&self) -> AssetKind {
    match self {
      AssetId::Fiat(_) => AssetKind::Fiat,
      AssetId::Crypto(_) => AssetKind::Crypto,
    }
  }

  fn metadata(&self) -> AssetMetadata {
    match self {
      AssetId::Fiat(fiat) => AssetMetadata {
        code: fiat.alpha(),
        kind: AssetKind::Fiat,
        name: fiat.name(),
        symbol: Some(fiat.alpha()),
        exponent: fiat.exponent(),
        status: fiat.status().into(),
        network: None,
      },
      AssetId::Crypto(id) => asset_metadata_from_crypto(crypto::lookup_crypto_metadata(id)),
    }
  }
}

pub fn supported_fiat_codes() -> &'static [CurrencyCode] {
  CURRENCY_CODES
}

pub type Currency = AssetId;
pub type CurrencyError = rve_money::AssetIdError;

// helper to convert metadata from crypto module
pub(crate) fn asset_metadata_from_crypto(meta: CryptoAssetMetadata) -> AssetMetadata {
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
