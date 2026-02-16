use axum::{
  Json,
  extract::{Path, Query, State},
  http::StatusCode,
};
use redis::AsyncCommands;
use semver::Version;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use rve_core::domain::{
  common::{Score, Severity},
  rule::*,
};

use crate::http::state::AppState;

#[derive(Deserialize)]
pub struct Pagination {
  pub page: Option<u32>,
  pub limit: Option<u32>,
}

/// Lists all rules with pagination
pub async fn list_rules(
  State(state): State<AppState>,
  Query(pagination): Query<Pagination>,
) -> Json<Vec<Rule>> {
  let _page = pagination.page.unwrap_or(1);
  let _limit = pagination.limit.unwrap_or(10);

  let mut r = state.store.get_connection().await;

  if let Ok(res) = r.set("foo", "bar").await {
    let res: String = res;
    println!("{res}"); // >>> OK
  } else {
    println!("Error setting foo");
  }

  match r.get("foo").await {
    Ok(res) => {
      let res: String = res;
      println!("{res}"); // >>> bar
    }
    Err(e) => {
      panic!("Error getting foo: {e}");
    }
  };

  Json(Vec::new())
}

/// Json extractor - extracts and deserializes JSON body
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
  name: String,
  email: String,
}

#[derive(Debug, Serialize)]
pub struct CreateUserResponse {
  id: u64,
  name: String,
  email: String,
}

pub async fn create_rule(Json(payload): Json<CreateUserRequest>) -> Json<CreateUserResponse> {
  Json(CreateUserResponse { id: 1, name: payload.name, email: payload.email })
}

pub async fn get_rule(Path(_id): Path<String>) -> Json<Rule> {
  Json(Rule {
    id: "FRAUD-HV-UNTRUSTED-01".into(),
    meta: RuleMeta {
      name: "High Value on Untrusted Device".into(),
      description: Some(
        "Dispara si el monto es > $5000 y el fingerprint del dispositivo es nuevo.".into(),
      ),
      version: Version::new(1, 0, 0),
      autor: "Analista".into(),
      tags: None,
    },
    state: RuleState {
      mode: mode::RuleMode::Active,
      audit: RuleAudit {
        created_at_ms: Some(1706790000000),
        updated_at_ms: Some(1707830000000),
        created_by: Some("Super User".into()),
        updated_by: Some("Analyst Jane".into()),
      },
    },
    schedule: RuleSchedule {
      // Activa desde el pasado, sin fecha de fin
      active_from_ms: Some(1700000000000),
      active_until_ms: None,
    },
    rollout: RolloutPolicy { percent: 100 },
    evaluation: RuleEvaluation {
      condition: Value::Bool(true),
      // Lógica: (Monto > 5000) AND (Trust Score < 0.4)
      logic: json!({
          "and": [
              { ">": [ { "var": "transaction.amount" }, 5000 ] },
              { "<": [ { "var": "device.trust_score" }, 0.4 ] }
          ]
      }),
    },
    enforcement: RuleEnforcement {
      score_impact: Score::new(8.5).unwrap(),
      action: RuleAction::Review,
      severity: Severity::High,
      tags: vec!["financial_fraud".into(), "device_fingerprinting".into(), "high_value".into()],
      cooldown_ms: Some(600_000),
    },
  })
}

pub async fn update_rule(Path(id): Path<String>, Json(_payload): Json<Value>) -> Json<Value> {
  Json(json!({
      "id": id,
      "status": "updated",
      "description": "Rule updated successfully"
  }))
}

pub async fn delete_rule(Path(_id): Path<String>) -> StatusCode {
  StatusCode::NO_CONTENT
}
