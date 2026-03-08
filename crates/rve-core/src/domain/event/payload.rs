use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::domain::common::Money;

use super::Parties;

/// Business payload used in rule evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payload {
  /// Transaction amount and currency.
  pub money: Money,
  /// Originator/beneficiary data.
  pub parties: Parties,
  /// Free-form extensions passed to rules.
  pub extensions: BTreeMap<String, Value>,
}
