use serde::{Deserialize, Serialize};

use crate::domain::common::Score;

#[derive(Debug, Serialize, Deserialize)]
pub struct EngineResult {
  pub score: Score,
  pub hits: Vec<String>,
}
