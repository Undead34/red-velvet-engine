use std::{collections::BTreeMap, sync::Arc};

use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use tokio::sync::RwLock;

use rve_core::domain::{
  common::{RuleId, Score, Severity, TimestampMs},
  rule::{RuleExpression, mode::RuleMode, *},
};
use rve_core::ports::{RepositoryResult, RulePage, RuleRepositoryError, RuleRepositoryPort};
use serde_json::json;

#[derive(Clone, Default)]
pub struct InMemoryRuleRepository {
  inner: Arc<RwLock<BTreeMap<RuleId, Rule>>>,
}

impl InMemoryRuleRepository {
  pub fn seeded() -> Self {
    Self::from_rules(default_rules())
  }

  pub fn from_rules(rules: Vec<Rule>) -> Self {
    let map = rules.into_iter().map(|rule| (rule.id.clone(), rule)).collect();
    Self { inner: Arc::new(RwLock::new(map)) }
  }
}

#[async_trait]
impl RuleRepositoryPort for InMemoryRuleRepository {
  async fn list(&self, page: u32, limit: u32) -> RepositoryResult<RulePage> {
    let page = page.max(1);
    let limit = limit.clamp(1, 100);

    let guard = self.inner.read().await;
    let total = guard.len() as u32;
    let start = ((page - 1) * limit) as usize;

    let items = guard.values().cloned().skip(start).take(limit as usize).collect();

    Ok(RulePage { items, total })
  }

  async fn get(&self, id: &RuleId) -> RepositoryResult<Option<Rule>> {
    let guard = self.inner.read().await;
    Ok(guard.get(id).cloned())
  }

  async fn all(&self) -> RepositoryResult<Vec<Rule>> {
    let guard = self.inner.read().await;
    Ok(guard.values().cloned().collect())
  }

  async fn create(&self, rule: Rule) -> RepositoryResult<Rule> {
    let mut guard = self.inner.write().await;
    if guard.contains_key(&rule.id) {
      return Err(RuleRepositoryError::AlreadyExists(rule.id));
    }
    guard.insert(rule.id.clone(), rule.clone());
    Ok(rule)
  }

  async fn replace(&self, rule: Rule) -> RepositoryResult<Rule> {
    let mut guard = self.inner.write().await;
    if !guard.contains_key(&rule.id) {
      return Err(RuleRepositoryError::NotFound(rule.id));
    }
    guard.insert(rule.id.clone(), rule.clone());
    Ok(rule)
  }

  async fn delete(&self, id: &RuleId) -> RepositoryResult<()> {
    let mut guard = self.inner.write().await;
    guard.remove(id).map(|_| ()).ok_or_else(|| RuleRepositoryError::NotFound(id.clone()))
  }
}

fn default_rules() -> Vec<Rule> {
  vec![high_value_untrusted_device(), velocity_flag()]
}

fn high_value_untrusted_device() -> Rule {
  Rule::new(
    rule_id("01952031-1a77-7f0c-9f3c-bfd27d450001"),
    RuleMeta {
      code: Some("FRAUD-HV-UNTRUSTED-01".into()),
      name: "High Value on Untrusted Device".into(),
      description: Some(
        "Dispara si el monto es > $5000 y el fingerprint del dispositivo es nuevo.".into(),
      ),
      version: semver::Version::new(1, 0, 0),
      author: "Analista".into(),
      tags: Some(vec!["high_value".into(), "device".into()]),
    },
    RulePolicy::new(
      RuleState::new(
        RuleMode::Active,
        RuleAudit {
          created_at_ms: TimestampMs::new(utc_ms(2024, 1, 2, 3, 4, 5)).expect("timestamp"),
          updated_at_ms: TimestampMs::new(utc_ms(2024, 2, 2, 3, 4, 5)).expect("timestamp"),
          created_by: Some("Super User".into()),
          updated_by: Some("Analyst Jane".into()),
        },
      )
      .expect("seed state"),
      RuleSchedule::new(
        Some(TimestampMs::new(utc_ms(2023, 10, 1, 0, 0, 0)).expect("timestamp")),
        None,
      )
      .expect("seed schedule"),
      RolloutPolicy::new(100).expect("seed rollout"),
    )
    .expect("seed policy"),
    RuleDefinition::new(
      RuleEvaluation::new(
        RuleExpression::new(json!(true)).expect("seed condition"),
        RuleExpression::new(json!({
          "and": [
            { ">": [{ "var": "transaction.amount" }, 5000] },
            { "<": [{ "var": "device.trust_score" }, 0.4] }
          ]
        }))
        .expect("seed logic"),
      )
      .expect("seed evaluation"),
    )
    .expect("seed definition"),
    RuleDecision::new(RuleEnforcement {
      score_impact: Score::new(8.5).expect("score"),
      action: RuleAction::Review,
      severity: Severity::High,
      tags: vec!["financial_fraud".into(), "device_fingerprinting".into()],
      cooldown_ms: Some(600_000),
    }),
  )
  .expect("seed rule")
}

fn velocity_flag() -> Rule {
  Rule::new(
    rule_id("01952031-1a77-7f0c-9f3c-bfd27d450002"),
    RuleMeta {
      code: Some("FRAUD-VEL-RETRY-02".into()),
      name: "Velocity Retry Spike".into(),
      description: Some("Dispara si el cliente hace >3 intentos fallidos en 10 minutos".into()),
      version: semver::Version::new(1, 1, 0),
      author: "Fraud Squad".into(),
      tags: Some(vec!["velocity".into(), "account_takeover".into()]),
    },
    RulePolicy::new(
      RuleState::new(
        RuleMode::Active,
        RuleAudit {
          created_at_ms: TimestampMs::new(utc_ms(2023, 11, 5, 0, 0, 0)).expect("timestamp"),
          updated_at_ms: TimestampMs::new(utc_ms(2024, 1, 15, 12, 0, 0)).expect("timestamp"),
          created_by: Some("Automation".into()),
          updated_by: Some("Fraud Squad".into()),
        },
      )
      .expect("seed state"),
      RuleSchedule::new(None, None).expect("seed schedule"),
      RolloutPolicy::new(75).expect("seed rollout"),
    )
    .expect("seed policy"),
    RuleDefinition::new(
      RuleEvaluation::new(
        RuleExpression::new(json!({ ">": [{ "var": "features.fin.consecutive_declines" }, 0] }))
          .expect("seed condition"),
        RuleExpression::new(json!({
          "and": [
            { ">=": [{ "var": "features.fin.current_hour_count" }, 5] },
            { "<": [{ "var": "features.fin.last_seen_delta_minutes" }, 10] }
          ]
        }))
        .expect("seed logic"),
      )
      .expect("seed evaluation"),
    )
    .expect("seed definition"),
    RuleDecision::new(RuleEnforcement {
      score_impact: Score::new(6.0).expect("score"),
      action: RuleAction::Review,
      severity: Severity::Moderate,
      tags: vec!["velocity".into(), "account_takeover".into()],
      cooldown_ms: Some(120_000),
    }),
  )
  .expect("seed rule")
}

fn utc_ms(year: i32, month: u32, day: u32, hour: u32, min: u32, sec: u32) -> u64 {
  Utc.with_ymd_and_hms(year, month, day, hour, min, sec).unwrap().timestamp_millis() as u64
}

fn rule_id(value: &str) -> RuleId {
  RuleId::try_from(value.to_owned()).expect("seed rule id")
}
