use async_trait::async_trait;
use serde_json::Value;

use crate::{
  domain::{event::Event, rule::Rule},
  services::connector::{ConnectorAck, ConnectorError, ConnectorKind},
};

pub trait RuleExecutorPort {
  fn matches(&self, rule: &Rule, event: &Event) -> Result<bool, String>;
}

#[async_trait]
pub trait DataConnectorPort: Send + Sync {
  fn kind(&self) -> ConnectorKind;

  async fn ingest(&self, payload: Value) -> Result<ConnectorAck, ConnectorError>;
}
