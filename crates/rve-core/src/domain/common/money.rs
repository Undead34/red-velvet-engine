use serde::{Deserialize, Serialize};

use super::codes::Currency;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Money {
  pub value: f64,
  pub ccy: Currency,
}
