use rve_core::{
  application::{Decision, DecisionHit, DecisionOutcome, DecisionTrace},
  ports::rule_engine::{RuleEngineTrace, RuleEngineTraceStep},
};
use serde::Serialize;
use serde_json::Value;

/// Decision returned by the decision API.
#[derive(Debug, Clone, Serialize)]
pub struct DecisionResponse {
  pub score: f32,
  pub outcome: String,
  pub hits: Vec<DecisionHitResponse>,
  pub evaluated_rules: u32,
  pub executed_rules: u32,
  pub rollout_bucket: u8,
}

/// Rule hit returned by the decision API.
#[derive(Debug, Clone, Serialize)]
pub struct DecisionHitResponse {
  pub rule_id: String,
  pub action: String,
  pub severity: String,
  pub score_delta: f32,
  pub explanation: Option<String>,
  pub tags: Vec<String>,
}

/// Decision response enriched with runtime trace data.
#[derive(Debug, Clone, Serialize)]
pub struct DecisionTraceResponse {
  pub decision: DecisionResponse,
  pub trace: RuleEngineTraceResponse,
}

/// Runtime trace returned by the decision trace endpoint.
#[derive(Debug, Clone, Serialize)]
pub struct RuleEngineTraceResponse {
  pub channel: Option<String>,
  pub steps: Vec<RuleEngineTraceStepResponse>,
}

/// Runtime trace step returned by the decision trace endpoint.
#[derive(Debug, Clone, Serialize)]
pub struct RuleEngineTraceStepResponse {
  pub workflow_id: String,
  pub rule_id: Option<String>,
  pub runtime_channel: Option<String>,
  pub task_id: Option<String>,
  pub result: String,
}

impl From<&Decision> for DecisionResponse {
  fn from(decision: &Decision) -> Self {
    Self {
      score: decision.score,
      outcome: serialize_outcome(&decision.outcome),
      hits: decision.hits.iter().map(DecisionHitResponse::from).collect(),
      evaluated_rules: decision.evaluated_rules,
      executed_rules: decision.executed_rules,
      rollout_bucket: decision.rollout_bucket,
    }
  }
}

impl From<&DecisionHit> for DecisionHitResponse {
  fn from(hit: &DecisionHit) -> Self {
    Self {
      rule_id: hit.rule_id.to_string(),
      action: serialize_as_string(&hit.action),
      severity: serialize_as_string(&hit.severity),
      score_delta: hit.score_delta,
      explanation: hit.explanation.clone(),
      tags: hit.tags.clone(),
    }
  }
}

impl From<&DecisionTrace> for DecisionTraceResponse {
  fn from(trace: &DecisionTrace) -> Self {
    Self { decision: DecisionResponse::from(&trace.decision), trace: (&trace.trace).into() }
  }
}

impl From<&RuleEngineTrace> for RuleEngineTraceResponse {
  fn from(trace: &RuleEngineTrace) -> Self {
    Self {
      channel: trace.channel.clone(),
      steps: trace.steps.iter().map(RuleEngineTraceStepResponse::from).collect(),
    }
  }
}

impl From<&RuleEngineTraceStep> for RuleEngineTraceStepResponse {
  fn from(step: &RuleEngineTraceStep) -> Self {
    Self {
      workflow_id: step.workflow_id.clone(),
      rule_id: step.rule_id.clone(),
      runtime_channel: step.runtime_channel.clone(),
      task_id: step.task_id.clone(),
      result: step.result.clone(),
    }
  }
}

fn serialize_outcome(outcome: &DecisionOutcome) -> String {
  serialize_as_string(outcome)
}

fn serialize_as_string<T>(value: &T) -> String
where
  T: Serialize + std::fmt::Debug,
{
  match serde_json::to_value(value) {
    Ok(Value::String(value)) => value,
    Ok(value) => value.to_string(),
    Err(_) => format!("{value:?}"),
  }
}
