use serde::{Deserialize, Serialize};

/// Static metadata that identifies and describes a rule.
///
/// `RuleIdentity` provides the non-operational attributes of a fraud rule.
/// It defines what the rule is called, who owns it, and how it is tracked
/// across different systems (e.g., logs, dashboards, and ticketing)
/// independently of the engine's execution logic.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleIdentity {
  /// An environment-agnostic business identifier (e.g., `"FRAUD-HV-UNTRUSTED-01"`).
  pub code: Option<String>,

  /// The human-readable display name.
  pub name: String,

  /// A detailed explanation of the rule's criteria and intent.
  pub description: Option<String>,

  /// The semantic version tracking the rule's iterations.
  pub version: semver::Version,

  /// The entity or team responsible for maintaining the rule.
  pub author: String,

  /// Categorical labels used for filtering and aggregation.
  pub tags: Option<Vec<String>>,
}

/// Type alias for [`RuleIdentity`].
pub type RuleMeta = RuleIdentity;
