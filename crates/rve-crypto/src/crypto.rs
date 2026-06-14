use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

pub use crate::error::{CryptoAssetError, SettlementNetworkError};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SettlementNetwork {
  Bitcoin,
  Ethereum,
  Tron,
  Solana,
  Polygon,
}

impl fmt::Display for SettlementNetwork {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let value = match self {
      SettlementNetwork::Bitcoin => "bitcoin",
      SettlementNetwork::Ethereum => "ethereum",
      SettlementNetwork::Tron => "tron",
      SettlementNetwork::Solana => "solana",
      SettlementNetwork::Polygon => "polygon",
    };
    write!(f, "{}", value)
  }
}

impl FromStr for SettlementNetwork {
  type Err = SettlementNetworkError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_ascii_lowercase().as_str() {
      "bitcoin" | "btc" => Ok(SettlementNetwork::Bitcoin),
      "ethereum" | "eth" => Ok(SettlementNetwork::Ethereum),
      "tron" | "trx" => Ok(SettlementNetwork::Tron),
      "solana" | "sol" => Ok(SettlementNetwork::Solana),
      "polygon" | "matic" => Ok(SettlementNetwork::Polygon),
      _ => Err(SettlementNetworkError(s.to_owned())),
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AssetStatus {
  Active,
  Deprecated,
  Disabled,
  Experimental,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CryptoAssetId {
  pub code: &'static str,
  pub network: Option<SettlementNetwork>,
}

#[derive(Clone, Copy, Debug)]
pub struct CryptoAssetMetadata {
  pub code: &'static str,
  pub name: &'static str,
  pub symbol: &'static str,
  pub exponent: u8,
  pub status: AssetStatus,
  pub network: Option<SettlementNetwork>,
}

const CRYPTO_ASSETS: &[CryptoAssetMetadata] = &[
  CryptoAssetMetadata {
    code: "BTC",
    name: "Bitcoin",
    symbol: "₿",
    exponent: 8,
    status: AssetStatus::Active,
    network: Some(SettlementNetwork::Bitcoin),
  },
  CryptoAssetMetadata {
    code: "ETH",
    name: "Ethereum",
    symbol: "Ξ",
    exponent: 18,
    status: AssetStatus::Active,
    network: Some(SettlementNetwork::Ethereum),
  },
  CryptoAssetMetadata {
    code: "USDC",
    name: "USD Coin",
    symbol: "USDC",
    exponent: 6,
    status: AssetStatus::Active,
    network: Some(SettlementNetwork::Ethereum),
  },
  CryptoAssetMetadata {
    code: "USDC",
    name: "USD Coin",
    symbol: "USDC",
    exponent: 6,
    status: AssetStatus::Active,
    network: Some(SettlementNetwork::Solana),
  },
  CryptoAssetMetadata {
    code: "USDT",
    name: "Tether USD",
    symbol: "USDT",
    exponent: 6,
    status: AssetStatus::Active,
    network: Some(SettlementNetwork::Ethereum),
  },
  CryptoAssetMetadata {
    code: "USDT",
    name: "Tether USD",
    symbol: "USDT",
    exponent: 6,
    status: AssetStatus::Active,
    network: Some(SettlementNetwork::Tron),
  },
];

pub fn parse_crypto_id(code: &str, network: Option<&str>) -> Result<CryptoAssetId, CryptoAssetError> {
  let normalized = code.to_ascii_uppercase();
  let entries: Vec<_> = CRYPTO_ASSETS.iter().filter(|meta| meta.code == normalized).collect();
  if entries.is_empty() {
    return Err(CryptoAssetError::UnknownCode { code: code.to_owned() });
  }

  let parsed_network = match network {
    Some(value) => Some(SettlementNetwork::from_str(value).map_err(|_| {
      CryptoAssetError::UnknownNetwork {
        code: code.to_owned(),
        network: value.to_owned(),
      }
    })?),
    None => None,
  };

  let selected = match parsed_network {
    Some(network) => entries.into_iter().find(|meta| meta.network == Some(network)).ok_or_else(
      || CryptoAssetError::UnknownNetwork {
        code: code.to_owned(),
        network: network.to_string(),
      },
    )?,
    None => {
      if entries.len() == 1 {
        entries[0]
      } else {
        return Err(CryptoAssetError::UnknownNetwork {
          code: code.to_owned(),
          network: String::new(),
        });
      }
    }
  };

  Ok(CryptoAssetId { code: selected.code, network: selected.network })
}

pub fn lookup_crypto_metadata(id: &CryptoAssetId) -> CryptoAssetMetadata {
  CRYPTO_ASSETS
    .iter()
    .find(|meta| meta.code == id.code && meta.network == id.network)
    .copied()
    .unwrap_or_else(|| CryptoAssetMetadata {
      code: id.code,
      name: id.code,
      symbol: id.code,
      exponent: 8,
      status: AssetStatus::Experimental,
      network: id.network,
    })
}

pub fn supported_crypto_assets() -> &'static [CryptoAssetMetadata] {
  CRYPTO_ASSETS
}
