use std::sync::Arc;

use crate::{engine::RVEngine, store::RedisStore};

#[derive(Clone)]
pub struct AppState {
  pub engine: RVEngine,
  pub store: Arc<RedisStore>,
}

impl AppState {
  pub fn new() -> Self {
    Self { engine: RVEngine::new(), store: Arc::new(RedisStore::new()) }
  }
}
