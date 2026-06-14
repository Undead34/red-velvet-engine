use std::str::FromStr;

use crate::error::CryptoError;
use crate::network::SettlementNetwork;

/// Operational status of a crypto token.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TokenStatus {
  Active,
  Deprecated,
  Disabled,
  Experimental,
}

/// A crypto token identifier with code and optional settlement network.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CryptoToken {
  code: &'static str,
  network: Option<SettlementNetwork>,
}

impl CryptoToken {
  /// Parse a crypto token from a code and optional network string.
  ///
  /// Returns `UnknownCode` if the code is not recognized, or `UnknownNetwork`
  /// if the code is valid but the network does not match any entry.
  pub fn parse(code: &str, network: Option<&str>) -> Result<Self, CryptoError> {
    let normalized = code.to_ascii_uppercase();
    let entries: Vec<_> = KNOWN_TOKENS.iter().filter(|meta| meta.code == normalized).collect();

    if entries.is_empty() {
      return Err(CryptoError::UnknownCode { code: code.to_owned() });
    }

    let parsed_network = match network {
      Some(value) => Some(SettlementNetwork::from_str(value).map_err(|_| {
        CryptoError::UnknownNetwork { code: code.to_owned(), network: value.to_owned() }
      })?),
      None => None,
    };

    let selected = match parsed_network {
      Some(network) => {
        entries.into_iter().find(|meta| meta.network == Some(network)).ok_or_else(|| {
          CryptoError::UnknownNetwork { code: code.to_owned(), network: network.to_string() }
        })?
      }
      None => {
        if entries.len() == 1 {
          entries[0]
        } else {
          return Err(CryptoError::UnknownNetwork {
            code: code.to_owned(),
            network: String::new(),
          });
        }
      }
    };

    Ok(CryptoToken { code: selected.code, network: selected.network })
  }

  /// Three-letter uppercase code (e.g. `"BTC"`, `"ETH"`).
  #[must_use]
  pub fn code(&self) -> &'static str {
    self.code
  }

  /// Optional settlement network for multi-chain tokens.
  #[must_use]
  pub fn network(&self) -> Option<SettlementNetwork> {
    self.network
  }

  /// Minor-unit exponent (e.g. `8` for BTC, `18` for ETH).
  #[must_use]
  pub fn exponent(&self) -> u8 {
    self.metadata().exponent
  }

  /// Full metadata for this token, falling back to reasonable defaults.
  #[must_use]
  pub fn metadata(&self) -> CryptoTokenMetadata {
    KNOWN_TOKENS
      .iter()
      .find(|meta| meta.code == self.code && meta.network == self.network)
      .copied()
      .unwrap_or_else(|| CryptoTokenMetadata {
        code: self.code,
        name: self.code,
        symbol: self.code,
        exponent: 8,
        status: TokenStatus::Experimental,
        network: self.network,
      })
  }
}

/// Full metadata describing a crypto token.
#[derive(Clone, Copy, Debug)]
pub struct CryptoTokenMetadata {
  pub code: &'static str,
  pub name: &'static str,
  pub symbol: &'static str,
  pub exponent: u8,
  pub status: TokenStatus,
  pub network: Option<SettlementNetwork>,
}

const KNOWN_TOKENS: &[CryptoTokenMetadata] = &[
  CryptoTokenMetadata {
    code: "BTC",
    name: "Bitcoin",
    symbol: "₿",
    exponent: 8,
    status: TokenStatus::Active,
    network: Some(SettlementNetwork::Bitcoin),
  },
  CryptoTokenMetadata {
    code: "ETH",
    name: "Ethereum",
    symbol: "Ξ",
    exponent: 18,
    status: TokenStatus::Active,
    network: Some(SettlementNetwork::Ethereum),
  },
  CryptoTokenMetadata {
    code: "USDC",
    name: "USD Coin",
    symbol: "USDC",
    exponent: 6,
    status: TokenStatus::Active,
    network: Some(SettlementNetwork::Ethereum),
  },
  CryptoTokenMetadata {
    code: "USDC",
    name: "USD Coin",
    symbol: "USDC",
    exponent: 6,
    status: TokenStatus::Active,
    network: Some(SettlementNetwork::Solana),
  },
  CryptoTokenMetadata {
    code: "USDT",
    name: "Tether USD",
    symbol: "USDT",
    exponent: 6,
    status: TokenStatus::Active,
    network: Some(SettlementNetwork::Ethereum),
  },
  CryptoTokenMetadata {
    code: "USDT",
    name: "Tether USD",
    symbol: "USDT",
    exponent: 6,
    status: TokenStatus::Active,
    network: Some(SettlementNetwork::Tron),
  },
];

/// All known crypto tokens with their metadata.
#[must_use]
pub fn supported_tokens() -> &'static [CryptoTokenMetadata] {
  KNOWN_TOKENS
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_btc() {
    let token = CryptoToken::parse("BTC", None).unwrap();
    assert_eq!(token.code(), "BTC");
    assert_eq!(token.network(), Some(SettlementNetwork::Bitcoin));
    assert_eq!(token.exponent(), 8);
  }

  #[test]
  fn test_parse_eth() {
    let token = CryptoToken::parse("ETH", None).unwrap();
    assert_eq!(token.code(), "ETH");
    assert_eq!(token.network(), Some(SettlementNetwork::Ethereum));
    assert_eq!(token.exponent(), 18);
  }

  #[test]
  fn test_parse_usdc_on_ethereum() {
    let token = CryptoToken::parse("USDC", Some("ethereum")).unwrap();
    assert_eq!(token.code(), "USDC");
    assert_eq!(token.network(), Some(SettlementNetwork::Ethereum));
  }

  #[test]
  fn test_parse_usdc_on_solana() {
    let token = CryptoToken::parse("USDC", Some("solana")).unwrap();
    assert_eq!(token.network(), Some(SettlementNetwork::Solana));
  }

  #[test]
  fn test_parse_unknown_code() {
    let err = CryptoToken::parse("ZZZ", None).unwrap_err();
    assert!(matches!(err, CryptoError::UnknownCode { .. }));
  }

  #[test]
  fn test_parse_unknown_network() {
    let err = CryptoToken::parse("USDC", Some("bitcoin")).unwrap_err();
    assert!(matches!(err, CryptoError::UnknownNetwork { .. }));
  }

  #[test]
  fn test_parse_ambiguous_without_network() {
    let err = CryptoToken::parse("USDC", None).unwrap_err();
    assert!(matches!(err, CryptoError::UnknownNetwork { .. }));
  }

  #[test]
  fn test_metadata_fallback() {
    let token = CryptoToken { code: "XXX", network: None };
    let meta = token.metadata();
    assert_eq!(meta.status, TokenStatus::Experimental);
    assert_eq!(meta.exponent, 8);
  }

  #[test]
  fn test_supported_tokens_not_empty() {
    assert!(!supported_tokens().is_empty());
  }
}
