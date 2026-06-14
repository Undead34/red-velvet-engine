use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::SettlementNetworkError;

/// Supported blockchain settlement networks for crypto assets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SettlementNetwork {
  Bitcoin,
  Ethereum,
  Tron,
  Solana,
  Polygon,
}

impl SettlementNetwork {
  /// All known network variants.
  pub const ALL: &'static [SettlementNetwork] = &[
    SettlementNetwork::Bitcoin,
    SettlementNetwork::Ethereum,
    SettlementNetwork::Tron,
    SettlementNetwork::Solana,
    SettlementNetwork::Polygon,
  ];
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_valid() {
    assert_eq!("bitcoin".parse::<SettlementNetwork>().unwrap(), SettlementNetwork::Bitcoin);
    assert_eq!("btc".parse::<SettlementNetwork>().unwrap(), SettlementNetwork::Bitcoin);
    assert_eq!("ETH".parse::<SettlementNetwork>().unwrap(), SettlementNetwork::Ethereum);
    assert_eq!("sol".parse::<SettlementNetwork>().unwrap(), SettlementNetwork::Solana);
  }

  #[test]
  fn test_parse_invalid() {
    assert!("dogecoin".parse::<SettlementNetwork>().is_err());
  }

  #[test]
  fn test_display() {
    assert_eq!(SettlementNetwork::Bitcoin.to_string(), "bitcoin");
    assert_eq!(SettlementNetwork::Ethereum.to_string(), "ethereum");
  }

  #[test]
  fn test_all_contains_all() {
    for &variant in SettlementNetwork::ALL {
      let s = variant.to_string();
      assert_eq!(s.parse::<SettlementNetwork>().unwrap(), variant);
    }
  }
}
