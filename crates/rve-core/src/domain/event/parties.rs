use serde::{Deserialize, Serialize};

use super::Party;

/// Pair of involved parties in a transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parties {
  pub originator: Party,
  pub beneficiary: Party,
}
