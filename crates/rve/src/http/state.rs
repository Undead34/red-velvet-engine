use std::sync::Arc;

use crate::{connector::ConnectorHub, engine::RVEngine, store::RedisStore};

#[derive(Clone)]
pub struct AppState {
  pub engine: RVEngine,
  pub store: Arc<RedisStore>,
  pub connectors: Arc<ConnectorHub>,
}

impl AppState {
  pub fn new() -> Self {
    let connectors = Arc::new(ConnectorHub::with_default_connectors());
    Self { engine: RVEngine::new(), store: Arc::new(RedisStore::new()), connectors }
  }
}
