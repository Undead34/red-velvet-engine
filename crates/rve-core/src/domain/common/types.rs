use serde::{Deserialize, Serialize};

pub type CountryCode = String;
pub type BankRef = String;
pub type EntityType = String;
pub type KycLevel = String;

pub type Currency = String;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Money {
  pub value: f64,
  pub ccy: Currency,
}
