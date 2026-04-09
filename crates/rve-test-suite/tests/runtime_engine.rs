use std::collections::{BTreeMap, HashSet};

use chrono::Utc;
use rve::engine::DataflowRuleEngine;
use rve_core::{
  domain::{
    common::{
      AccountId, Channel, Currency, EventSource, Flag, RuleId, Score, Severity, TimestampMs,
    },
    event::{
      Context, EnvironmentContext, Event, Features, FinancialFeatures, GeoContext, Header,
      NetworkContext, Parties, Party, Payload, Signals,
    },
    rule::{
      RolloutPolicy, Rule, RuleAction, RuleAudit, RuleDecision, RuleDefinition, RuleEnforcement,
      RuleEvaluation, RuleExpression, RuleIdentity, RuleMode, RulePolicy, RuleSchedule, RuleScope,
      RuleState,
    },
  },
  ports::RuleEnginePort,
};

#[tokio::test]
async fn publish_rules_returns_snapshot() {
  let engine = DataflowRuleEngine::new();
  let snapshot =
    RuleEnginePort::publish_rules(&engine, vec![]).await.expect("runtime publish should succeed");

  assert_eq!(snapshot.loaded_rules, 0);
  assert_eq!(snapshot.compile_stats.failed_rules, 0);
}

#[tokio::test]
async fn evaluate_returns_runtime_evaluation() {
  let engine = DataflowRuleEngine::new();
  RuleEnginePort::publish_rules(&engine, vec![]).await.expect("runtime publish should succeed");
  let evaluation = RuleEnginePort::evaluate(&engine, &valid_event())
    .await
    .expect("runtime evaluate should succeed");

  assert_eq!(evaluation.hits.len(), 0);
  assert_eq!(evaluation.score, 0.0);
  assert_eq!(evaluation.evaluated_rules, 0);
}

#[tokio::test]
async fn publish_rules_hot_swaps_workflows_and_keeps_emitting_hits() {
  let engine = DataflowRuleEngine::new();
  RuleEnginePort::publish_rules(
    &engine,
    vec![active_rule("rule-a", 5_000, 6.5, RuleAction::Review, None)],
  )
  .await
  .expect("first publish should succeed");

  let first = RuleEnginePort::evaluate(&engine, &valid_event())
    .await
    .expect("first evaluation should succeed");
  assert_eq!(first.hits.len(), 1);
  assert!(matches!(first.hits[0].action, RuleAction::Review));
  assert_eq!(first.score, 6.5);

  let second_snapshot = RuleEnginePort::publish_rules(
    &engine,
    vec![active_rule("rule-b", 3_000, 9.0, RuleAction::Block, None)],
  )
  .await
  .expect("second publish should succeed");

  assert_eq!(second_snapshot.version, 2);
  assert_eq!(second_snapshot.loaded_rules, 1);

  let second = RuleEnginePort::evaluate(&engine, &valid_event())
    .await
    .expect("second evaluation should succeed");
  assert_eq!(second.hits.len(), 1);
  assert!(matches!(second.hits[0].action, RuleAction::Block));
  assert_eq!(second.score, 9.0);
}

#[tokio::test]
async fn evaluate_in_channel_uses_rule_scope_and_native_channel_routing() {
  let engine = DataflowRuleEngine::new();
  RuleEnginePort::publish_rules(
    &engine,
    vec![active_rule("rule-web", 5_000, 6.5, RuleAction::Review, Some(&["web"]))],
  )
  .await
  .expect("runtime publish should succeed");

  let web_channel =
    RuleEnginePort::evaluate_in_channel(&engine, "web", &valid_event_in_channel("web"))
      .await
      .expect("web channel evaluation should succeed");
  assert_eq!(web_channel.hits.len(), 1);
  assert_eq!(web_channel.evaluated_rules, 1);
  assert_eq!(web_channel.score, 6.5);

  let other_channel =
    RuleEnginePort::evaluate_in_channel(&engine, "mobile", &valid_event_in_channel("mobile"))
      .await
      .expect("other channel evaluation should succeed");
  assert_eq!(other_channel.hits.len(), 0);
  assert_eq!(other_channel.evaluated_rules, 0);
  assert_eq!(other_channel.score, 0.0);
}

fn active_rule(
  code: &str,
  threshold_minor_units: u64,
  score: f32,
  action: RuleAction,
  channels: Option<&[&str]>,
) -> Rule {
  let scope = channels.map_or_else(RuleScope::all, |channels| {
    RuleScope::only(channels.iter().map(|channel| Channel::new(*channel).unwrap())).unwrap()
  });

  Rule::new(
    RuleId::new_v7(),
    RuleIdentity {
      code: Some(code.to_owned()),
      name: format!("{code} rule"),
      description: Some(format!("Triggers above {threshold_minor_units} minor units")),
      version: semver::Version::new(1, 0, 0),
      author: "runtime-test".to_owned(),
      tags: Some(vec!["runtime".to_owned()]),
    },
    scope,
    RulePolicy::new(
      RuleState::new(
        RuleMode::Active,
        RuleAudit {
          created_at_ms: TimestampMs::new(1_760_000_000_000).unwrap(),
          updated_at_ms: TimestampMs::new(1_760_000_000_001).unwrap(),
          created_by: Some("qa".to_owned()),
          updated_by: Some("qa".to_owned()),
        },
      )
      .unwrap(),
      RuleSchedule::new(None, None).unwrap(),
      RolloutPolicy::new(100).unwrap(),
    )
    .unwrap(),
    RuleDefinition::new(
      RuleEvaluation::new(
        RuleExpression::new(serde_json::json!(true)).unwrap(),
        RuleExpression::new(
          serde_json::json!({ ">": [{ "var": "payload.money.minor_units" }, threshold_minor_units] }),
        )
        .unwrap(),
      )
      .unwrap(),
    )
    .unwrap(),
    RuleDecision::new(RuleEnforcement {
      score_impact: Score::new(score).unwrap(),
      action,
      severity: Severity::High,
      tags: vec!["runtime_hit".to_owned()],
      cooldown_ms: None,
      functions: vec![],
    }),
  )
  .unwrap()
}

fn valid_event() -> Event {
  valid_event_in_channel("web")
}

fn valid_event_in_channel(channel: &str) -> Event {
  Event::new(
    Header {
      timestamp: Utc::now(),
      source: EventSource::new("api_gateway").unwrap(),
      event_id: None,
      instrument: None,
      channel: Some(Channel::new(channel).unwrap()),
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
    Payload::value_transfer(
      rve_core::domain::common::Money::from_major_str("100.0", Currency::new("USD").unwrap())
        .unwrap(),
      Parties {
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
      BTreeMap::new(),
    ),
  )
  .unwrap()
}
