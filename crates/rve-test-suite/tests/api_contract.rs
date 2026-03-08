use axum::{
  body::{Body, to_bytes},
  http::{Request, StatusCode, header},
  response::Response,
  Router,
};
use serde_json::{Value, json};
use tower::ServiceExt;

use rve::http::{build_router, state::AppState};

const RULE_ID: &str = "01952031-1a77-7f0c-9f3c-bfd27d450001";

async fn app() -> Router {
  build_router(AppState::new().await)
}

async fn response_json(response: Response) -> Value {
  let body = to_bytes(response.into_body(), usize::MAX).await.expect("read response body");
  serde_json::from_slice(&body).expect("parse response body")
}

fn valid_rule_payload() -> Value {
  json!({
    "meta": {
      "code": "RL01",
      "name": "High Value Payment",
      "description": "flags high value transaction",
      "version": "1.0.0",
      "author": "RiskOps",
      "tags": ["high_value", "payments"]
    },
    "state": {
      "mode": "active",
      "audit": {
        "created_at_ms": 1730000000000u64,
        "updated_at_ms": 1730000001000u64,
        "created_by": "alice",
        "updated_by": "alice"
      }
    },
    "schedule": {
      "active_from_ms": 1730000000000u64,
      "active_until_ms": 1731000000000u64
    },
    "rollout": { "percent": 50 },
    "evaluation": {
      "condition": true,
      "logic": { ">": [{"var": "payload.money.value"}, 1000] }
    },
    "enforcement": {
      "score_impact": 6.5,
      "action": "review",
      "severity": "high",
      "tags": ["financial_fraud"],
      "cooldown_ms": 60000
    }
  })
}

fn valid_event_payload() -> Value {
  json!({
    "header": {
      "timestamp": "2026-03-01T00:00:00Z",
      "source": "api_gateway",
      "event_id": "0195d80e-4f96-7a4b-a8e0-3c5a3f0e7b21",
      "instrument": "card",
      "channel": "web"
    },
    "context": {
      "geo": {
        "country": "US",
        "lon": -74.0,
        "lat": 40.7
      },
      "net": {
        "source_ip": "1.1.1.1"
      },
      "env": {
        "locale": "en-US",
        "timezone": "UTC",
        "device_id": "dev_001",
        "session_id": "sess_001"
      }
    },
    "features": {
      "fin": {
        "first_seen_at": 1730000000000u64,
        "last_seen_at": 1730000001000u64,
        "last_declined_at": 1730000000500u64,
        "total_successful_txns": 10u64,
        "total_declined_txns": 1u64,
        "total_amount_spent": 500000u64,
        "max_ticket_ever": 120000u64,
        "consecutive_failed_logins": 0,
        "consecutive_declines": 0,
        "current_hour_count": 1,
        "current_hour_amount": 1000u64,
        "current_day_count": 2,
        "current_day_amount": 2000u64,
        "known_ips": ["1.1.1.1"],
        "known_devices": ["dev_001"]
      }
    },
    "signals": {
      "flags": {
        "vpn": "no"
      }
    },
    "payload": {
      "money": {
        "value": 100.5,
        "ccy": "USD"
      },
      "parties": {
        "originator": {
          "entity_type": "individual",
          "acct": "acct_001",
          "country": "US",
          "watchlist": "no"
        },
        "beneficiary": {
          "entity_type": "business",
          "acct": "acct_002",
          "country": "US",
          "watchlist": "unknown"
        }
      },
      "extensions": {}
    }
  })
}

#[tokio::test]
async fn patch_returns_409_on_stale_if_match() {
  let app = app().await;

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

  let stale_patch_response = app
    .oneshot(
      Request::builder()
        .method("PATCH")
        .uri(format!("/api/v1/rules/{RULE_ID}"))
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::IF_MATCH, &if_match)
        .body(Body::from(json!({ "rollout": { "percent": 80 } }).to_string()))
        .expect("build stale patch request"),
    )
    .await
    .expect("stale patch response");

  assert_eq!(stale_patch_response.status(), StatusCode::CONFLICT);
  let body = response_json(stale_patch_response).await;
  assert_eq!(body["code"], "conflict");
}

#[tokio::test]
async fn rejects_unknown_fields_in_rule_payload() {
  let app = app().await;
  let mut payload = valid_rule_payload();
  payload["unknown"] = json!(true);

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/v1/rules")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(payload.to_string()))
        .expect("build request"),
    )
    .await
    .expect("rule create response");

  assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn rejects_invalid_schedule_in_rule_payload() {
  let app = app().await;
  let mut payload = valid_rule_payload();
  payload["schedule"]["active_until_ms"] = json!(1720000000000u64);

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/v1/rules")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(payload.to_string()))
        .expect("build request"),
    )
    .await
    .expect("rule create response");

  assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn rejects_legacy_expression_operators_in_rule_payload() {
  let app = app().await;
  let mut payload = valid_rule_payload();
  payload["evaluation"]["condition"] = json!({"=": [{"var": "payload.money.value"}, 1000]});
  payload["evaluation"]["logic"] = json!({
    "and": [
      {"not_in": [{"var": "payload.money.ccy"}, ["USD", "EUR"]]},
      {"=": [{"var": "features.fin.current_hour_count"}, 0]}
    ]
  });

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/v1/rules")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(payload.to_string()))
        .expect("build request"),
    )
    .await
    .expect("rule create response");

  assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn rejects_legacy_author_alias_in_rule_payload() {
  let app = app().await;
  let mut payload = valid_rule_payload();
  payload["meta"]["autor"] = json!("RiskOps");
  payload["meta"].as_object_mut().expect("meta object").remove("author");

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/v1/rules")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(payload.to_string()))
        .expect("build request"),
    )
    .await
    .expect("rule create response");

  assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn rejects_disallowed_var_roots_in_rule_payload() {
  let app = app().await;
  let mut payload = valid_rule_payload();
  payload["evaluation"]["logic"] = json!({">": [{"var": "config.latam_countries"}, 0]});

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/v1/rules")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(payload.to_string()))
        .expect("build request"),
    )
    .await
    .expect("rule create response");

  assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn accepts_direct_event_body() {
  let app = app().await;

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/v1/decisions")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(valid_event_payload().to_string()))
        .expect("build request"),
    )
    .await
    .expect("decision response");

  assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn rejects_old_wrapper_shape() {
  let app = app().await;
  let payload = json!({ "event": valid_event_payload() });

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/v1/decisions")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(payload.to_string()))
        .expect("build request"),
    )
    .await
    .expect("decision response");

  assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
  let body = to_bytes(response.into_body(), usize::MAX).await.expect("read response body");
  let body = String::from_utf8(body.to_vec()).expect("utf8 body");
  assert!(body.contains("unknown field `event`"), "unexpected body: {body}");
}

#[tokio::test]
async fn rejects_invalid_geo_latitude_with_422_mapping() {
  let app = app().await;
  let mut payload = valid_event_payload();
  payload["context"]["geo"]["lat"] = json!(123.0);

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/v1/decisions")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(payload.to_string()))
        .expect("build request"),
    )
    .await
    .expect("decision response");

  assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
  let body = response_json(response).await;
  assert_eq!(body["validation"]["errors"][0]["path"], "context.geo.lat");
}
