use std::{collections::BTreeMap, sync::Arc};

use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use tokio::sync::RwLock;

use rve_core::domain::{
  common::{RuleId, Score, Severity, TimestampMs},
  rule::{mode::RuleMode, *},
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
  Rule {
    id: rule_id("01952031-1a77-7f0c-9f3c-bfd27d450001"),
    meta: RuleMeta {
      code: Some("FRAUD-HV-UNTRUSTED-01".into()),
      name: "High Value on Untrusted Device".into(),
      description: Some(
        "Dispara si el monto es > $5000 y el fingerprint del dispositivo es nuevo.".into(),
      ),
      version: semver::Version::new(1, 0, 0),
      autor: "Analista".into(),
      tags: Some(vec!["high_value".into(), "device".into()]),
    },
    state: RuleState {
      mode: RuleMode::Active,
      audit: RuleAudit {
        created_at_ms: TimestampMs::new(utc_ms(2024, 1, 2, 3, 4, 5)).expect("timestamp"),
        updated_at_ms: TimestampMs::new(utc_ms(2024, 2, 2, 3, 4, 5)).expect("timestamp"),
        created_by: Some("Super User".into()),
        updated_by: Some("Analyst Jane".into()),
      },
    },
    schedule: RuleSchedule {
      active_from_ms: Some(TimestampMs::new(utc_ms(2023, 10, 1, 0, 0, 0)).expect("timestamp")),
      active_until_ms: None,
    },
    rollout: RolloutPolicy { percent: 100 },
    evaluation: RuleEvaluation {
      condition: json!(true),
      logic: json!({
        "and": [
          { ">": [{ "var": "transaction.amount" }, 5000] },
          { "<": [{ "var": "device.trust_score" }, 0.4] }
        ]
      }),
    },
    enforcement: RuleEnforcement {
      score_impact: Score::new(8.5).expect("score"),
      action: RuleAction::Review,
      severity: Severity::High,
      tags: vec!["financial_fraud".into(), "device_fingerprinting".into()],
      cooldown_ms: Some(600_000),
    },
  }
}

fn velocity_flag() -> Rule {
  Rule {
    id: rule_id("01952031-1a77-7f0c-9f3c-bfd27d450002"),
    meta: RuleMeta {
      code: Some("FRAUD-VEL-RETRY-02".into()),
      name: "Velocity Retry Spike".into(),
      description: Some("Dispara si el cliente hace >3 intentos fallidos en 10 minutos".into()),
      version: semver::Version::new(1, 1, 0),
      autor: "Fraud Squad".into(),
      tags: Some(vec!["velocity".into(), "account_takeover".into()]),
    },
    state: RuleState {
      mode: RuleMode::Active,
      audit: RuleAudit {
        created_at_ms: TimestampMs::new(utc_ms(2023, 11, 5, 0, 0, 0)).expect("timestamp"),
        updated_at_ms: TimestampMs::new(utc_ms(2024, 1, 15, 12, 0, 0)).expect("timestamp"),
        created_by: Some("Automation".into()),
        updated_by: Some("Fraud Squad".into()),
      },
    },
    schedule: RuleSchedule { active_from_ms: None, active_until_ms: None },
    rollout: RolloutPolicy { percent: 75 },
    evaluation: RuleEvaluation {
      condition: json!({ ">": [{ "var": "context.fin.consecutive_declines" }, 0] }),
      logic: json!({
        "and": [
          { ">=": [{ "var": "context.fin.current_hour_count" }, 5] },
          { "<": [{ "var": "context.fin.last_seen_delta_minutes" }, 10] }
        ]
      }),
    },
    enforcement: RuleEnforcement {
      score_impact: Score::new(6.0).expect("score"),
      action: RuleAction::Review,
      severity: Severity::Moderate,
      tags: vec!["velocity".into(), "account_takeover".into()],
      cooldown_ms: Some(120_000),
    },
  }
}

fn utc_ms(year: i32, month: u32, day: u32, hour: u32, min: u32, sec: u32) -> u64 {
  Utc.with_ymd_and_hms(year, month, day, hour, min, sec).unwrap().timestamp_millis() as u64
}

fn rule_id(value: &str) -> RuleId {
  RuleId::try_from(value.to_owned()).expect("seed rule id")
}
