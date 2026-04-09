use std::sync::{Arc, RwLock as StdRwLock};

use async_trait::async_trait;
use axum::{
  Router,
  body::{Body, to_bytes},
  http::{Request, StatusCode, header},
  response::Response,
};
use serde_json::{Value, json};
use tokio::sync::RwLock as AsyncRwLock;
use tower::ServiceExt;

use rve::{
  engine::DataflowRuleEngine,
  http::{build_router, state::AppState},
};
use rve_core::{
  domain::{common::RuleId, event::Event, rule::Rule},
  ports::{
    rule_engine::{
      RuleCompileStats, RuleEngineExecution, RuleEnginePort, RuleEngineStatus, RuleEngineTrace,
      RulesetSnapshot, RuntimeEngineError, RuntimeEvaluation,
    },
    rule_repository::{RepositoryResult, RulePage, RuleRepositoryError, RuleRepositoryPort},
  },
};

async fn app() -> Router {
  build_router(test_state())
}

async fn runtime_app() -> Router {
  build_router(runtime_state())
}

fn test_state() -> AppState {
  AppState {
    rule_engine: Arc::new(TestRuleEngine::default()),
    rule_repo: Arc::new(InMemoryRuleRepository::default()),
  }
}

fn runtime_state() -> AppState {
  AppState {
    rule_engine: Arc::new(DataflowRuleEngine::new()),
    rule_repo: Arc::new(InMemoryRuleRepository::default()),
  }
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
      "logic": { ">": [{"var": "payload.money.minor_units"}, 100000] }
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
  event_payload_in_channel("web")
}

