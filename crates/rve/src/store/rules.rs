use async_trait::async_trait;
use redis::AsyncCommands;

use rve_core::{
  RuleRepositoryPort,
  domain::{common::RuleId, rule::Rule},
  ports::rule_repository::{RepositoryResult, RulePage, RuleRepositoryError},
};

#[derive(Clone)]
pub struct RedisRuleRepository {
  client: redis::Client,
  prefix: String,
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

  fn sorted_index_key(&self) -> String {
    format!("{}:rules:index:sorted", self.prefix)
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

  async fn ensure_sorted_index_populated(
    &self,
    conn: &mut redis::aio::MultiplexedConnection,
  ) -> RepositoryResult<()> {
    let sorted_key = self.sorted_index_key();
    let total: u64 = conn
      .zcard(&sorted_key)
      .await
      .map_err(|err| RuleRepositoryError::Storage(format!("redis zcard error: {err}")))?;
    if total > 0 {
      return Ok(());
    }

    let index_key = self.index_key();
    let mut ids: Vec<String> = conn
      .smembers(&index_key)
      .await
      .map_err(|err| RuleRepositoryError::Storage(format!("redis smembers error: {err}")))?;

    if ids.is_empty() {
      return Ok(());
    }

    ids.sort();
    for id in ids {
      let _: usize = conn
        .zadd(&sorted_key, &id, 0)
        .await
        .map_err(|err| RuleRepositoryError::Storage(format!("redis zadd error: {err}")))?;
    }

    Ok(())
  }
}

#[async_trait]
impl RuleRepositoryPort for RedisRuleRepository {
  async fn list(&self, page: u32, limit: u32) -> RepositoryResult<RulePage> {
    let page = page.max(1);
    let limit = limit.clamp(1, 100);

    let mut conn = self.connection().await?;
    self.ensure_sorted_index_populated(&mut conn).await?;

    let sorted_key = self.sorted_index_key();
    let total: u64 = conn
      .zcard(&sorted_key)
      .await
      .map_err(|err| RuleRepositoryError::Storage(format!("redis zcard error: {err}")))?;

    if total == 0 {
      return Ok(RulePage { items: Vec::new(), total: 0 });
    }

    let start = ((page - 1) * limit) as isize;
    let stop = start + (limit as isize) - 1;
    if start as u64 >= total {
      return Ok(RulePage { items: Vec::new(), total: total as u32 });
    }

    let ids: Vec<String> = conn
      .zrange(&sorted_key, start, stop)
      .await
      .map_err(|err| RuleRepositoryError::Storage(format!("redis zrange error: {err}")))?;

    let mut items = Vec::with_capacity(ids.len());
    for id in ids {
      let key = self.rule_key(&id);
      let raw: Option<String> = conn
        .get(key)
        .await
        .map_err(|err| RuleRepositoryError::Storage(format!("redis get error: {err}")))?;
      if let Some(raw) = raw {
        let rule = serde_json::from_str::<Rule>(&raw).map_err(|err| {
          RuleRepositoryError::Storage(format!("rule deserialization error: {err}"))
        })?;
        items.push(rule);
      }
    }

    Ok(RulePage { items, total: total as u32 })
  }

  async fn get(&self, id: &RuleId) -> RepositoryResult<Option<Rule>> {
    self.fetch_rule(id).await
  }

  async fn all(&self) -> RepositoryResult<Vec<Rule>> {
    let mut conn = self.connection().await?;
    self.ensure_sorted_index_populated(&mut conn).await?;
    let sorted_key = self.sorted_index_key();
    let ids: Vec<String> = conn
      .zrange(&sorted_key, 0, -1)
      .await
      .map_err(|err| RuleRepositoryError::Storage(format!("redis zrange error: {err}")))?;

    let mut rules = Vec::with_capacity(ids.len());
    for id in ids {
      let key = self.rule_key(&id);
      let raw: Option<String> = conn
        .get(key)
        .await
        .map_err(|err| RuleRepositoryError::Storage(format!("redis get error: {err}")))?;
      if let Some(raw) = raw {
        let rule = serde_json::from_str::<Rule>(&raw).map_err(|err| {
          RuleRepositoryError::Storage(format!("rule deserialization error: {err}"))
        })?;
        rules.push(rule);
      }
    }

    Ok(rules)
  }

  async fn create(&self, rule: Rule) -> RepositoryResult<Rule> {
    let id = rule.id.to_string();
    let key = self.rule_key(&id);
    let index_key = self.index_key();
    let sorted_key = self.sorted_index_key();
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
      .sadd::<_, _, ()>(&index_key, &id)
      .await
      .map_err(|err| RuleRepositoryError::Storage(format!("redis sadd error: {err}")))?;
    let _: usize = conn
      .zadd(&sorted_key, &id, 0)
      .await
      .map_err(|err| RuleRepositoryError::Storage(format!("redis zadd error: {err}")))?;

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
    let sorted_key = self.sorted_index_key();

    let mut conn = self.connection().await?;
    let removed: u64 = conn
      .srem(&index_key, &id_str)
      .await
      .map_err(|err| RuleRepositoryError::Storage(format!("redis srem error: {err}")))?;
    conn
      .del::<_, ()>(&key)
      .await
      .map_err(|err| RuleRepositoryError::Storage(format!("redis del error: {err}")))?;
    let _: usize = conn
      .zrem(&sorted_key, &id_str)
      .await
      .map_err(|err| RuleRepositoryError::Storage(format!("redis zrem error: {err}")))?;

    if removed == 0 {
      return Err(RuleRepositoryError::NotFound(id.clone()));
    }

    Ok(())
  }
}
