use serde::{Deserialize, Serialize};

/// The operational directive prescribed by a triggered rule.
///
/// `RuleAction` defines the high-level enforcement strategy that the consuming
/// system must apply to an event. It categorizes the risk response into
/// discrete execution paths.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleAction {
  /// Permits the event to proceed through the pipeline without interference.
  Allow,

  /// Routes the event to a manual queue for human operator inspection.
  ///
  /// The transaction is typically held in a pending state until a
  /// decision is reached.
  Review,

  /// Terminates the event flow immediately.
  ///
  /// This is a final, restrictive action used to prevent suspected fraud
  /// in real-time.
  Block,

  /// Annotates the event with metadata without altering its execution flow.
  ///
  /// Used primarily for "shadow" testing, data enrichment, or downstream
  /// asynchronous analysis.
  TagOnly,
}
