use async_trait::async_trait;

use rve_core::{
  domain::{event::Event, rule::Rule},
  ports::{RulesetSnapshot, RuntimeEngineError, RuntimeEnginePort, RuntimeEvaluation},
};

pub const RUNTIME_NOT_IMPLEMENTED_MESSAGE: &str = "runtime engine is not implemented yet";

#[derive(Default)]
pub struct RVEngine;

impl RVEngine {
  pub fn new() -> Self {
    Self
  }

  pub fn publish_rules(&self, _rules: Vec<Rule>) -> Result<RulesetSnapshot, RuntimeEngineError> {
    Err(RuntimeEngineError::NotImplemented {
      message: RUNTIME_NOT_IMPLEMENTED_MESSAGE.to_owned(),
    })
  }

  pub fn evaluate(&self, _event: &Event) -> Result<RuntimeEvaluation, RuntimeEngineError> {
    Err(RuntimeEngineError::NotImplemented {
      message: RUNTIME_NOT_IMPLEMENTED_MESSAGE.to_owned(),
    })
  }
}

#[async_trait]
impl RuntimeEnginePort for RVEngine {
  async fn publish_rules(&self, rules: Vec<Rule>) -> Result<RulesetSnapshot, RuntimeEngineError> {
    self.publish_rules(rules)
  }

  async fn evaluate(&self, event: &Event) -> Result<RuntimeEvaluation, RuntimeEngineError> {
    self.evaluate(event)
  }
}

#[cfg(test)]
mod tests {
  use std::collections::{BTreeMap, HashSet};

  use chrono::Utc;
  use rve_core::{
    domain::{
      common::{AccountId, Currency, EventSource, Flag},
      event::{
        Context, EnvironmentContext, Event, Features, FinancialFeatures, GeoContext, Header,
        NetworkContext, Parties, Party, Payload, Signals,
      },
    },
    ports::RuntimeEngineError,
  };

  use super::RVEngine;

  #[test]
  fn publish_rules_returns_not_implemented() {
    let engine = RVEngine::new();
    let error = engine.publish_rules(vec![]).expect_err("placeholder runtime must fail");

    assert!(matches!(error, RuntimeEngineError::NotImplemented { .. }));
  }

  #[test]
  fn evaluate_returns_not_implemented() {
    let engine = RVEngine::new();
    let error = engine.evaluate(&valid_event()).expect_err("placeholder runtime must fail");

    assert!(matches!(error, RuntimeEngineError::NotImplemented { .. }));
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
}
