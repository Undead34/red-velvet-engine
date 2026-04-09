use serde::{Deserialize, Serialize};

/// Metadata describing a supported cryptoasset.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CryptoAsset {
  pub code: &'static str,
  pub name: &'static str,
  pub symbol: &'static str,
  pub exponent: u8,
}

impl CryptoAsset {
  #[must_use]
  pub fn minor_unit_name(&self) -> &'static str {
    match self.code {
      "BTC" => "satoshi",
      "ETH" => "wei",
      _ => "unit",
    }
  }
}

const SUPPORTED: &[CryptoAsset] = &[
  CryptoAsset { code: "BTC", name: "Bitcoin", symbol: "₿", exponent: 8 },
  CryptoAsset { code: "ETH", name: "Ethereum", symbol: "Ξ", exponent: 18 },
  CryptoAsset { code: "USDC", name: "USD Coin", symbol: "USDC", exponent: 6 },
  CryptoAsset { code: "USDT", name: "Tether USD", symbol: "USDT", exponent: 6 },
];

#[must_use]
pub fn supported_crypto_assets() -> &'static [CryptoAsset] {
  SUPPORTED
}

#[must_use]
pub fn find_crypto_asset(code: &str) -> Option<&'static CryptoAsset> {
  let upper = code.to_ascii_uppercase();
  SUPPORTED.iter().find(|asset| asset.code == upper)
}
