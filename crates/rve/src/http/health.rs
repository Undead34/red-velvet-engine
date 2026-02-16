use axum::{http::HeaderMap, response::IntoResponse};
use serde_json::json;

pub async fn handler() -> impl IntoResponse {
  let mut headers = HeaderMap::new();
  headers.insert("X-Miku", "39".parse().unwrap()); // Miku says: thank you

  let body = axum::Json(json!({ "status": "ok" }));

  (headers, body)
}
