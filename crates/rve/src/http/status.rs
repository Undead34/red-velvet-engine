use axum::Json;
use serde_json::{Value, json};

use rve_core::PKG_VERSION;

pub async fn handler() -> Json<Value> {
  Json(json!({ "status": "pass", "version": PKG_VERSION }))
}
