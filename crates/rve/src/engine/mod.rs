use std::{
  collections::hash_map::DefaultHasher,
  hash::{Hash, Hasher},
  sync::Arc,
  time::{SystemTime, UNIX_EPOCH},
};

use arc_swap::ArcSwap;
use datalogic_rs::{CompiledLogic, DataLogic, Error as LogicError};
use serde_json::{Value, json};
use thiserror::Error;

use rve_core::{
  domain::{common::RuleId, event::Event, rule::Rule},
  services::engine::{Decision, DecisionHit, EngineResult},
};

#[derive(Debug, Error)]
pub enum EngineError {
  #[error("failed to compile rule {rule_id}: {source}")]
  Compilation {
    rule_id: RuleId,
    #[source]
    source: LogicError,
  },
  #[error("failed to evaluate rule {rule_id}: {source}")]
  Evaluation {
    rule_id: RuleId,
    #[source]
    source: LogicError,
  },
}

pub struct RVEngine {
  logic: Arc<DataLogic>,
  rules: ArcSwap<Vec<CompiledRule>>,
}

struct CompiledRule {
  rule: Rule,
  condition: Arc<CompiledLogic>,
  logic: Arc<CompiledLogic>,
}

impl RVEngine {
  pub fn new() -> Self {
    Self { logic: Arc::new(DataLogic::new()), rules: ArcSwap::from_pointee(Vec::new()) }
  }

  pub fn publish_rules(&self, rules: Vec<Rule>) -> Result<(), EngineError> {
    let compiled: Vec<CompiledRule> =
      rules.into_iter().map(|rule| self.compile_rule(rule)).collect::<Result<_, _>>()?;
    self.rules.store(Arc::new(compiled));
    Ok(())
  }

  fn compile_rule(&self, rule: Rule) -> Result<CompiledRule, EngineError> {
    let condition = self
      .logic
      .compile(rule.evaluation().condition.as_value())
      .map_err(|source| EngineError::Compilation { rule_id: rule.id.clone(), source })?;
    let logic = self
      .logic
      .compile(rule.evaluation().logic.as_value())
      .map_err(|source| EngineError::Compilation { rule_id: rule.id.clone(), source })?;

    Ok(CompiledRule { rule, condition, logic })
  }

  pub fn evaluate(&self, event: &Event) -> Result<EngineResult, EngineError> {
    let snapshot = self.rules.load();
    let bucket = bucket_for_event(event);
    let now_ms = current_time_ms();
    let context = Arc::new(build_context(event));

    let mut hits = Vec::new();
    let mut score = 0.0;
    let mut evaluated = 0u32;

    for compiled in snapshot.iter() {
      evaluated += 1;
      if !compiled.rule.is_executable(now_ms, bucket) {
        continue;
      }

      let condition_value = self
        .logic
        .evaluate(compiled.condition.as_ref(), Arc::clone(&context))
        .map_err(|source| EngineError::Evaluation { rule_id: compiled.rule.id.clone(), source })?;

      if !is_truthy(&condition_value) {
        continue;
      }

      let logic_value = self
        .logic
        .evaluate(compiled.logic.as_ref(), Arc::clone(&context))
        .map_err(|source| EngineError::Evaluation { rule_id: compiled.rule.id.clone(), source })?;

      if !is_truthy(&logic_value) {
        continue;
      }

      let delta = compiled.rule.enforcement().score_impact.as_f32();
      score += delta;

      hits.push(DecisionHit {
        rule_id: compiled.rule.id.clone(),
        action: compiled.rule.enforcement().action.clone(),
        severity: compiled.rule.enforcement().severity,
        score_delta: delta,
        explanation: compiled
          .rule
          .identity()
          .description
          .clone()
          .or_else(|| Some(compiled.rule.identity().name.clone())),
        tags: compiled.rule.enforcement().tags.clone(),
      });
    }

    Ok(Decision::with_scores(score, hits, evaluated, bucket))
  }
}

fn build_context(event: &Event) -> Value {
  let transaction = event.payload.extensions.get("transaction").cloned().unwrap_or(Value::Null);
  let device = event.payload.extensions.get("device").cloned().unwrap_or(Value::Null);

  json!({
    "event": event,
    "payload": event.payload,
    "context": event.context,
    "signals": event.signals,
    "extensions": event.payload.extensions,
    "transaction": transaction,
    "device": device,
  })
}

fn bucket_for_event(event: &Event) -> u8 {
  let mut hasher = DefaultHasher::new();
  if let Some(event_id) = &event.header.event_id {
    event_id.hash(&mut hasher);
  } else {
    event.header.source.hash(&mut hasher);
    if let Some(instrument) = &event.header.instrument {
      instrument.hash(&mut hasher);
    }
    if let Some(channel) = &event.header.channel {
      channel.hash(&mut hasher);
    }
  }
  (hasher.finish() % 100) as u8
}

fn current_time_ms() -> u64 {
  SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

fn is_truthy(value: &Value) -> bool {
  match value {
    Value::Null => false,
    Value::Bool(b) => *b,
    Value::Number(n) => n.as_f64().map(|v| v != 0.0).unwrap_or(false),
    Value::String(s) => !s.is_empty(),
    Value::Array(arr) => !arr.is_empty(),
    Value::Object(map) => !map.is_empty(),
  }
}
