use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::domain::common::Money;

use super::Parties;

/// Monetary and participant data used in rule logic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payload {
  pub money: Money,
  pub parties: Parties,
  pub extensions: BTreeMap<String, Value>,
}