fn event_payload_in_channel(channel: &str) -> Value {
  json!({
    "header": {
      "timestamp": "2026-03-01T00:00:00Z",
      "source": "api_gateway",
      "event_id": "0195d80e-4f96-7a4b-a8e0-3c5a3f0e7b21",
      "instrument": "card",
      "channel": channel
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
        "minor_units": 10050,
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

fn runtime_rule_payload() -> Value {
  json!({
    "meta": {
      "code": "RL-RUNTIME-001",
      "name": "Runtime High Value Payment",
      "description": "flags runtime high value transaction",
      "version": "1.0.0",
      "author": "RiskOps",
      "tags": ["runtime", "payments"]
    },
    "scope": {
      "channels": ["web"]
    },
    "state": {
      "mode": "active",
      "audit": {
        "created_at_ms": 1760000000000u64,
        "updated_at_ms": 1760000001000u64,
        "created_by": "alice",
        "updated_by": "alice"
      }
    },
    "schedule": {
      "active_from_ms": Value::Null,
      "active_until_ms": Value::Null
    },
    "rollout": { "percent": 100 },
    "evaluation": {
      "condition": true,
      "logic": { ">": [{"var": "payload.money.minor_units"}, 10000] }
    },
    "enforcement": {
      "score_impact": 6.5,
      "action": "review",
      "severity": "high",
      "tags": ["financial_fraud"],
      "cooldown_ms": 60000,
      "functions": []
    }
  })
}

#[tokio::test]
async fn patch_returns_409_on_stale_if_match() {
  let app = app().await;

  let create_response = app
    .clone()
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/v1/rules")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(valid_rule_payload().to_string()))
        .expect("build create request"),
    )
    .await
    .expect("create rule response");

  assert_eq!(create_response.status(), StatusCode::CREATED);
  let created_rule = response_json(create_response).await;
  let rule_id = created_rule["id"].as_str().expect("created rule id").to_owned();

  let get_response = app
    .clone()
    .oneshot(
      Request::builder()
        .method("GET")
        .uri(format!("/api/v1/rules/{rule_id}"))
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
        "updated_at_ms": 1730000002000u64
      }
    }
  });

  let first_patch_response = app
    .clone()
    .oneshot(
      Request::builder()
        .method("PATCH")
        .uri(format!("/api/v1/rules/{rule_id}"))
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
        .uri(format!("/api/v1/rules/{rule_id}"))
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
  payload["evaluation"]["condition"] = json!({"=": [{"var": "payload.money.minor_units"}, 100000]});
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
async fn decisions_returns_runtime_evaluation_for_valid_json() {
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
  let body = response_json(response).await;
  assert!(body.get("score").is_some());
  assert!(body.get("outcome").is_some());
}

#[tokio::test]
async fn docs_alias_serves_api_reference() {
  let app = app().await;

  let response = app
    .oneshot(
      Request::builder()
        .method("GET")
        .uri("/docs")
        .body(Body::empty())
        .expect("build docs request"),
    )
    .await
    .expect("docs response");

  assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn responses_propagate_request_id_header() {
  let app = app().await;

  let response = app
    .oneshot(
      Request::builder()
        .method("GET")
        .uri("/health")
        .body(Body::empty())
        .expect("build health request"),
    )
    .await
    .expect("health response");

  assert_eq!(response.status(), StatusCode::OK);
  let request_id = response.headers().get("x-request-id").expect("request id header");
  assert!(!request_id.to_str().expect("request id text").is_empty());
}

#[tokio::test]
async fn decisions_rejects_invalid_wrapper_shape() {
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
  let body = response_json(response).await;
  assert_eq!(body["code"], "unprocessable_entity");
}

#[tokio::test]
async fn engine_status_reports_runtime_state() {
  let app = app().await;
  let list_response = app
    .clone()
    .oneshot(
      Request::builder()
        .method("GET")
        .uri("/api/v1/rules")
        .body(Body::empty())
        .expect("build rules request"),
    )
    .await
    .expect("rules response");
  let list_body = response_json(list_response).await;
  let repository_rules = list_body["data"].as_array().expect("rules array").len() as u64;

  let response = app
    .oneshot(
      Request::builder()
        .method("GET")
        .uri("/api/v1/engine/status")
        .body(Body::empty())
        .expect("build request"),
    )
    .await
    .expect("status response");

  assert_eq!(response.status(), StatusCode::OK);
  let body = response_json(response).await;
  assert_eq!(body["mode"], "dataflow-rs");
  assert_eq!(body["ready"], false);
  assert_eq!(body["loaded_rules"], 0);
  assert_eq!(body["repository_rules"], repository_rules);
}

#[tokio::test]
async fn engine_reload_compiles_rules() {
  let app = app().await;

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/v1/engine/reload")
        .body(Body::empty())
        .expect("build request"),
    )
    .await
    .expect("reload response");

  assert_eq!(response.status(), StatusCode::OK);
  let body = response_json(response).await;
  assert!(body["version"].as_u64().is_some());
  assert!(body["loaded_rules"].as_u64().is_some());
}

#[tokio::test]
async fn create_rule_reload_and_decide_uses_real_runtime_with_channel_scope() {
  let app = runtime_app().await;

  let create_response = app
    .clone()
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/v1/rules")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(runtime_rule_payload().to_string()))
        .expect("build create request"),
    )
    .await
    .expect("create rule response");

  assert_eq!(create_response.status(), StatusCode::CREATED);

  let reload_response = app
    .clone()
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/v1/engine/reload")
        .body(Body::empty())
        .expect("build reload request"),
    )
    .await
    .expect("reload response");

  assert_eq!(reload_response.status(), StatusCode::OK);
  let reload_body = response_json(reload_response).await;
  assert_eq!(reload_body["loaded_rules"], 1);

  let decision_response = app
    .clone()
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/v1/decisions")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(valid_event_payload().to_string()))
        .expect("build decision request"),
    )
    .await
    .expect("decision response");

  assert_eq!(decision_response.status(), StatusCode::OK);
  let decision_body = response_json(decision_response).await;
  assert_eq!(decision_body["outcome"], "review");
  assert_eq!(decision_body["executed_rules"], 1);
  assert_eq!(decision_body["evaluated_rules"], 1);
  assert_eq!(decision_body["hits"].as_array().expect("hits array").len(), 1);

  let score = decision_body["score"].as_f64().expect("numeric score");
  assert!((score - 6.5).abs() < f64::EPSILON);

  let mobile_response = app
    .clone()
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/v1/decisions")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(event_payload_in_channel("mobile").to_string()))
        .expect("build mobile decision request"),
    )
    .await
    .expect("mobile decision response");

  assert_eq!(mobile_response.status(), StatusCode::OK);
  let mobile_body = response_json(mobile_response).await;
  assert_eq!(mobile_body["outcome"], "allow");
  assert_eq!(mobile_body["executed_rules"], 0);
  assert_eq!(mobile_body["evaluated_rules"], 0);

  let trace_response = app
    .clone()
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/v1/decisions/trace")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(valid_event_payload().to_string()))
        .expect("build decision trace request"),
    )
    .await
    .expect("decision trace response");

  assert_eq!(trace_response.status(), StatusCode::OK);
  let trace_body = response_json(trace_response).await;
  assert_eq!(trace_body["decision"]["outcome"], "review");
  assert_eq!(trace_body["trace"]["channel"], "web");
  let trace_steps = trace_body["trace"]["steps"].as_array().expect("trace steps");
  assert!(!trace_steps.is_empty());
  assert_eq!(trace_steps[0]["rule_id"], created_rule_id(&trace_body));
  assert_eq!(trace_steps[0]["runtime_channel"], "web");

  let status_response = app
    .oneshot(
      Request::builder()
        .method("GET")
        .uri("/api/v1/engine/status")
        .body(Body::empty())
        .expect("build status request"),
    )
    .await
    .expect("status response");

  assert_eq!(status_response.status(), StatusCode::OK);
  let status_body = response_json(status_response).await;
  assert_eq!(status_body["ready"], true);
  assert_eq!(status_body["repository_rules"], 1);
  assert_eq!(status_body["loaded_rules"], 1);
}

