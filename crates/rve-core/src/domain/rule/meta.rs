use serde::{Deserialize, Serialize};

/// Descriptive metadata owned by risk/fraud operators.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleMeta {
  /// Human-defined business code (for example: FRAUD-HV-UNTRUSTED-01).
  ///
  /// Practical uses:
  /// - operator search/filter in consoles,
  /// - references in tickets/runbooks/Slack,
  /// - continuity across environments even when technical IDs differ.
  ///
  /// This field is intended for user-facing templates in the frontend.
  pub code: Option<String>,
  pub name: String,
  pub description: Option<String>,
  pub version: semver::Version,
  pub autor: String,
  pub tags: Option<Vec<String>>,
}
