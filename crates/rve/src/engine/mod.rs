use std::{
  collections::HashMap,
  sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use async_trait::async_trait;
use dataflow_rs::{
  AsyncFunctionHandler, DataflowError, Engine, FunctionConfig, Workflow,
  engine::{message::Change, trace::ExecutionTrace},
};
use rve_core::{
  domain::{
    common::RuleId,
    event::Event,
    rule::{Rule, RuleAction, RuleMode},
  },
  ports::{
    RuleCompileStats, RulesetSnapshot, RuntimeEngineError, RuntimeEnginePort, RuntimeEvaluation,
    RuntimeHit,
  },
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleEngineStatus {
  pub mode: String,
  pub ready: bool,
  pub version: u64,
  pub loaded_rules: u32,
  pub compile_stats: RuleCompileStats,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleEngineTraceStep {
  pub workflow_id: String,
  pub task_id: Option<String>,
  pub result: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleEngineTrace {
  pub channel: Option<String>,
  pub steps: Vec<RuleEngineTraceStep>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleEngineExecution {
  pub evaluation: RuntimeEvaluation,
  pub trace: RuleEngineTrace,
}

#[async_trait]
pub trait RuleEnginePort: Send + Sync {
  async fn publish_rules(&self, rules: Vec<Rule>) -> Result<RulesetSnapshot, RuntimeEngineError>;
  async fn evaluate(&self, event: &Event) -> Result<RuntimeEvaluation, RuntimeEngineError>;
  async fn evaluate_with_trace(
    &self,
    event: &Event,
  ) -> Result<RuleEngineExecution, RuntimeEngineError>;
  async fn evaluate_in_channel(
    &self,
    channel: &str,
    event: &Event,
  ) -> Result<RuntimeEvaluation, RuntimeEngineError>;
  async fn reload(&self) -> Result<RulesetSnapshot, RuntimeEngineError>;
  fn status(&self) -> Result<RuleEngineStatus, RuntimeEngineError>;
}

pub struct RVEngine {
  runtime: Arc<dyn RuleEnginePort>,
}

impl Default for RVEngine {
  fn default() -> Self {
    Self::new()
  }
}

impl RVEngine {
  pub fn new() -> Self {
    Self { runtime: Arc::new(DataflowRuleEngine::new()) }
  }

  pub fn with_runtime(runtime: Arc<dyn RuleEnginePort>) -> Self {
    Self { runtime }
  }

  pub async fn reload(&self) -> Result<RulesetSnapshot, RuntimeEngineError> {
    self.runtime.reload().await
  }

  pub fn status(&self) -> Result<RuleEngineStatus, RuntimeEngineError> {
    self.runtime.status()
  }
}

#[async_trait]
impl RuntimeEnginePort for RVEngine {
  async fn publish_rules(&self, rules: Vec<Rule>) -> Result<RulesetSnapshot, RuntimeEngineError> {
    self.runtime.publish_rules(rules).await
  }

  async fn evaluate(&self, event: &Event) -> Result<RuntimeEvaluation, RuntimeEngineError> {
    self.runtime.evaluate(event).await
  }
}

#[derive(Default)]
pub struct DataflowRuleEngine {
  state: RwLock<DataflowState>,
}

struct DataflowState {
  version: u64,
  compiled_rules: u32,
  compile_stats: RuleCompileStats,
  published_rules: Vec<Rule>,
  rules_by_id: HashMap<String, Rule>,
  engine: Option<Arc<Engine>>,
}

impl Default for DataflowState {
  fn default() -> Self {
    Self {
      version: 0,
      compiled_rules: 0,
      compile_stats: RuleCompileStats { total_rules: 0, compiled_rules: 0, failed_rules: 0 },
      published_rules: Vec::new(),
      rules_by_id: HashMap::new(),
      engine: None,
    }
  }
}

impl DataflowRuleEngine {
  pub fn new() -> Self {
    Self::default()
  }

  fn read_state(&self) -> Result<RwLockReadGuard<'_, DataflowState>, RuntimeEngineError> {
    self
      .state
      .read()
      .map_err(|_| RuntimeEngineError::Internal { message: "rule engine lock poisoned".to_owned() })
  }

  fn write_state(&self) -> Result<RwLockWriteGuard<'_, DataflowState>, RuntimeEngineError> {
    self
      .state
      .write()
      .map_err(|_| RuntimeEngineError::Internal { message: "rule engine lock poisoned".to_owned() })
  }

  fn to_runtime_error(error: DataflowError) -> RuntimeEngineError {
    RuntimeEngineError::Evaluation { rule_id: None, message: error.to_string() }
  }

  fn compile_rules(rules: &[Rule]) -> Result<(Arc<Engine>, RuleCompileStats), RuntimeEngineError> {
    let mut workflows = Vec::with_capacity(rules.len());
    for rule in rules {
      workflows.push(rule_to_workflow(rule)?);
    }

    let engine = Engine::new(workflows, Some(custom_functions()));
    let compiled_rules = engine.workflows().len() as u32;
    let total_rules = rules.len() as u32;
    let compile_stats = RuleCompileStats {
      total_rules,
      compiled_rules,
      failed_rules: total_rules.saturating_sub(compiled_rules),
    };

    Ok((Arc::new(engine), compile_stats))
  }

  fn build_message(event: &Event) -> Result<dataflow_rs::Message, RuntimeEngineError> {
    let event_json =
      serde_json::to_value(event).map_err(|err| RuntimeEngineError::Internal { message: err.to_string() })?;
    let payload_json = event_json.get("payload").cloned().unwrap_or(Value::Null);
    let context_json = event_json.get("context").cloned().unwrap_or(Value::Null);
    let features_json = event_json.get("features").cloned().unwrap_or(Value::Null);
    let signals_json = event_json.get("signals").cloned().unwrap_or(Value::Null);
    let extensions_json = payload_json
      .get("extensions")
      .cloned()
      .unwrap_or_else(|| json!({}));
    let transaction_json = extensions_json
      .get("transaction")
      .cloned()
      .unwrap_or(Value::Null);
    let device_json = extensions_json.get("device").cloned().unwrap_or(Value::Null);

    let rollout_bucket = compute_rollout_bucket(event);
    let now_ms = event.header.timestamp.timestamp_millis().max(0) as u64;

    let mut message = dataflow_rs::Message::from_value(&payload_json);
    message.context = json!({
      "data": {
        "event": event_json,
        "payload": payload_json,
        "context": context_json,
        "features": features_json,
        "signals": signals_json,
        "extensions": extensions_json,
        "transaction": transaction_json,
        "device": device_json,
      },
      "metadata": {
        "now_ms": now_ms,
        "rollout_bucket": rollout_bucket,
      },
      "temp_data": {
        "rve_hits": []
      }
    });
    message.invalidate_context_cache();

    Ok(message)
  }

  fn runtime_evaluation_from_message(
    message: &dataflow_rs::Message,
    state: &DataflowState,
    event: &Event,
  ) -> Result<RuntimeEvaluation, RuntimeEngineError> {
    let now_ms = event.header.timestamp.timestamp_millis().max(0) as u64;
    let rollout_bucket = compute_rollout_bucket(event);

    let mut hits = extract_hits_from_message(message)?;
    hits.retain(|hit| {
      let key = hit.rule_id.to_string();
      state
        .rules_by_id
        .get(&key)
        .is_some_and(|rule| rule.is_executable(now_ms, rollout_bucket))
    });

    let score = hits.iter().map(|hit| hit.score_delta).sum::<f32>();
    let evaluated_rules = state
      .rules_by_id
      .values()
      .filter(|rule| rule.is_executable(now_ms, rollout_bucket))
      .count() as u32;

    Ok(RuntimeEvaluation { score, hits, evaluated_rules, rollout_bucket })
  }

  fn trace_from_dataflow(channel: Option<String>, trace: ExecutionTrace) -> RuleEngineTrace {
    let steps = trace
      .steps
      .into_iter()
      .map(|step| RuleEngineTraceStep {
        workflow_id: step.workflow_id,
        task_id: step.task_id,
        result: format!("{:?}", step.result).to_lowercase(),
      })
      .collect();

    RuleEngineTrace { channel, steps }
  }
}

#[async_trait]
impl RuleEnginePort for DataflowRuleEngine {
  async fn publish_rules(&self, rules: Vec<Rule>) -> Result<RulesetSnapshot, RuntimeEngineError> {
    let (engine, compile_stats) = Self::compile_rules(&rules)?;

    let mut rules_by_id = HashMap::with_capacity(rules.len());
    for rule in &rules {
      rules_by_id.insert(rule.id.to_string(), rule.clone());
    }

    let mut state = self.write_state()?;
    state.version = state.version.saturating_add(1);
    state.compiled_rules = compile_stats.compiled_rules;
    state.compile_stats = compile_stats.clone();
    state.published_rules = rules;
    state.rules_by_id = rules_by_id;
    state.engine = Some(engine);

    Ok(RulesetSnapshot {
      version: state.version,
      loaded_rules: state.compiled_rules,
      compile_stats,
    })
  }

  async fn evaluate(&self, event: &Event) -> Result<RuntimeEvaluation, RuntimeEngineError> {
    let (engine, state_rules) = {
      let state = self.read_state()?;
      let engine = state.engine.as_ref().cloned().ok_or_else(|| RuntimeEngineError::Configuration {
        message: "ruleset not loaded; call publish_rules first".to_owned(),
      })?;
      (engine, state.rules_by_id.clone())
    };

    let mut message = Self::build_message(event)?;
    engine
      .process_message(&mut message)
      .await
      .map_err(Self::to_runtime_error)?;

    let state = DataflowState {
      rules_by_id: state_rules,
      ..DataflowState::default()
    };
    Self::runtime_evaluation_from_message(&message, &state, event)
  }

  async fn evaluate_with_trace(
    &self,
    event: &Event,
  ) -> Result<RuleEngineExecution, RuntimeEngineError> {
    let (engine, state_rules) = {
      let state = self.read_state()?;
      let engine = state.engine.as_ref().cloned().ok_or_else(|| RuntimeEngineError::Configuration {
        message: "ruleset not loaded; call publish_rules first".to_owned(),
      })?;
      (engine, state.rules_by_id.clone())
    };

    let mut message = Self::build_message(event)?;
    let trace = engine
      .process_message_with_trace(&mut message)
      .await
      .map_err(Self::to_runtime_error)?;

    let state = DataflowState {
      rules_by_id: state_rules,
      ..DataflowState::default()
    };
    let evaluation = Self::runtime_evaluation_from_message(&message, &state, event)?;
    Ok(RuleEngineExecution {
      evaluation,
      trace: Self::trace_from_dataflow(None, trace),
    })
  }

  async fn evaluate_in_channel(
    &self,
    channel: &str,
    _event: &Event,
  ) -> Result<RuntimeEvaluation, RuntimeEngineError> {
    Err(RuntimeEngineError::Configuration {
      message: format!(
        "channel routing is unavailable with current dataflow-rs backend for channel '{channel}'"
      ),
    })
  }

  async fn reload(&self) -> Result<RulesetSnapshot, RuntimeEngineError> {
    let rules = {
      let state = self.read_state()?;
      state.published_rules.clone()
    };
    self.publish_rules(rules).await
  }

  fn status(&self) -> Result<RuleEngineStatus, RuntimeEngineError> {
    let state = self.read_state()?;
    Ok(RuleEngineStatus {
      mode: "dataflow-rs".to_owned(),
      ready: state.engine.is_some(),
      version: state.version,
      loaded_rules: state.compiled_rules,
      compile_stats: state.compile_stats.clone(),
    })
  }
}

fn custom_functions() -> HashMap<String, Box<dyn AsyncFunctionHandler + Send + Sync>> {
  let mut handlers: HashMap<String, Box<dyn AsyncFunctionHandler + Send + Sync>> = HashMap::new();
  handlers.insert("rve_emit_hit".to_owned(), Box::new(EmitHitFunction));
  handlers
}

fn rule_to_workflow(rule: &Rule) -> Result<Workflow, RuntimeEngineError> {
  let id = rule.id.to_string();
  let name = rule.identity().name.clone();
  let description = rule.identity().description.clone();

  let mut tasks = Vec::new();
  tasks.push(json!({
    "id": "emit_hit",
    "name": "Emit Runtime Hit",
    "function": {
      "name": "rve_emit_hit",
      "input": {
        "rule_id": id,
        "action": action_string(rule.enforcement().action),
        "severity": severity_string(rule.enforcement().severity),
        "score_delta": rule.enforcement().score_impact.as_f32(),
        "explanation": rule.identity().description,
        "tags": rule.enforcement().tags,
        "mode": mode_string(rule.state().mode),
      }
    }
  }));

  let condition = json!({
    "and": [
      rewrite_vars_for_dataflow(rule.evaluation().condition.as_value()),
      rewrite_vars_for_dataflow(rule.evaluation().logic.as_value())
    ]
  });

  let workflow_json = json!({
    "id": id,
    "name": name,
    "priority": 0,
    "description": description,
    "condition": condition,
    "continue_on_error": false,
    "tasks": tasks,
  });

  let workflow_str = serde_json::to_string(&workflow_json)
    .map_err(|err| RuntimeEngineError::Compilation { rule_id: Some(rule.id.clone()), message: err.to_string() })?;

  Workflow::from_json(&workflow_str)
    .map_err(|err| RuntimeEngineError::Compilation { rule_id: Some(rule.id.clone()), message: err.to_string() })
}

fn rewrite_vars_for_dataflow(value: &Value) -> Value {
  match value {
    Value::Object(map) => {
      let mut next = serde_json::Map::with_capacity(map.len());
      for (key, nested) in map {
        if key == "var" {
          let rewritten = match nested {
            Value::String(path) => Value::String(rewrite_var_path(path)),
            Value::Array(items) => {
              if let Some(Value::String(path)) = items.first() {
                let mut rewritten_items = items.clone();
                rewritten_items[0] = Value::String(rewrite_var_path(path));
                Value::Array(rewritten_items)
              } else {
                Value::Array(items.clone())
              }
            }
            _ => nested.clone(),
          };
          next.insert(key.clone(), rewritten);
          continue;
        }
        next.insert(key.clone(), rewrite_vars_for_dataflow(nested));
      }
      Value::Object(next)
    }
    Value::Array(items) => Value::Array(items.iter().map(rewrite_vars_for_dataflow).collect()),
    _ => value.clone(),
  }
}

fn rewrite_var_path(path: &str) -> String {
  let root = path.split('.').next().unwrap_or_default();
  match root {
    "event" | "payload" | "context" | "features" | "signals" | "extensions" | "transaction"
    | "device" => format!("data.{path}"),
    _ => path.to_owned(),
  }
}

fn extract_hits_from_message(message: &dataflow_rs::Message) -> Result<Vec<RuntimeHit>, RuntimeEngineError> {
  let Some(raw_hits) = message.context.get("temp_data").and_then(|v| v.get("rve_hits")) else {
    return Ok(Vec::new());
  };

  let Some(array) = raw_hits.as_array() else {
    return Ok(Vec::new());
  };

  let mut hits = Vec::with_capacity(array.len());
  for value in array {
    let emitted: EmittedHit = serde_json::from_value(value.clone()).map_err(|err| {
      RuntimeEngineError::Evaluation { rule_id: None, message: format!("invalid emitted hit: {err}") }
    })?;

    let rule_id = RuleId::try_from(emitted.rule_id.clone()).map_err(|err| RuntimeEngineError::Evaluation {
      rule_id: None,
      message: format!("invalid emitted rule_id '{}': {err}", emitted.rule_id),
    })?;

    hits.push(RuntimeHit {
      rule_id,
      action: emitted.action,
      severity: emitted.severity,
      score_delta: emitted.score_delta,
      explanation: emitted.explanation,
      tags: emitted.tags,
    });
  }

  Ok(hits)
}

fn compute_rollout_bucket(event: &Event) -> u8 {
  event
    .header
    .event_id
    .as_ref()
    .map(|id| (id.as_uuid().as_u128() % 100) as u8)
    .unwrap_or(0)
}

fn action_string(action: RuleAction) -> &'static str {
  match action {
    RuleAction::Allow => "allow",
    RuleAction::Review => "review",
    RuleAction::Block => "block",
    RuleAction::TagOnly => "tag_only",
  }
}

fn severity_string(severity: rve_core::domain::common::Severity) -> &'static str {
  match severity {
    rve_core::domain::common::Severity::Catastrophic => "catastrophic",
    rve_core::domain::common::Severity::VeryHigh => "very_high",
    rve_core::domain::common::Severity::High => "high",
    rve_core::domain::common::Severity::Moderate => "moderate",
    rve_core::domain::common::Severity::Low => "low",
    rve_core::domain::common::Severity::None => "none",
  }
}

fn mode_string(mode: RuleMode) -> &'static str {
  match mode {
    RuleMode::Active => "active",
    RuleMode::Suspended => "paused",
    RuleMode::Deactivated => "archived",
    RuleMode::Staged => "paused",
  }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct EmittedHit {
  rule_id: String,
  action: RuleAction,
  severity: rve_core::domain::common::Severity,
  score_delta: f32,
  explanation: Option<String>,
  tags: Vec<String>,
}

struct EmitHitFunction;

#[async_trait]
impl AsyncFunctionHandler for EmitHitFunction {
  async fn execute(
    &self,
    message: &mut dataflow_rs::Message,
    config: &FunctionConfig,
    _datalogic: Arc<datalogic_rs::DataLogic>,
  ) -> dataflow_rs::Result<(usize, Vec<Change>)> {
    let input = match config {
      FunctionConfig::Custom { input, .. } => input,
      _ => {
        return Err(DataflowError::Validation(
          "rve_emit_hit requires custom function config".to_owned(),
        ));
      }
    };

    let rule_id = input
      .get("rule_id")
      .and_then(Value::as_str)
      .ok_or_else(|| DataflowError::Validation("missing rule_id".to_owned()))?;
    let action = input
      .get("action")
      .and_then(Value::as_str)
      .ok_or_else(|| DataflowError::Validation("missing action".to_owned()))?;
    let severity = input
      .get("severity")
      .and_then(Value::as_str)
      .ok_or_else(|| DataflowError::Validation("missing severity".to_owned()))?;
    let score_delta = input
      .get("score_delta")
      .and_then(Value::as_f64)
      .ok_or_else(|| DataflowError::Validation("missing score_delta".to_owned()))?
      as f32;
    let mode = input
      .get("mode")
      .and_then(Value::as_str)
      .ok_or_else(|| DataflowError::Validation("missing mode".to_owned()))?;

    if mode != "active" {
      return Ok((200, Vec::new()));
    }

    let explanation = input.get("explanation").cloned().unwrap_or(Value::Null);
    let tags = input.get("tags").cloned().unwrap_or_else(|| json!([]));

    let old_value = message
      .context
      .get("temp_data")
      .and_then(|v| v.get("rve_hits"))
      .cloned()
      .unwrap_or_else(|| json!([]));

    let mut hits = old_value.as_array().cloned().unwrap_or_default();
    hits.push(json!({
      "rule_id": rule_id,
      "action": action,
      "severity": severity,
      "score_delta": score_delta,
      "explanation": explanation,
      "tags": tags,
    }));
    let new_value = Value::Array(hits);

    message.context["temp_data"]["rve_hits"] = new_value.clone();
    message.invalidate_context_cache();

    let changes = vec![Change {
      path: Arc::from("temp_data.rve_hits"),
      old_value: Arc::new(old_value),
      new_value: Arc::new(new_value),
    }];

    Ok((200, changes))
  }
}

#[cfg(test)]
mod tests {
  use std::collections::{BTreeMap, HashSet};

  use chrono::Utc;
  use rve_core::{
    domain::{
      common::{AccountId, Currency, EventSource, Flag, RuleId, Score, Severity},
      event::{
        Context, EnvironmentContext, Event, Features, FinancialFeatures, GeoContext, Header,
        NetworkContext, Parties, Party, Payload, Signals,
      },
      rule::{
        RolloutPolicy, Rule, RuleAction, RuleAudit, RuleDecision, RuleDefinition, RuleEnforcement,
        RuleEvaluation, RuleExpression, RuleIdentity, RuleMode, RulePolicy, RuleSchedule, RuleState,
      },
    },
    ports::RuntimeEnginePort,
  };
  use semver::Version;
  use serde_json::json;

  use crate::engine::RuleEnginePort;

  use super::DataflowRuleEngine;

  #[tokio::test]
  async fn publishes_rules_and_evaluates_one_hit() {
    let runtime = DataflowRuleEngine::new();
    let rule = sample_rule();

    let snapshot = runtime
      .publish_rules(vec![rule.clone()])
      .await
      .expect("publish should succeed");
    assert_eq!(snapshot.loaded_rules, 1);

    let evaluation = runtime.evaluate(&sample_event()).await.expect("evaluate should succeed");
    assert_eq!(evaluation.hits.len(), 1);
    assert_eq!(evaluation.evaluated_rules, 1);
    assert!(evaluation.score > 0.0);
  }

  #[tokio::test]
  async fn exposes_trace_steps() {
    let runtime = DataflowRuleEngine::new();
    runtime
      .publish_rules(vec![sample_rule()])
      .await
      .expect("publish should succeed");

    let execution = runtime
      .evaluate_with_trace(&sample_event())
      .await
      .expect("trace should succeed");

    assert!(!execution.trace.steps.is_empty());
    assert_eq!(execution.evaluation.hits.len(), 1);
  }

  #[tokio::test]
  async fn rv_engine_delegates_runtime_port() {
    let engine = super::RVEngine::new();
    let rule = sample_rule();
    let _ = RuntimeEnginePort::publish_rules(&engine, vec![rule]).await.expect("publish should work");
    let evaluation = RuntimeEnginePort::evaluate(&engine, &sample_event())
      .await
      .expect("evaluate should work");
    assert_eq!(evaluation.hits.len(), 1);
  }

  fn sample_rule() -> Rule {
    let rule_id = RuleId::new_v7();
    let identity = RuleIdentity {
      code: Some("FRAUD-HV-001".to_owned()),
      name: "High Value Transfer".to_owned(),
      description: Some("High value transfer detected".to_owned()),
      version: Version::new(1, 0, 0),
      author: "engine-test".to_owned(),
      tags: Some(vec!["fraud".to_owned(), "value".to_owned()]),
    };
    let state = RuleState::new(
      RuleMode::Active,
      RuleAudit {
        created_at_ms: 1_730_000_000_000u64.try_into().expect("valid ts"),
        updated_at_ms: 1_730_000_000_001u64.try_into().expect("valid ts"),
        created_by: Some("test".to_owned()),
        updated_by: Some("test".to_owned()),
      },
    )
    .expect("valid state");
    let policy = RulePolicy::new(
      state,
      RuleSchedule::new(None, None).expect("valid schedule"),
      RolloutPolicy::new(100).expect("valid rollout"),
    )
    .expect("valid policy");
    let evaluation = RuleEvaluation::new(
      RuleExpression::new(json!(true)).expect("valid expression"),
      RuleExpression::new(json!({">": [{"var": "payload.money.minor_units"}, 100000]}))
        .expect("valid expression"),
    )
    .expect("valid evaluation");
    let definition = RuleDefinition::new(evaluation).expect("valid definition");
    let outcome = RuleDecision::new(RuleEnforcement {
      score_impact: Score::new(6.5).expect("valid score"),
      action: RuleAction::Review,
      severity: Severity::High,
      tags: vec!["financial_fraud".to_owned()],
      cooldown_ms: None,
      functions: Vec::new(),
    });

    Rule::new(rule_id, identity, policy, definition, outcome).expect("valid rule")
  }

  fn sample_event() -> Event {
    Event::new(
      Header {
        timestamp: Utc::now(),
        source: EventSource::new("api_gateway").expect("valid source"),
        event_id: None,
        instrument: None,
        channel: None,
      },
      Context {
        geo: GeoContext {
          address: None,
          city: None,
          region: None,
          country: None,
          postal_code: None,
          lon: None,
          lat: None,
        },
        net: NetworkContext {
          source_ip: None,
          destination_ip: None,
          hop_count: None,
          asn: None,
          isp: None,
        },
        env: EnvironmentContext {
          user_agent: None,
          locale: None,
          timezone: None,
          device_id: None,
          session_id: None,
        },
      },
      Features {
        fin: FinancialFeatures {
          first_seen_at: 1_730_000_000_000,
          last_seen_at: 1_730_000_000_001,
          last_declined_at: None,
          total_successful_txns: 1,
          total_declined_txns: 0,
          total_amount_spent: 100,
          max_ticket_ever: 100,
          consecutive_failed_logins: 0,
          consecutive_declines: 0,
          current_hour_count: 1,
          current_hour_amount: 100,
          current_day_count: 1,
          current_day_amount: 100,
          known_ips: HashSet::from([String::from("1.1.1.1")]),
          known_devices: HashSet::from([String::from("dev_001")]),
        },
      },
      Signals { flags: BTreeMap::new() },
      Payload::value_transfer(
        rve_core::domain::common::Money::from_major_str("1500.0", Currency::new("USD").expect("currency"))
          .expect("money"),
        Parties {
          originator: Party::new(
            rve_core::domain::common::EntityType::Individual,
            AccountId::new("acct_001").expect("account"),
            None,
            None,
            None,
            Flag::Unknown,
            Some(0.1),
          )
          .expect("party"),
          beneficiary: Party::new(
            rve_core::domain::common::EntityType::Business,
            AccountId::new("acct_002").expect("account"),
            None,
            None,
            None,
            Flag::Unknown,
            Some(0.2),
          )
          .expect("party"),
        },
        BTreeMap::new(),
      ),
    )
    .expect("valid event")
  }
}
