use axum::{
  body::{Body, to_bytes},
  http::{Request, StatusCode, header},
};
use serde_json::{Value, json};
use tower::ServiceExt;

use rve::http::{build_router, state::AppState};

const RULE_ID: &str = "01952031-1a77-7f0c-9f3c-bfd27d450001";

#[tokio::test]
async fn patch_returns_409_on_stale_if_match() {
  let app = build_router(AppState::new().await);

  let get_response = app
    .clone()
    .oneshot(
      Request::builder()
        .method("GET")
        .uri(format!("/api/v1/rules/{RULE_ID}"))
        .body(Body::empty())
        .expect("build get request"),
    )
    .await
    .expect("get rule response");

  assert_eq!(get_response.status(), StatusCode::OK);

  let version = get_response
    .headers()
    .get("x-rule-version")
    .and_then(|value| value.to_str().ok())
    .expect("version header from GET")
    .to_owned();
  let if_match = format!("\"{version}\"");

  let first_patch = json!({
    "state": {
      "audit": {
        "updated_by": "integration-test",
        "updated_at_ms": 1706843045001u64
      }
    }
  });

  let first_patch_response = app
    .clone()
    .oneshot(
      Request::builder()
        .method("PATCH")
        .uri(format!("/api/v1/rules/{RULE_ID}"))
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::IF_MATCH, &if_match)
        .body(Body::from(first_patch.to_string()))
        .expect("build first patch request"),
    )
    .await
    .expect("first patch response");

  assert_eq!(first_patch_response.status(), StatusCode::OK);

  let stale_patch = json!({
    "rollout": { "percent": 80 }
  });

  let stale_patch_response = app
    .oneshot(
      Request::builder()
        .method("PATCH")
        .uri(format!("/api/v1/rules/{RULE_ID}"))
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::IF_MATCH, &if_match)
        .body(Body::from(stale_patch.to_string()))
        .expect("build stale patch request"),
    )
    .await
    .expect("stale patch response");

  assert_eq!(stale_patch_response.status(), StatusCode::CONFLICT);

  let body =
    to_bytes(stale_patch_response.into_body(), usize::MAX).await.expect("read stale patch body");
  let json: Value = serde_json::from_slice(&body).expect("parse stale patch body");

  assert_eq!(json["code"], "conflict");
}
