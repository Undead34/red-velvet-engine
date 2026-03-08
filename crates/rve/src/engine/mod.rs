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
    "features": event.features,
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

#[cfg(test)]
mod tests {
  use std::collections::{BTreeMap, HashSet};

  use chrono::Utc;
  use rve_core::domain::{
    common::{AccountId, Currency, EventSource, Flag, RuleId, Score, Severity, TimestampMs},
    event::{
      Context, EnvironmentContext, Event, Features, FinancialFeatures, GeoContext, Header,
      NetworkContext, Parties, Party, Payload, Signals,
    },
    rule::{
      RolloutPolicy, Rule, RuleAction, RuleAudit, RuleDecision, RuleDefinition, RuleEnforcement,
      RuleEvaluation, RuleExpression, RuleIdentity, RuleMode, RulePolicy, RuleSchedule, RuleState,
    },
  };
  use serde_json::json;

  use super::RVEngine;

  #[test]
  fn evaluates_rule_using_features_context() {
    let engine = RVEngine::new();
    engine.publish_rules(vec![valid_rule()]).expect("rules compile");

    let event = valid_event();
    let decision = engine.evaluate(&event).expect("evaluation succeeds");

    assert_eq!(decision.hits.len(), 1);
    assert!(decision.score > 0.0);
    assert!(matches!(decision.outcome, rve_core::services::engine::DecisionOutcome::Review));
  }

  fn valid_rule() -> Rule {
    Rule::new(
      RuleId::new_v7(),
      RuleIdentity {
        code: Some("FRAUD-FEATURES-001".into()),
        name: "Features Velocity".into(),
        description: Some("match when current_hour_count > 0".into()),
        version: semver::Version::new(1, 0, 0),
        author: "risk".into(),
        tags: Some(vec!["velocity".into()]),
      },
      RulePolicy::new(
        RuleState::new(
          RuleMode::Active,
          RuleAudit {
            created_at_ms: TimestampMs::new(1_730_000_000_000).unwrap(),
            updated_at_ms: TimestampMs::new(1_730_000_000_001).unwrap(),
            created_by: Some("test".into()),
            updated_by: Some("test".into()),
          },
        )
        .unwrap(),
        RuleSchedule::new(None, None).unwrap(),
        RolloutPolicy::new(100).unwrap(),
      )
      .unwrap(),
      RuleDefinition::new(
        RuleEvaluation::new(
          RuleExpression::new(json!(true)).unwrap(),
          RuleExpression::new(json!({ ">": [{ "var": "features.fin.current_hour_count" }, 0] }))
            .unwrap(),
        )
        .unwrap(),
      )
      .unwrap(),
      RuleDecision::new(RuleEnforcement {
        score_impact: Score::new(7.0).unwrap(),
        action: RuleAction::Review,
        severity: Severity::High,
        tags: vec!["velocity".into()],
        cooldown_ms: None,
      }),
    )
    .unwrap()
  }

  fn valid_event() -> Event {
    Event::new(
      Header {
        timestamp: Utc::now(),
        source: EventSource::new("api_gateway").unwrap(),
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
          current_hour_count: 3,
          current_hour_amount: 100,
          current_day_count: 3,
          current_day_amount: 100,
          known_ips: HashSet::from([String::from("1.1.1.1")]),
          known_devices: HashSet::from([String::from("dev_001")]),
        },
      },
      Signals { flags: BTreeMap::from([]) },
      Payload {
        money: rve_core::domain::common::Money::new(100.0, Currency::new("USD").unwrap()).unwrap(),
        parties: Parties {
          originator: Party::new(
            rve_core::domain::common::EntityType::Individual,
            AccountId::new("acct_001").unwrap(),
            None,
            None,
            None,
            Flag::Unknown,
            Some(0.1),
          )
          .unwrap(),
          beneficiary: Party::new(
            rve_core::domain::common::EntityType::Business,
            AccountId::new("acct_002").unwrap(),
            None,
            None,
            None,
            Flag::Unknown,
            Some(0.2),
          )
          .unwrap(),
        },
        extensions: BTreeMap::new(),
      },
    )
    .unwrap()
  }
}
