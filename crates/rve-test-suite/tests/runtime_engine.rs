use std::collections::{BTreeMap, HashSet};

use chrono::Utc;
use rve::engine::RVEngine;
use rve_core::{
  domain::{
    common::{AccountId, Currency, EventSource, Flag, RuleId, Score, Severity, TimestampMs},
    event::{
      Context, EnvironmentContext, Event, Features, FinancialFeatures, GeoContext, Header,
      NetworkContext, Parties, Party, Payload, Signals,
    },
    rule::{
      RolloutPolicy, Rule, RuleAction, RuleAudit, RuleDecision, RuleDefinition, RuleEnforcement,
      RuleEvaluation, RuleExpression, RuleIdentity, RuleMode, RulePolicy, RuleSchedule, RuleState,
    },
  },
  services::engine::{Decision, DecisionOutcome},
};
use serde_json::json;

#[test]
fn evaluates_rule_using_features_context() {
  let engine = RVEngine::new();
  engine.publish_rules(vec![valid_rule()]).expect("rules compile");

  let event = valid_event();
  let evaluation = engine.evaluate(&event).expect("evaluation succeeds");
  let decision = Decision::from_runtime(evaluation);

  assert_eq!(decision.hits.len(), 1);
  assert!(decision.score > 0.0);
  assert!(matches!(decision.outcome, DecisionOutcome::Review));
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
      functions: vec![],
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