#[derive(Clone, Default)]
struct InMemoryRuleRepository {
  rules: Arc<AsyncRwLock<Vec<Rule>>>,
}

fn created_rule_id(trace_body: &Value) -> Value {
  trace_body["decision"]["hits"][0]["rule_id"].clone()
}

#[async_trait]
impl RuleRepositoryPort for InMemoryRuleRepository {
  async fn list(&self, page: u32, limit: u32) -> RepositoryResult<RulePage> {
    let page = page.max(1);
    let limit = limit.clamp(1, 100);
    let rules = self.rules.read().await;
    let total = rules.len() as u32;
    let start = ((page - 1) * limit) as usize;
    if start >= rules.len() {
      return Ok(RulePage { items: Vec::new(), total });
    }
    let end = usize::min(start + limit as usize, rules.len());
    Ok(RulePage { items: rules[start..end].to_vec(), total })
  }

  async fn get(&self, id: &RuleId) -> RepositoryResult<Option<Rule>> {
    let rules = self.rules.read().await;
    Ok(rules.iter().find(|rule| rule.id == *id).cloned())
  }

  async fn all(&self) -> RepositoryResult<Vec<Rule>> {
    let rules = self.rules.read().await;
    Ok(rules.clone())
  }

  async fn create(&self, rule: Rule) -> RepositoryResult<Rule> {
    let mut rules = self.rules.write().await;
    if rules.iter().any(|existing| existing.id == rule.id) {
      return Err(RuleRepositoryError::AlreadyExists(rule.id.clone()));
    }
    rules.push(rule.clone());
    Ok(rule)
  }

  async fn replace(&self, rule: Rule) -> RepositoryResult<Rule> {
    let id = rule.id.clone();
    let mut rules = self.rules.write().await;
    if let Some(existing) = rules.iter_mut().find(|existing| existing.id == id) {
      *existing = rule.clone();
      return Ok(rule);
    }
    Err(RuleRepositoryError::NotFound(id))
  }

  async fn delete(&self, id: &RuleId) -> RepositoryResult<()> {
    let mut rules = self.rules.write().await;
    let before = rules.len();
    let target = id.clone();
    rules.retain(|rule| rule.id != target);
    if rules.len() == before {
      return Err(RuleRepositoryError::NotFound(id.clone()));
    }
    Ok(())
  }
}

#[derive(Clone, Default)]
struct TestRuleEngine {
  state: Arc<StdRwLock<TestRuleEngineState>>,
}

#[derive(Clone, Default)]
struct TestRuleEngineState {
  version: u64,
  ready: bool,
  rules: Vec<Rule>,
}

#[async_trait]
impl RuleEnginePort for TestRuleEngine {
  async fn publish_rules(&self, rules: Vec<Rule>) -> Result<RulesetSnapshot, RuntimeEngineError> {
    let mut state = self.state.write().expect("test runtime lock");
    state.version = state.version.saturating_add(1);
    state.ready = true;
    state.rules = rules.clone();

    let stats = RuleCompileStats {
      total_rules: rules.len() as u32,
      compiled_rules: rules.len() as u32,
      failed_rules: 0,
    };

    Ok(RulesetSnapshot {
      version: state.version,
      loaded_rules: stats.compiled_rules,
      compile_stats: stats,
    })
  }

  async fn evaluate(&self, _event: &Event) -> Result<RuntimeEvaluation, RuntimeEngineError> {
    let state = self.state.read().expect("test runtime lock");
    Ok(RuntimeEvaluation {
      score: 0.0,
      hits: Vec::new(),
      evaluated_rules: state.rules.len() as u32,
      rollout_bucket: 0,
    })
  }

  async fn evaluate_with_trace(
    &self,
    event: &Event,
  ) -> Result<RuleEngineExecution, RuntimeEngineError> {
    let evaluation = self.evaluate(event).await?;
    Ok(RuleEngineExecution {
      evaluation,
      trace: RuleEngineTrace { channel: None, steps: Vec::new() },
    })
  }

  async fn evaluate_in_channel(
    &self,
    _channel: &str,
    event: &Event,
  ) -> Result<RuntimeEvaluation, RuntimeEngineError> {
    self.evaluate(event).await
  }

  async fn reload(&self) -> Result<RulesetSnapshot, RuntimeEngineError> {
    let rules = {
      let state = self.state.read().expect("test runtime lock");
      state.rules.clone()
    };
    self.publish_rules(rules).await
  }

  fn status(&self) -> Result<RuleEngineStatus, RuntimeEngineError> {
    let state = self.state.read().expect("test runtime lock");
    Ok(RuleEngineStatus {
      mode: "dataflow-rs".to_owned(),
      ready: state.ready,
      version: state.version,
      loaded_rules: state.rules.len() as u32,
      compile_stats: RuleCompileStats {
        total_rules: state.rules.len() as u32,
        compiled_rules: state.rules.len() as u32,
        failed_rules: 0,
      },
    })
  }
}
