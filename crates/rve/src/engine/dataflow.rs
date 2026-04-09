use std::{
  collections::HashMap,
  sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use async_trait::async_trait;
use dataflow_rs::{
  AsyncFunctionHandler, DataflowError, Engine, FunctionConfig, engine::message::Change,
};
use serde_json::Value;

use rve_core::domain::{event::Event, rule::Rule};
use rve_core::ports::rule_engine::*;

const GLOBAL_WORKFLOW_CHANNEL: &str = "__rve_all__";
const SCOPED_WORKFLOW_CHANNEL_PREFIX: &str = "__rve_channel__:";

// ==========================================
// ESTADO Y ESTRUCTURA PRINCIPAL
// ==========================================

#[derive(Default)]
struct DataflowState {
  version: u64,
  compiled_rules: u32,
  compile_stats: RuleCompileStats,
  published_rules: Vec<Rule>,
  rules_by_id: HashMap<String, Rule>,
  rules_by_channel: HashMap<String, HashMap<String, Rule>>,
  engine: Option<Arc<Engine>>,
}

#[derive(Default)]
pub struct DataflowRuleEngine {
  state: RwLock<DataflowState>,
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

  fn compile_rules(
    rules: &[Rule],
    existing_engine: Option<&Arc<Engine>>,
  ) -> Result<
    (Arc<Engine>, RuleCompileStats, HashMap<String, HashMap<String, Rule>>),
    RuntimeEngineError,
  > {
    let workflows = rules
      .iter()
      .map(mapper::rule_to_workflows)
      .collect::<Result<Vec<_>, _>>()?
      .into_iter()
      .flatten()
      .collect::<Vec<_>>();

    let engine = if let Some(engine) = existing_engine {
      engine.with_new_workflows(workflows)
    } else {
      Engine::new(workflows, Some(functions::custom_functions()))
    };
    let total_rules = rules.len() as u32;
    let compiled_rules = total_rules;
    let rules_by_channel = mapper::index_rules_by_channel(rules);

    let compile_stats = RuleCompileStats {
      total_rules,
      compiled_rules,
      failed_rules: total_rules.saturating_sub(compiled_rules),
    };

    Ok((Arc::new(engine), compile_stats, rules_by_channel))
  }
}

// ==========================================
// IMPLEMENTACIÓN DEL PUERTO (TRAIT)
// ==========================================

#[async_trait]
impl RuleEnginePort for DataflowRuleEngine {
  async fn publish_rules(&self, rules: Vec<Rule>) -> Result<RulesetSnapshot, RuntimeEngineError> {
    let existing_engine = self.read_state()?.engine.clone();
    let (engine, compile_stats, rules_by_channel) =
      Self::compile_rules(&rules, existing_engine.as_ref())?;
    let rules_by_id: HashMap<String, Rule> =
      rules.iter().map(|r| (r.id.to_string(), r.clone())).collect();

    let mut state = self.write_state()?;
    state.version = state.version.saturating_add(1);
    state.compiled_rules = compile_stats.compiled_rules;
    state.compile_stats = compile_stats.clone();
    state.published_rules = rules;
    state.rules_by_id = rules_by_id;
    state.rules_by_channel = rules_by_channel;
    state.engine = Some(engine);

    Ok(RulesetSnapshot {
      version: state.version,
      loaded_rules: state.compiled_rules,
      compile_stats,
    })
  }

  async fn evaluate(&self, event: &Event) -> Result<RuntimeEvaluation, RuntimeEngineError> {
    let (engine, rules_by_channel) = {
      let state = self.read_state()?;
      let engine = state.engine.clone().ok_or_else(|| RuntimeEngineError::Configuration {
        message: "ruleset not loaded; call publish_rules first".to_owned(),
      })?;
      (engine, state.rules_by_channel.clone())
    };

    let mut message = mapper::build_message(event)?;
    let (rules_by_id, evaluation_channel) =
      mapper::process_event_channels(&engine, &mut message, event, &rules_by_channel).await?;

    mapper::runtime_evaluation_from_message(&message, &rules_by_id, event, evaluation_channel)
  }

  async fn evaluate_with_trace(
    &self,
    event: &Event,
  ) -> Result<RuleEngineExecution, RuntimeEngineError> {
    let (engine, rules_by_channel) = {
      let state = self.read_state()?;
      let engine = state.engine.clone().ok_or_else(|| RuntimeEngineError::Configuration {
        message: "ruleset not loaded; call publish_rules first".to_owned(),
      })?;
      (engine, state.rules_by_channel.clone())
    };

    let mut message = mapper::build_message(event)?;
    let (rules_by_id, evaluation_channel, trace) =
      mapper::process_event_channels_with_trace(&engine, &mut message, event, &rules_by_channel)
        .await?;

    let evaluation = mapper::runtime_evaluation_from_message(
      &message,
      &rules_by_id,
      event,
      evaluation_channel.clone(),
    )?;

    Ok(RuleEngineExecution {
      evaluation,
      trace: mapper::trace_from_dataflow(evaluation_channel, trace),
    })
  }

  async fn evaluate_in_channel(
    &self,
    channel: &str,
    event: &Event,
  ) -> Result<RuntimeEvaluation, RuntimeEngineError> {
    let (engine, rules_by_channel) = {
      let state = self.read_state()?;
      let engine = state.engine.clone().ok_or_else(|| RuntimeEngineError::Configuration {
        message: "ruleset not loaded; call publish_rules first".to_owned(),
      })?;
      (engine, state.rules_by_channel.clone())
    };

    let mut message = mapper::build_message(event)?;
    let runtime_channels = mapper::runtime_channels_for_override(channel);
    let channel_rules =
      mapper::collect_rules_for_runtime_channels(&rules_by_channel, &runtime_channels);

    mapper::process_runtime_channels(&engine, &mut message, &runtime_channels).await?;

    mapper::runtime_evaluation_from_message(
      &message,
      &channel_rules,
      event,
      Some(channel.to_owned()),
    )
  }

  async fn reload(&self) -> Result<RulesetSnapshot, RuntimeEngineError> {
    let rules = self.read_state()?.published_rules.clone();
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

// ==========================================
// MÓDULO DE FUNCIONES CUSTOM (DATAFLOW)
// ==========================================

mod functions {
  use super::*;
  use serde::Deserialize;
  use serde_json::json;

  pub fn custom_functions() -> HashMap<String, Box<dyn AsyncFunctionHandler + Send + Sync>> {
    let mut handlers: HashMap<String, Box<dyn AsyncFunctionHandler + Send + Sync>> = HashMap::new();
    handlers.insert("rve_emit_hit".to_owned(), Box::new(EmitHitFunction));
    handlers
  }

  #[derive(Deserialize)]
  struct EmitHitInput {
    rule_id: String,
    action: String,
    severity: String,
    score_delta: f32,
    mode: String,
    explanation: Option<String>,
    #[serde(default)]
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
      let FunctionConfig::Custom { input: raw_input, .. } = config else {
        return Err(DataflowError::Validation("Requires custom function config".to_owned()));
      };

      // Deserializamos el input tipado automáticamente (código mucho más limpio)
      let input_val = serde_json::to_value(raw_input).unwrap_or_default();
      let input: EmitHitInput = serde_json::from_value(input_val)
        .map_err(|e| DataflowError::Validation(format!("Invalid inputs: {}", e)))?;

      if input.mode != "active" {
        return Ok((200, Vec::new()));
      }

      let old_value = message
        .context
        .get("temp_data")
        .and_then(|v| v.get("rve_hits"))
        .cloned()
        .unwrap_or_else(|| json!([]));

      let mut hits = old_value.as_array().cloned().unwrap_or_default();
      hits.push(json!({
          "rule_id": input.rule_id,
          "action": input.action,
          "severity": input.severity,
          "score_delta": input.score_delta,
          "explanation": input.explanation,
          "tags": input.tags,
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
}

// ==========================================
// MÓDULO DE MAPEOS (DOMAIN <-> DATAFLOW)
// ==========================================

mod mapper {
  use super::*;
  use dataflow_rs::{Workflow, engine::trace::ExecutionTrace};
  use rve_core::domain::{
    common::{Channel, RuleId},
    rule::RuleMode,
  };
  use serde::Deserialize;
  use serde_json::json;

  pub fn build_message(event: &Event) -> Result<dataflow_rs::Message, RuntimeEngineError> {
    let event_json = serde_json::to_value(event)
      .map_err(|err| RuntimeEngineError::Internal { message: err.to_string() })?;

    let get_json = |key: &str| event_json.get(key).cloned().unwrap_or(Value::Null);
    let get_ext = |key: &str| {
      get_json("payload").get("extensions").and_then(|e| e.get(key)).cloned().unwrap_or(Value::Null)
    };

    let now_ms = event.header.timestamp.timestamp_millis().max(0) as u64;
    let rollout_bucket = compute_rollout_bucket(event);

    let mut message = dataflow_rs::Message::from_value(&get_json("payload"));
    message.context = json!({
        "data": {
            "event": event_json,
            "payload": get_json("payload"),
            "context": get_json("context"),
            "features": get_json("features"),
            "signals": get_json("signals"),
            "extensions": get_json("payload").get("extensions").cloned().unwrap_or_else(|| json!({})),
            "transaction": get_ext("transaction"),
            "device": get_ext("device"),
        },
        "metadata": { "now_ms": now_ms, "rollout_bucket": rollout_bucket },
        "temp_data": { "rve_hits": [] }
    });
    message.invalidate_context_cache();

    Ok(message)
  }

  pub fn runtime_evaluation_from_message(
    message: &dataflow_rs::Message,
    rules_by_id: &HashMap<String, Rule>,
    event: &Event,
    evaluation_channel: Option<String>,
  ) -> Result<RuntimeEvaluation, RuntimeEngineError> {
    let now_ms = event.header.timestamp.timestamp_millis().max(0) as u64;
    let rollout_bucket = compute_rollout_bucket(event);
    let event_channel = event.header.channel.as_ref();

    let mut hits = extract_hits(message)?;
    hits.retain(|hit| {
      rules_by_id
        .get(&hit.rule_id.to_string())
        .is_some_and(|r| r.is_executable_for_channel(now_ms, rollout_bucket, event_channel))
    });

    let score = hits.iter().map(|h| h.score_delta).sum::<f32>();
    let evaluated_rules = rules_by_id
      .values()
      .filter(|r| r.is_executable_for_channel(now_ms, rollout_bucket, event_channel))
      .count() as u32;

    let _ = evaluation_channel;

    Ok(RuntimeEvaluation { score, hits, evaluated_rules, rollout_bucket })
  }

  pub fn rule_to_workflows(rule: &Rule) -> Result<Vec<Workflow>, RuntimeEngineError> {
    runtime_channels_for_rule(rule)
      .into_iter()
      .map(|runtime_channel| rule_to_workflow(rule, &runtime_channel))
      .collect()
  }

  pub fn index_rules_by_channel(rules: &[Rule]) -> HashMap<String, HashMap<String, Rule>> {
    let mut index = HashMap::<String, HashMap<String, Rule>>::new();

    for rule in rules.iter().filter(|rule| rule.is_active_mode()) {
      for runtime_channel in runtime_channels_for_rule(rule) {
        index.entry(runtime_channel).or_default().insert(rule.id.to_string(), rule.clone());
      }
    }

    index
  }

  pub async fn process_event_channels(
    engine: &Arc<Engine>,
    message: &mut dataflow_rs::Message,
    event: &Event,
    rules_by_channel: &HashMap<String, HashMap<String, Rule>>,
  ) -> Result<(HashMap<String, Rule>, Option<String>), RuntimeEngineError> {
    let runtime_channels = runtime_channels_for_event(event.header.channel.as_ref());
    let rules = collect_rules_for_runtime_channels(rules_by_channel, &runtime_channels);

    process_runtime_channels(engine, message, &runtime_channels).await?;

    Ok((rules, event.header.channel.as_ref().map(ToString::to_string)))
  }

  pub async fn process_event_channels_with_trace(
    engine: &Arc<Engine>,
    message: &mut dataflow_rs::Message,
    event: &Event,
    rules_by_channel: &HashMap<String, HashMap<String, Rule>>,
  ) -> Result<(HashMap<String, Rule>, Option<String>, ExecutionTrace), RuntimeEngineError> {
    let runtime_channels = runtime_channels_for_event(event.header.channel.as_ref());
    let rules = collect_rules_for_runtime_channels(rules_by_channel, &runtime_channels);
    let trace = process_runtime_channels_with_trace(engine, message, &runtime_channels).await?;

    Ok((rules, event.header.channel.as_ref().map(ToString::to_string), trace))
  }

  pub async fn process_runtime_channels(
    engine: &Arc<Engine>,
    message: &mut dataflow_rs::Message,
    runtime_channels: &[String],
  ) -> Result<(), RuntimeEngineError> {
    for runtime_channel in runtime_channels {
      engine
        .process_message_for_channel(runtime_channel, message)
        .await
        .map_err(|e| RuntimeEngineError::Evaluation { rule_id: None, message: e.to_string() })?;
    }

    Ok(())
  }

  pub async fn process_runtime_channels_with_trace(
    engine: &Arc<Engine>,
    message: &mut dataflow_rs::Message,
    runtime_channels: &[String],
  ) -> Result<ExecutionTrace, RuntimeEngineError> {
    let mut combined = ExecutionTrace::new();

    for runtime_channel in runtime_channels {
      let trace = engine
        .process_message_for_channel_with_trace(runtime_channel, message)
        .await
        .map_err(|e| RuntimeEngineError::Evaluation { rule_id: None, message: e.to_string() })?;
      combined.steps.extend(trace.steps);
    }

    Ok(combined)
  }

  pub fn collect_rules_for_runtime_channels(
    rules_by_channel: &HashMap<String, HashMap<String, Rule>>,
    runtime_channels: &[String],
  ) -> HashMap<String, Rule> {
    let mut merged = HashMap::new();

    for runtime_channel in runtime_channels {
      if let Some(channel_rules) = rules_by_channel.get(runtime_channel) {
        for (rule_id, rule) in channel_rules {
          merged.insert(rule_id.clone(), rule.clone());
        }
      }
    }

    merged
  }

  pub fn runtime_channels_for_override(channel: &str) -> Vec<String> {
    let mut channels = vec![GLOBAL_WORKFLOW_CHANNEL.to_owned()];
    channels.push(scoped_runtime_channel(channel));
    channels
  }

  fn runtime_channels_for_event(channel: Option<&Channel>) -> Vec<String> {
    let mut channels = vec![GLOBAL_WORKFLOW_CHANNEL.to_owned()];
    if let Some(channel) = channel {
      channels.push(scoped_runtime_channel(channel.as_str()));
    }
    channels
  }

  fn runtime_channels_for_rule(rule: &Rule) -> Vec<String> {
    match rule.scope().channels() {
      None => vec![GLOBAL_WORKFLOW_CHANNEL.to_owned()],
      Some(channels) => {
        channels.iter().map(|channel| scoped_runtime_channel(channel.as_str())).collect()
      }
    }
  }

  fn rule_to_workflow(rule: &Rule, runtime_channel: &str) -> Result<Workflow, RuntimeEngineError> {
    let rule_id = rule.id.to_string();
    let workflow_json = json!({
        "id": workflow_id(&rule_id, runtime_channel),
        "name": rule.identity().name,
        "priority": 0,
        "channel": runtime_channel,
        "description": rule.identity().description,
        "status": workflow_status(rule.state().mode),
        "condition": {
            "and": [
                rewrite_vars(rule.evaluation().condition.as_value()),
                rewrite_vars(rule.evaluation().logic.as_value())
            ]
        },
        "continue_on_error": false,
        "tasks": [{
            "id": "emit_hit",
            "name": "Emit Runtime Hit",
            "function": {
                "name": "rve_emit_hit",
                "input": {
                    "rule_id": rule_id,
                    "action": serde_json::to_string(&rule.enforcement().action).unwrap_or_default().trim_matches('"'),
                    "severity": serde_json::to_string(&rule.enforcement().severity).unwrap_or_default().trim_matches('"'),
                    "score_delta": rule.enforcement().score_impact.as_f32(),
                    "explanation": rule.identity().description,
                    "tags": rule.enforcement().tags,
                    "mode": match rule.state().mode {
                        RuleMode::Active => "active",
                        RuleMode::Suspended | RuleMode::Staged => "paused",
                        RuleMode::Deactivated => "archived",
                    },
                }
            }
        }],
    });

    let workflow_str = serde_json::to_string(&workflow_json).map_err(|e| {
      RuntimeEngineError::Compilation { rule_id: Some(rule.id.clone()), message: e.to_string() }
    })?;

    Workflow::from_json(&workflow_str).map_err(|e| RuntimeEngineError::Compilation {
      rule_id: Some(rule.id.clone()),
      message: e.to_string(),
    })
  }

  pub fn trace_from_dataflow(channel: Option<String>, trace: ExecutionTrace) -> RuleEngineTrace {
    let steps = trace
      .steps
      .into_iter()
      .map(|step| {
        let (rule_id, runtime_channel) = split_workflow_identity(&step.workflow_id);

        RuleEngineTraceStep {
          workflow_id: step.workflow_id,
          rule_id,
          runtime_channel,
          task_id: step.task_id,
          result: format!("{:?}", step.result).to_lowercase(),
        }
      })
      .collect();

    RuleEngineTrace { channel, steps }
  }

  fn extract_hits(message: &dataflow_rs::Message) -> Result<Vec<RuntimeHit>, RuntimeEngineError> {
    let raw_hits = message.context.get("temp_data").and_then(|v| v.get("rve_hits"));
    let Some(array) = raw_hits.and_then(Value::as_array) else { return Ok(Vec::new()) };

    #[derive(Deserialize)]
    struct EmittedHit {
      rule_id: String,
      action: rve_core::domain::rule::RuleAction,
      severity: rve_core::domain::common::Severity,
      score_delta: f32,
      explanation: Option<String>,
      tags: Vec<String>,
    }

    array
      .iter()
      .map(|value| {
        let emitted: EmittedHit =
          serde_json::from_value(value.clone()).map_err(|e| RuntimeEngineError::Evaluation {
            rule_id: None,
            message: format!("invalid emitted hit: {e}"),
          })?;
        let rule_id = RuleId::try_from(emitted.rule_id.clone()).map_err(|e| {
          RuntimeEngineError::Evaluation { rule_id: None, message: format!("invalid rule_id: {e}") }
        })?;

        Ok(RuntimeHit {
          rule_id,
          action: emitted.action,
          severity: emitted.severity,
          score_delta: emitted.score_delta,
          explanation: emitted.explanation,
          tags: emitted.tags,
        })
      })
      .collect()
  }

  fn compute_rollout_bucket(event: &Event) -> u8 {
    event.header.event_id.as_ref().map(|id| (id.as_uuid().as_u128() % 100) as u8).unwrap_or(0)
  }

  fn rewrite_vars(value: &Value) -> Value {
    match value {
      Value::Object(map) => {
        let mut next = serde_json::Map::with_capacity(map.len());
        for (key, nested) in map {
          if key == "var" {
            let rewritten = match nested {
              Value::String(p) => Value::String(rewrite_path(p)),
              Value::Array(items) if !items.is_empty() => {
                let mut new_items = items.clone();
                if let Value::String(p) = &items[0] {
                  new_items[0] = Value::String(rewrite_path(p));
                }
                Value::Array(new_items)
              }
              _ => nested.clone(),
            };
            next.insert(key.clone(), rewritten);
          } else {
            next.insert(key.clone(), rewrite_vars(nested));
          }
        }
        Value::Object(next)
      }
      Value::Array(items) => Value::Array(items.iter().map(rewrite_vars).collect()),
      _ => value.clone(),
    }
  }

  fn rewrite_path(path: &str) -> String {
    match path.split('.').next().unwrap_or_default() {
      "event" | "payload" | "context" | "features" | "signals" | "extensions" | "transaction"
      | "device" => format!("data.{path}"),
      _ => path.to_owned(),
    }
  }

  fn workflow_status(mode: RuleMode) -> &'static str {
    match mode {
      RuleMode::Active => "active",
      RuleMode::Suspended | RuleMode::Staged => "paused",
      RuleMode::Deactivated => "archived",
    }
  }

  fn scoped_runtime_channel(channel: &str) -> String {
    format!("{SCOPED_WORKFLOW_CHANNEL_PREFIX}{channel}")
  }

  fn workflow_id(rule_id: &str, runtime_channel: &str) -> String {
    format!("{rule_id}::{runtime_channel}")
  }

  fn split_workflow_identity(workflow_id: &str) -> (Option<String>, Option<String>) {
    let Some((rule_id, runtime_channel)) = workflow_id.split_once("::") else {
      return (None, None);
    };

    let runtime_channel = if runtime_channel == GLOBAL_WORKFLOW_CHANNEL {
      Some("all".to_owned())
    } else {
      runtime_channel
        .strip_prefix(SCOPED_WORKFLOW_CHANNEL_PREFIX)
        .map(std::borrow::ToOwned::to_owned)
    };

    (Some(rule_id.to_owned()), runtime_channel)
  }
}
