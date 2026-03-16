use std::{collections::BTreeMap, sync::Arc};

use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use redis::AsyncCommands;
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

#[derive(Clone)]
pub struct RedisRuleRepository {
  client: redis::Client,
  prefix: String,
}

impl InMemoryRuleRepository {
  pub fn seeded() -> Self {
    Self::from_rules(seed_rules())
  }

  pub fn from_rules(rules: Vec<Rule>) -> Self {
    let map = rules.into_iter().map(|rule| (rule.id.clone(), rule)).collect();
    Self { inner: Arc::new(RwLock::new(map)) }
  }
}

impl RedisRuleRepository {
  pub fn new(redis_url: &str, prefix: impl Into<String>) -> RepositoryResult<Self> {
    let client = redis::Client::open(redis_url)
      .map_err(|err| RuleRepositoryError::Storage(format!("redis client error: {err}")))?;
    Ok(Self { client, prefix: prefix.into() })
  }

  fn index_key(&self) -> String {
    format!("{}:rules:index", self.prefix)
  }

  fn rule_key(&self, id: &str) -> String {
    format!("{}:rules:{id}", self.prefix)
  }

  async fn connection(&self) -> RepositoryResult<redis::aio::MultiplexedConnection> {
    self
      .client
      .get_multiplexed_async_connection()
      .await
      .map_err(|err| RuleRepositoryError::Storage(format!("redis connection error: {err}")))
  }

  async fn fetch_rule(&self, id: &RuleId) -> RepositoryResult<Option<Rule>> {
    let key = self.rule_key(&id.to_string());
    let mut conn = self.connection().await?;
    let raw: Option<String> = conn
      .get(key)
      .await
      .map_err(|err| RuleRepositoryError::Storage(format!("redis get error: {err}")))?;

    raw
      .map(|json| serde_json::from_str::<Rule>(&json))
      .transpose()
      .map_err(|err| RuleRepositoryError::Storage(format!("rule deserialization error: {err}")))
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

#[async_trait]
impl RuleRepositoryPort for RedisRuleRepository {
  async fn list(&self, page: u32, limit: u32) -> RepositoryResult<RulePage> {
    let page = page.max(1);
    let limit = limit.clamp(1, 100);

    let mut conn = self.connection().await?;
    let index_key = self.index_key();
    let mut ids: Vec<String> = conn
      .smembers(&index_key)
      .await
      .map_err(|err| RuleRepositoryError::Storage(format!("redis smembers error: {err}")))?;
    ids.sort();

    let total = ids.len() as u32;
    let start = ((page - 1) * limit) as usize;
    let selected = ids.into_iter().skip(start).take(limit as usize).collect::<Vec<_>>();

    let mut items = Vec::with_capacity(selected.len());
    for id in selected {
      let key = self.rule_key(&id);
      let raw: Option<String> = conn
        .get(key)
        .await
        .map_err(|err| RuleRepositoryError::Storage(format!("redis get error: {err}")))?;
      if let Some(raw) = raw {
        let rule = serde_json::from_str::<Rule>(&raw)
          .map_err(|err| RuleRepositoryError::Storage(format!("rule deserialization error: {err}")))?;
        items.push(rule);
      }
    }

    Ok(RulePage { items, total })
  }

  async fn get(&self, id: &RuleId) -> RepositoryResult<Option<Rule>> {
    self.fetch_rule(id).await
  }

  async fn all(&self) -> RepositoryResult<Vec<Rule>> {
    let mut conn = self.connection().await?;
    let index_key = self.index_key();
    let mut ids: Vec<String> = conn
      .smembers(&index_key)
      .await
      .map_err(|err| RuleRepositoryError::Storage(format!("redis smembers error: {err}")))?;
    ids.sort();

    let mut rules = Vec::with_capacity(ids.len());
    for id in ids {
      let key = self.rule_key(&id);
      let raw: Option<String> = conn
        .get(key)
        .await
        .map_err(|err| RuleRepositoryError::Storage(format!("redis get error: {err}")))?;
      if let Some(raw) = raw {
        let rule = serde_json::from_str::<Rule>(&raw)
          .map_err(|err| RuleRepositoryError::Storage(format!("rule deserialization error: {err}")))?;
        rules.push(rule);
      }
    }

    Ok(rules)
  }

  async fn create(&self, rule: Rule) -> RepositoryResult<Rule> {
    let id = rule.id.to_string();
    let key = self.rule_key(&id);
    let index_key = self.index_key();
    let payload = serde_json::to_string(&rule)
      .map_err(|err| RuleRepositoryError::Storage(format!("rule serialization error: {err}")))?;

    let mut conn = self.connection().await?;
    let exists: bool = conn
      .sismember(&index_key, &id)
      .await
      .map_err(|err| RuleRepositoryError::Storage(format!("redis sismember error: {err}")))?;
    if exists {
      return Err(RuleRepositoryError::AlreadyExists(rule.id));
    }

    conn
      .set::<_, _, ()>(&key, payload)
      .await
      .map_err(|err| RuleRepositoryError::Storage(format!("redis set error: {err}")))?;
    conn
      .sadd::<_, _, ()>(&index_key, id)
      .await
      .map_err(|err| RuleRepositoryError::Storage(format!("redis sadd error: {err}")))?;

    Ok(rule)
  }

  async fn replace(&self, rule: Rule) -> RepositoryResult<Rule> {
    let id = rule.id.to_string();
    let key = self.rule_key(&id);
    let index_key = self.index_key();
    let payload = serde_json::to_string(&rule)
      .map_err(|err| RuleRepositoryError::Storage(format!("rule serialization error: {err}")))?;

    let mut conn = self.connection().await?;
    let exists: bool = conn
      .sismember(&index_key, &id)
      .await
      .map_err(|err| RuleRepositoryError::Storage(format!("redis sismember error: {err}")))?;
    if !exists {
      return Err(RuleRepositoryError::NotFound(rule.id));
    }

    conn
      .set::<_, _, ()>(&key, payload)
      .await
      .map_err(|err| RuleRepositoryError::Storage(format!("redis set error: {err}")))?;

    Ok(rule)
  }

  async fn delete(&self, id: &RuleId) -> RepositoryResult<()> {
    let id_str = id.to_string();
    let key = self.rule_key(&id_str);
    let index_key = self.index_key();

    let mut conn = self.connection().await?;
    let removed: u64 = conn
      .srem(&index_key, &id_str)
      .await
      .map_err(|err| RuleRepositoryError::Storage(format!("redis srem error: {err}")))?;
    conn
      .del::<_, ()>(&key)
      .await
      .map_err(|err| RuleRepositoryError::Storage(format!("redis del error: {err}")))?;

    if removed == 0 {
      return Err(RuleRepositoryError::NotFound(id.clone()));
    }

    Ok(())
  }
}

pub fn seed_rules() -> Vec<Rule> {
  vec![high_value_untrusted_device(), velocity_flag()]
}

fn high_value_untrusted_device() -> Rule {
  Rule::new(
    rule_id("01952031-1a77-7f0c-9f3c-bfd27d450001"),
    RuleIdentity {
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
      functions: vec![],
    }),
  )
  .expect("seed rule")
}

fn velocity_flag() -> Rule {
  Rule::new(
    rule_id("01952031-1a77-7f0c-9f3c-bfd27d450002"),
    RuleIdentity {
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
      functions: vec![],
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
