use serde::{Deserialize, Serialize};

use crate::domain::{
  common::Severity,
  rule::{RuleAction, RuleId},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EngineResult {
  pub score: f32,
  pub hits: Vec<RuleHit>,
  pub evaluated_rules: u32,
  pub rollout_bucket: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RuleHit {
  pub rule_id: RuleId,
  pub action: RuleAction,
  pub severity: Severity,
  pub score_delta: f32,
  pub explanation: Option<String>,
  pub tags: Vec<String>,
}
