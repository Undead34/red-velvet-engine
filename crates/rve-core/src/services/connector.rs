use std::fmt::{Display, Formatter};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use thiserror::Error;

/// Identifies the transport/source that is submitting an event payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectorKind {
  Rest,
  Kafka,
  RedisStream,
}

impl ConnectorKind {
  pub const fn as_str(&self) -> &'static str {
    match self {
      ConnectorKind::Rest => "rest",
      ConnectorKind::Kafka => "kafka",
      ConnectorKind::RedisStream => "redis_stream",
    }
  }
}

impl Display for ConnectorKind {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.as_str())
  }
}

impl std::str::FromStr for ConnectorKind {
  type Err = ConnectorError;

  fn from_str(value: &str) -> Result<Self, Self::Err> {
    match value.trim().to_lowercase().as_str() {
      "rest" => Ok(ConnectorKind::Rest),
      "kafka" => Ok(ConnectorKind::Kafka),
      "redis" | "redis_stream" | "redis-stream" => Ok(ConnectorKind::RedisStream),
      other => Err(ConnectorError::UnknownConnector(other.to_string())),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectorStatus {
  Accepted,
  Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorAck {
  pub connector: ConnectorKind,
  pub status: ConnectorStatus,
  pub received_at: DateTime<Utc>,
  pub message: String,
  pub metadata: Value,
  pub payload: Value,
}

impl ConnectorAck {
  pub fn accepted(connector: ConnectorKind, payload: Value, metadata: Value) -> Self {
    Self {
      connector,
      status: ConnectorStatus::Accepted,
      received_at: Utc::now(),
      message: "payload accepted".to_string(),
      metadata,
      payload,
    }
  }

  pub fn rejected(connector: ConnectorKind, reason: &str, payload: Value) -> Self {
    Self {
      connector,
      status: ConnectorStatus::Rejected,
      received_at: Utc::now(),
      message: reason.to_string(),
      metadata: json!({}),
      payload,
    }
  }
}

#[derive(Debug, Error)]
pub enum ConnectorError {
  #[error("connector `{0}` is not registered")]
  UnknownConnector(String),

  #[error("connector {0} is disabled")]
  Disabled(ConnectorKind),

  #[error("invalid payload: {0}")]
  InvalidPayload(String),

  #[error("ingestion error: {0}")]
  Processing(String),
}

impl ConnectorError {
  pub fn invalid_payload(msg: impl Into<String>) -> Self {
    ConnectorError::InvalidPayload(msg.into())
  }

  pub fn processing(msg: impl Into<String>) -> Self {
    ConnectorError::Processing(msg.into())
  }
}
