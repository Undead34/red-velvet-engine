use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{Value, json};

use rve_core::{ConnectorAck, ConnectorError, ConnectorKind, DataConnectorPort};

/// Registry that keeps track of every configured connector implementation.
pub struct ConnectorHub {
  connectors: HashMap<ConnectorKind, Arc<dyn DataConnectorPort>>,
}

impl ConnectorHub {
  pub fn empty() -> Self {
    Self { connectors: HashMap::new() }
  }

  pub fn with_default_connectors() -> Self {
    let mut hub = Self::empty();
    hub.register(Arc::new(RestConnector::default()));
    hub.register(Arc::new(KafkaConnector::new("fraud-events")));
    hub.register(Arc::new(RedisStreamConnector::new("fraud-events")));
    hub
  }

  pub fn register(&mut self, connector: Arc<dyn DataConnectorPort>) {
    self.connectors.insert(connector.kind(), connector);
  }

  pub async fn ingest(
    &self,
    kind: ConnectorKind,
    payload: Value,
  ) -> Result<ConnectorAck, ConnectorError> {
    let connector = self
      .connectors
      .get(&kind)
      .ok_or_else(|| ConnectorError::UnknownConnector(kind.to_string()))?;

    connector.ingest(payload).await
  }

  pub fn list(&self) -> Vec<ConnectorKind> {
    self.connectors.keys().copied().collect()
  }
}

#[derive(Default)]
pub struct RestConnector;

#[async_trait]
impl DataConnectorPort for RestConnector {
  fn kind(&self) -> ConnectorKind {
    ConnectorKind::Rest
  }

  async fn ingest(&self, payload: Value) -> Result<ConnectorAck, ConnectorError> {
    if !payload.is_object() {
      return Err(ConnectorError::invalid_payload("REST payload must be a JSON object"));
    }

    Ok(ConnectorAck::accepted(
      ConnectorKind::Rest,
      payload,
      json!({ "transport": "http", "adapter": "rest" }),
    ))
  }
}

pub struct KafkaConnector {
  topic: String,
}

impl KafkaConnector {
  pub fn new(topic: impl Into<String>) -> Self {
    Self { topic: topic.into() }
  }
}

#[async_trait]
impl DataConnectorPort for KafkaConnector {
  fn kind(&self) -> ConnectorKind {
    ConnectorKind::Kafka
  }

  async fn ingest(&self, payload: Value) -> Result<ConnectorAck, ConnectorError> {
    if !payload.is_object() {
      return Err(ConnectorError::invalid_payload("Kafka messages must be JSON objects"));
    }

    Ok(ConnectorAck::accepted(
      ConnectorKind::Kafka,
      payload,
      json!({ "transport": "kafka", "topic": self.topic }),
    ))
  }
}

pub struct RedisStreamConnector {
  stream: String,
}

impl RedisStreamConnector {
  pub fn new(stream: impl Into<String>) -> Self {
    Self { stream: stream.into() }
  }
}

#[async_trait]
impl DataConnectorPort for RedisStreamConnector {
  fn kind(&self) -> ConnectorKind {
    ConnectorKind::RedisStream
  }

  async fn ingest(&self, payload: Value) -> Result<ConnectorAck, ConnectorError> {
    if !payload.is_object() {
      return Err(ConnectorError::invalid_payload("Redis stream payload must be an object"));
    }

    Ok(ConnectorAck::accepted(
      ConnectorKind::RedisStream,
      payload,
      json!({ "transport": "redis_stream", "stream": self.stream }),
    ))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn rest_connector_accepts_objects() {
    let hub = ConnectorHub::with_default_connectors();
    let payload = json!({ "id": "test" });
    let ack = hub.ingest(ConnectorKind::Rest, payload.clone()).await.unwrap();
    assert_eq!(ack.connector, ConnectorKind::Rest);
    assert_eq!(ack.status, rve_core::ConnectorStatus::Accepted);
    assert_eq!(ack.payload, payload);
  }
}
