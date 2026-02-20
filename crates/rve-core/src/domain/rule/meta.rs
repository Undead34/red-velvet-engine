use serde::{Deserialize, Serialize};

/// Descriptive metadata owned by risk/fraud operators.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleMeta {
  pub name: String,
  pub description: Option<String>,
  pub version: semver::Version,
  pub autor: String,
  pub tags: Option<Vec<String>>,
}
